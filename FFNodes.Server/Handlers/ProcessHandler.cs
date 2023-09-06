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

namespace FFNodes.Server.Handlers;

/// <summary>
/// Handles the processing of files.
/// </summary>
public sealed class ProcessHandler
{
    /// <summary>
    /// The instance of the process handler.
    /// </summary>
    public static readonly ProcessHandler Instance = Instance ??= new ProcessHandler();

    private readonly List<ProcessedFile> processedFiles;
    private readonly Dictionary<User, List<ProcessedFile>> checkedOutFiles;

    private ProcessHandler()
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
    public void Load()
    {
        Parallel.ForEach(Configuration.Instance.Directories, directory =>
        {
            processedFiles.AddRange(FFVideoUtility.GetFilesAsync(directory, Configuration.Instance.ScanRecursively).Select(i => new ProcessedFile(i)));
            FileSystemWatcher watcher = new()
            {
                EnableRaisingEvents = true,
                IncludeSubdirectories = Configuration.Instance.ScanRecursively,
                NotifyFilter = NotifyFilters.CreationTime,
                Path = directory
            };
            watcher.Created += (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    processedFiles.Add(new(e.FullPath));
                }
            };
            watcher.Renamed += (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.OldFullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        processedFiles.Remove(processedFile.Value);
                    }
                    processedFiles.Add(new(e.FullPath));
                }
            };
            watcher.Deleted += (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.FullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        processedFiles.Remove(processedFile.Value);
                    }
                }
            };
            watcher.Changed += (s, e) =>
            {
                if (FFVideoUtility.HasVideoExtension(e.FullPath))
                {
                    ProcessedFile? processedFile = processedFiles.FirstOrDefault(i => i.Path == e.FullPath);
                    if (processedFile != null && processedFile.HasValue)
                    {
                        processedFiles.Remove(processedFile.Value);
                    }
                }
            };
        });
    }
}