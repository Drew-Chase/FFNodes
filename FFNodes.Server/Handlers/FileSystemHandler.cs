/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.FFmpeg.Extra;
using FFNodes.Core.Model;
using FFNodes.Server.Data;
using FFNodes.Server.Model;
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
    private List<ProcessedFile> processedFiles;

    /// <summary>
    /// If the processed files have finished loading.
    /// </summary>
    public bool FinishedLoading { get; set; } = false;

    private FileSystemHandler()
    {
        processedFiles = new();
        checkedOutFiles = new();
    }

    /// <summary>
    /// Checks out a number of files for a user.
    /// </summary>
    /// <param name="user"></param>
    /// <param name="count"></param>
    /// <returns></returns>
    public ProcessedFile[] CheckoutFiles(User user, int count)
    {
        List<ProcessedFile> files = new();

        for (int i = 0; i < count; i++)
        {
            ProcessedFile file = processedFiles[i];
            files.Add(file);
            processedFiles.Remove(file);
        }

        return (checkedOutFiles[user] = files).ToArray();
    }

    /// <summary>
    /// Checks in a number of files for a user.
    /// </summary>
    /// <param name="user"></param>
    /// <param name="files"></param>
    public void CheckinFiles(User user, ProcessedFile[] files)
    {
        List<ProcessedFile> alreadyProcessed = user.Files.ToList();
        foreach (ProcessedFile file in files)
        {
            if (checkedOutFiles[user].Contains(file) && !alreadyProcessed.Contains(file))
            {
                processedFiles.Add(file);
                alreadyProcessed.Add(file);
                checkedOutFiles[user].Remove(file);
            }
        }
        user.Files = alreadyProcessed.ToArray();
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
        Parallel.ForEach(Configuration.Instance.Directories, directory =>
        {
            Log.Debug("Scanning {Directory}...", directory);
            processedFiles.AddRange(FFVideoUtility.GetFilesAsync(directory, Configuration.Instance.ScanRecursively).Select(i => new ProcessedFile(i)));
            Log.Debug("Adding watcher for {Directory}...", directory);
            FileSystemWatcher watcher = new()
            {
                EnableRaisingEvents = true,
                IncludeSubdirectories = Configuration.Instance.ScanRecursively,
                NotifyFilter = NotifyFilters.CreationTime,
                Path = directory
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
            watcher.Deleted += async (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.FullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        Log.Information("Watched file {File} was deleted.", processedFile.Value.Path);
                        processedFiles.Remove(processedFile.Value);
                        await Sort();
                    }
                }
            };
        });
        await Sort();
        FinishedLoading = true;
    }

    /// <summary>
    /// Sorts the processed files.
    /// </summary>
    /// <returns></returns>
    public Task Sort() => Task.Run(() =>
    {
        Log.Warning("Beginning sort of processed files...");
        processedFiles = processedFiles.OrderByDescending(i => i.OriginalSize).ToList();
        GC.Collect();
        Log.Information("Finished sorting processed files.");
    });
}