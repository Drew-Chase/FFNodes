﻿/*
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
using Serilog;
using System.Diagnostics;
using Timer = System.Timers.Timer;

namespace FFNodes.Client.Core;

public class ProcessManager
{
    private readonly Timer pingTimer;
    public static ProcessManager Instance { get; } = Instance ??= new ProcessManager();
    public bool IsProcessing { get; set; } = false;
    public Dictionary<Guid, FileItem> Files { get; set; } = new();
    public EventHandler<EventArgs> OnUpdateEvent { get; set; } = (s, e) => { };
    private Dictionary<Guid, Process> ActiveProcesses { get; set; } = new();
    private FFAdvancedNetworkClient Client { get; }

    private ProcessManager()
    {
        Client = new FFAdvancedNetworkClient(ClientAppConfig.Instance.ConnectionUrl, ClientAppConfig.Instance.UserId);
        pingTimer = new(TimeSpan.FromSeconds(10))
        {
            AutoReset = true,
            Enabled = true,
        };
        pingTimer.Elapsed += async (s, e) =>
        {
            if (IsProcessing)
            {
                await Client.Ping();
            }
        };
        pingTimer.Start();
    }

    public void PauseProcessing()
    {
        if (IsProcessing)
        {
            IsProcessing = false;
        }
    }

    public async Task<Guid> DownloadNextFile(EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress = null)
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
                OnUpdateEvent?.Invoke(this, EventArgs.Empty);
            });
            return id;
        }
        return Guid.Empty;
    }

    public async Task<bool> ProcessFile(Guid fileId, EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress = null)
    {
        string cmd = "";
        try
        {
            if (!IsProcessing)
            {
                return false;
            }
            Log.Debug("Starting to process file: {FILE}.", Files[fileId].FileName);
            SystemStatusModel? status = await Client.GetSystemStatus();
            Files[fileId].CurrentOperation = Operation.Processing;
            string filePath = Path.Combine(ClientAppConfig.Instance.WorkingDirectory, Files[fileId].FileName);
            string outputDirectory = Directory.CreateDirectory(Path.Combine(ClientAppConfig.Instance.WorkingDirectory, "output")).FullName;
            string outputFile = Path.Combine(outputDirectory, Path.GetFileNameWithoutExtension(Files[fileId].FileName));
            string codec = ClientAppConfig.Instance.Codec;

            if (codec == "auto")
            {
                codec = ClientHandler.GetVendorSpecificCodec();
            }

            cmd = status.Value.FFMpegCommand
            .Replace("{INPUT}", $"{filePath}")
            .Replace("{CODEC}", codec)
            .Replace("{OUTPUT}", outputFile)
            .Replace("{EXTENSION}", Path.GetExtension(filePath)[1..]);

            string? newFileName = cmd.Split('"')[^2].TrimEnd('"');

            Log.Debug("Running FFMPEG with Command: {FILE}.", cmd);
            OnUpdateEvent?.Invoke(this, EventArgs.Empty);
            return await Task.Run(() =>
             {
                 FFMediaInfo info = new(filePath);
                 Process process = Chase.FFmpeg.FFProcessHandler.ExecuteFFmpeg(cmd, info: info, updated: (s, e) =>
                 {
                     if (string.IsNullOrWhiteSpace(newFileName))
                     {
                         newFileName = cmd.Split('"')[^2].TrimEnd('"');
                     }
                     long currentSize = new FileInfo(newFileName).Length;
                     if (currentSize >= (long)info.Size)
                     {
                         CancelProcessing(fileId);
                     }
                     Files[fileId].CurrentOperation = Operation.Processing;
                     Files[fileId].Percentage = e.Percentage;
                     fileItemProgress?.Invoke(this, new(Files[fileId]));
                     OnUpdateEvent?.Invoke(this, EventArgs.Empty);
                 }, auto_start: false);

                 process.Start();
                 ActiveProcesses.Add(fileId, process);
                 process.BeginErrorReadLine();
                 process.BeginOutputReadLine();
                 process.WaitForExit();
                 ActiveProcesses.Remove(fileId);
                 Log.Debug("Done processing file: {FILE}.", Files[fileId].FileName);
                 return process.ExitCode == 0;
             });
        }
        catch (Exception e)
        {
            Log.Error(e, "Unable to process file");
            CrashHandler.HandleCrash(e, new { ffmpeg_command = cmd });
            Environment.Exit(1);
        }
        OnUpdateEvent?.Invoke(this, EventArgs.Empty);
        return false;
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
        OnUpdateEvent?.Invoke(this, EventArgs.Empty);
    }

    public void CancelProcessing(Guid fileId)
    {
        if (ActiveProcesses.ContainsKey(fileId) && !ActiveProcesses[fileId].HasExited)
        {
            Files[fileId].CurrentOperation = Operation.Cancelling;
            Files[fileId].Percentage = 1;
            ActiveProcesses[fileId]?.Kill();
            ActiveProcesses.Remove(fileId);
        }
        OnUpdateEvent?.Invoke(this, EventArgs.Empty);
    }

    public async Task UploadFile(Guid fileId, TimeSpan duration, EventHandler<FileItemProgressUpdateEventArgs>? fileItemProgress = null)
    {
        if (IsProcessing)
        {
            Files[fileId].CurrentOperation = Operation.Uploading;

            string? file = Directory.GetFiles(Path.Combine(ClientAppConfig.Instance.WorkingDirectory, "output"), Path.GetFileNameWithoutExtension(Files[fileId].FileName) + ".*", SearchOption.TopDirectoryOnly).FirstOrDefault();
            if (!string.IsNullOrWhiteSpace(file) && File.Exists(file))
            {
                await Task.Run(async () =>
                {
                    await Client.CheckinFile(file, duration, (s, e) =>
                    {
                        Files[fileId].CurrentOperation = Operation.Uploading;
                        Files[fileId].Percentage = (float)e.Percentage;
                        fileItemProgress?.Invoke(this, new FileItemProgressUpdateEventArgs(Files[fileId]));
                        OnUpdateEvent?.Invoke(this, EventArgs.Empty);
                    });
                });
                File.Delete(file);
            }

            Files[fileId].CurrentOperation = Operation.Completed;
            Files[fileId].Percentage = 1f;
            OnUpdateEvent?.Invoke(this, EventArgs.Empty);
        }
    }

    private async Task CleanupFile(Guid fileId, int attempt = 0)
    {
        try
        {
            string? file = Directory.GetFiles(Path.Combine(ClientAppConfig.Instance.WorkingDirectory, "output"), Path.GetFileNameWithoutExtension(Files[fileId].FileName) + ".*", SearchOption.TopDirectoryOnly).FirstOrDefault();
            if (!string.IsNullOrWhiteSpace(file) && File.Exists(file))
            {
                File.Delete(file);
            }
        }
        catch (Exception e)
        {
            if (attempt < 5)
            {
                Log.Error(e, "Unable to delete file: {FILE}, Trying again in 5 seconds", Files[fileId].FileName);
                Thread.Sleep(TimeSpan.FromSeconds(5));
                await CleanupFile(fileId, attempt + 1);
            }
            else
            {
                Log.Error(e, "Unable to delete file: {FILE}, Manual deletion will have to occur.", Files[fileId].FileName);
            }
        }
    }
}