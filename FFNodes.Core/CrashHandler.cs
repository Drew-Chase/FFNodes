/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Data;
using Serilog;

namespace FFNodes.Core;

public static class CrashHandler
{
    public static void HandleCrash(Exception exception)
    {
        using (StreamWriter writer = File.CreateText(Path.Combine(Directories.CrashReports, $"crash-{DateTime.Now:MMM dd, yyyy - HH-mm-ss.fff}.log")))
        {
            writer.WriteLine($"FFNodes Crash Report - {DateTime.Now:MMM dd, yyyy - HH-mm-ss.fff}");
            writer.WriteLine(new string('=', 81));
            writer.WriteLine($"\nTL;DR: {exception.Source} - {exception.Message}\n");
            writer.WriteLine($"Stack Trace: {exception.StackTrace}");
            if (exception.Data != null && exception.Data.Count > 0)
            {
                writer.WriteLine($"Data:\n{string.Join(",\n", exception.Data)}");
            }
            writer.WriteLine($"HResult: {exception.HResult}");
        }
        Log.Fatal(exception, "Unhandled Exception");
        Log.CloseAndFlush();
    }
}