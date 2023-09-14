/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.FFmpeg.Extra;
using FFNodes.Core.Data;
using FFNodes.Core.Model;
using FFNodes.Server.Data;
using FFNodes.Server.Model;
using Newtonsoft.Json;
using Serilog;

namespace FFNodes.Server.Handlers;

/// <summary>
/// Handles the processing of files.
/// </summary>
public sealed class FileSystemHandler
{
    /// <summary>
    /// The instance of the process handler.
    /// </summary>
    public static readonly FileSystemHandler Instance = Instance ??= new FileSystemHandler();

    private readonly Dictionary<User, List<ProcessedFile>> checkedOutFiles;
    private readonly List<FileSystemWatcher> watcherList;
    private List<ProcessedFile> processedFiles;

    /// <summary>
    /// If the processed files have finished loading.
    /// </summary>
    public bool FinishedLoading { get; set; } = false;

    private FileSystemHandler()
    {
        processedFiles = new();
        checkedOutFiles = new();
        watcherList = new();
    }

    public static string GetFileReportPath(Guid userId, Guid fileId) => Path.Combine(Directories.Users, userId.ToString("N")[..2], userId.ToString("N"), fileId.ToString("N")[..2], $"{fileId:N}.json");

    /// <summary>
    /// Checks out a file for a user.
    /// </summary>
    /// <param name="user"></param>
    /// <param name="count"></param>
    /// <returns></returns>
    public ProcessedFile CheckoutFile(User user)
    {
        ProcessedFile file = processedFiles.First();
        processedFiles.Remove(file);

        if (!checkedOutFiles.ContainsKey(user))
        {
            checkedOutFiles.Add(user, new());
        }
        checkedOutFiles[user].Add(file);

        return file;
    }

    /// <summary>
    /// Gets all the processed files.
    /// </summary>
    /// <returns></returns>
    public ProcessedFile[] GetProcessedFiles()
    {
        return processedFiles.ToArray();
    }

    /// <summary>
    /// Loads the processed files.
    /// </summary>
    public async Task Load()
    {
        if (!Configuration.Instance.Directories.Any())
        {
            Log.Warning("Please fill out the directories in the configuration file.");
            return;
        }
        Log.Information("Loading processed files...");
        processedFiles.Clear();
        FinishedLoading = false;

        // Wait for all directories to be loaded.
        Task.WaitAll(Configuration.Instance.Directories.Select(i => Load(i)).ToArray());

        // Sort the files.
        await Sort();
        FinishedLoading = true;
    }

    public async Task Load(string directory, int attempt = 0)
    {
        Log.Debug("Scanning {Directory}...", directory);
        try
        {
            processedFiles.AddRange(FFVideoUtility.GetFilesAsync(directory, Configuration.Instance.ScanRecursively).Select(i => new ProcessedFile(i)));
        }
        catch (Exception e)
        {
            Log.Error("Failed to scan {Directory} - {MSG}.", directory, e.Message, e);
            int maxAttempts = 5;
            if (attempt >= maxAttempts)
            {
                Log.Error("Failed to scan {Directory} after {MAX} attempts.", directory, maxAttempts);
                return;
            }
            Log.Error("Attempting again in 5 seconds...");
            await Task.Delay(TimeSpan.FromSeconds(5));
            await Load(directory, attempt + 1);
            return;
        }
        Log.Debug("Adding watcher for {Directory}...", directory);
        try
        {
            FileSystemWatcher watcher = new()
            {
                Path = directory,
                IncludeSubdirectories = Configuration.Instance.ScanRecursively,
                NotifyFilter = NotifyFilters.CreationTime | NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.Size | NotifyFilters.DirectoryName,
                EnableRaisingEvents = true,
            };
            watcher.Created += async (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath) && !processedFiles.Any(item => item.Path.Equals(e.FullPath)))
                {
                    processedFiles.Add(new(e.FullPath));
                    Log.Information("Watched file {File} was created.", e.FullPath);
                    await Sort();
                }
            };
            watcher.Renamed += async (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.OldFullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        processedFiles.Remove(processedFile.Value);
                    }
                    processedFiles.Add(new(e.FullPath));
                    Log.Information("Watched file {File} was renamed to {NewFile}.", e.OldFullPath, e.FullPath);
                    await Sort();
                }
            };
            watcher.Deleted += (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.FullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        Log.Information("Watched file {File} was deleted.", processedFile.Value.Path);
                        processedFiles.Remove(processedFile.Value);
                    }
                }
            };
            watcherList.Add(watcher);
        }
        catch (Exception e)
        {
            Log.Error("Failed to setup watcher for {Directory} - {MSG}.", directory, e.Message, e);
        }
    }

    public ProcessedFile[] GetCheckedOutFiles(User user)
    {
        if (checkedOutFiles.ContainsKey(user))
        {
            return checkedOutFiles[user].ToArray();
        }
        return Array.Empty<ProcessedFile>();
    }

    /// <summary>
    /// Writes the processed file to the file system.
    /// </summary>
    /// <param name="user"></param>
    /// <param name="file"></param>
    /// <returns></returns>
    public Task ReportProcessedFile(User user, ProcessedFile file) => Task.Run(() =>
    {
        List<Guid> files = user.Files.ToList();
        files.Add(file.Id);
        user.Files = files.ToArray();
        UserHandler.Instance.Save(user);
        string path = GetFileReportPath(user.Id, file.Id);
        if (!File.Exists(path))
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path) ?? "");
        }
        using StreamWriter writer = File.CreateText(path);
        writer.Write(JsonConvert.SerializeObject(file, Formatting.Indented));
        writer.Flush();
    });

    public ProcessedFile? LoadReportedFile(Guid userId, Guid fileId)
    {
        string file = GetFileReportPath(userId, fileId);
        if (File.Exists(file))
        {
            try
            {
                using StreamReader reader = File.OpenText(file);
                return JsonConvert.DeserializeObject<ProcessedFile>(reader.ReadToEnd());
            }
            catch (Exception e)
            {
                Log.Error("Failed to load file {File} - {ERROR}.", file, e.Message, e);
            }
        }
        return null;
    }

    /// <summary>
    /// Sorts the processed files.
    /// </summary>
    /// <returns></returns>
    public Task Sort() => Task.Run(() =>
    {
        Log.Warning("Beginning sort of processed files...");
        processedFiles = processedFiles.OrderByDescending(i => i.OriginalSize).ToList();
        GC.Collect(); // Cleans up memory of the old array.
        Log.Information("Finished sorting processed files.");
    });

    public bool TryParseFile(string filename, out ProcessedFile? file) => (file = ParseFile(filename)) != null;

    public ProcessedFile? ParseFile(string filename)
    {
        return processedFiles.FirstOrDefault(i => Path.GetFileNameWithoutExtension(i.Path).Equals(Path.GetFileNameWithoutExtension(filename)));
    }
}