/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.FFmpeg.Info;
using FFNodes.Client.Core.Data;
using FFNodes.Client.Core.Events;
using FFNodes.Client.Core.Networking;
using FFNodes.Client.GUI.Data;
using FFNodes.Core;
using FFNodes.Core.Model;
using FFNodes.Server.Data;
using Serilog;
using System.Diagnostics;

namespace FFNodes.Client.Core;

public class ProcessManager
{
    public static ProcessManager Instance { get; } = Instance ??= new ProcessManager();
    public bool IsProcessing { get; set; } = false;
    public Dictionary<Guid, FileItem> Files { get; set; } = new();
    private FFNetworkClient Client { get; set; }
    private Dictionary<Guid, Process> ActiveProcesses { get; set; } = new();

    private ProcessManager()
    {
        Client = new(Configuration.Instance.ConnectionUrl, Configuration.Instance.UserId);
    }

    public void PauseProcessing()
    {
        if (IsProcessing)
        {
            IsProcessing = false;
        }
    }

    public async Task<Guid> DownloadNextFile(EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress)
    {
        if (IsProcessing)
        {
            Log.Debug("Downloading Next File.");
            Guid id = Guid.NewGuid();
            await Client.CheckoutFile((s, e) =>
            {
                if (!Files.ContainsKey(id))
                {
                    Files.Add(id, new FileItem(e.FileName, (float)e.Percentage, Operation.Downloading));
                }
                Files[id] = new FileItem(e.FileName, (float)e.Percentage, Operation.Downloading);
                fileItemProgress?.Invoke(this, new(Files[id]));
            });
            return id;
        }
        return Guid.Empty;
    }

    public async Task ProcessFile(Guid fileId, EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress)
    {
        try
        {
            Log.Debug("Starting to process file: {FILE}.", Files[fileId].FileName);
            SystemStatusModel? status = await Client.GetSystemStatus();
            Files[fileId].CurrentOperation = Operation.Processing;
            string filePath = Path.Combine(Configuration.Instance.WorkingDirectory, Files[fileId].FileName);
            string outputDirectory = Directory.CreateDirectory(Path.Combine(Configuration.Instance.WorkingDirectory, "output")).FullName;
            string outputFile = Path.Combine(outputDirectory, Path.GetFileNameWithoutExtension(Files[fileId].FileName));
            string codec = Configuration.Instance.Codec;

            if (codec == "auto")
            {
                codec = ClientHandler.GetVendorSpecificCodec();
            }

            string cmd = status.Value.FFMpegCommand
            .Replace("{INPUT}", $"{filePath}")
            .Replace("{CODEC}", codec)
            .Replace("{OUTPUT}", outputFile)
            .Replace("{EXTENSION}", Path.GetExtension(filePath)[1..]);

            Log.Debug("Running FFMPEG with Command: {FILE}.", cmd);
            await Task.Run(() =>
            {
                FFMediaInfo info = new(filePath);
                string newFileName = "";
                Process process = Chase.FFmpeg.FFProcessHandler.ExecuteFFmpeg(cmd, info: info, updated: (s, e) =>
                {
                    if (string.IsNullOrWhiteSpace(newFileName))
                    {
                        newFileName = Directory.GetFiles(outputDirectory, Path.GetFileNameWithoutExtension(outputFile) + ".*", SearchOption.TopDirectoryOnly).First();
                    }
                    long currentSize = new FileInfo(newFileName).Length;
                    if (currentSize >= (long)info.Size)
                    {
                        CancelProcessing(fileId);
                    }
                    Files[fileId].CurrentOperation = Operation.Processing;
                    Files[fileId].Percentage = e.Percentage;
                    fileItemProgress?.Invoke(this, new(Files[fileId]));
                }, auto_start: false);

                process.Start();
                ActiveProcesses.Add(fileId, process);
                process.BeginErrorReadLine();
                process.BeginOutputReadLine();
                process.WaitForExit();
                ActiveProcesses.Remove(fileId);
                Log.Debug("Done processing file: {FILE}.", Files[fileId].FileName);
            });
        }
        catch (Exception e)
        {
            Log.Error(e, "Unable to process file");
            CrashHandler.HandleCrash(e);
            Environment.Exit(1);
        }
    }

    public void CancelProcessing()
    {
        if (IsProcessing)
        {
            IsProcessing = false;

            foreach (Guid fileId in ActiveProcesses.Keys)
            {
                CancelProcessing(fileId);
            }
        }
    }

    public void CancelProcessing(Guid fileId)
    {
        if (ActiveProcesses.ContainsKey(fileId) && !ActiveProcesses[fileId].HasExited)
        {
            Files[fileId].CurrentOperation = Operation.Cancelling;
            Files[fileId].Percentage = 1;
            ActiveProcesses[fileId].Kill();
            ActiveProcesses.Remove(fileId);
            string? file = Directory.GetFiles(Path.Combine(Configuration.Instance.WorkingDirectory, "output"), Path.GetFileNameWithoutExtension(Files[fileId].FileName) + ".*", SearchOption.TopDirectoryOnly).FirstOrDefault();
            if (!string.IsNullOrWhiteSpace(file) && File.Exists(file))
            {
                File.Delete(file);
            }
        }
    }

    public async Task UploadFile(Guid fileId, EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress)
    {
        Files[fileId].CurrentOperation = Operation.Uploading;

        string? file = Directory.GetFiles(Path.Combine(Configuration.Instance.WorkingDirectory, "output"), Path.GetFileNameWithoutExtension(Files[fileId].FileName) + ".*", SearchOption.TopDirectoryOnly).FirstOrDefault();
        if (!string.IsNullOrWhiteSpace(file) && File.Exists(file))
        {
            await Client.CheckinFile(file, (s, e) =>
            {
                Files[fileId].CurrentOperation = Operation.Uploading;
                Files[fileId].Percentage = (float)e.Percentage;
                fileItemProgress?.Invoke(this, new FileItemProgressUpdateEventArgs(Files[fileId]));
            });
            Files.Remove(fileId);
            File.Delete(file);
        }
    }
}