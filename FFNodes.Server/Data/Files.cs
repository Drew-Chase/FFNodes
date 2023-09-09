/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

namespace FFNodes.Server.Data;

using static Directories;

public static class Files
{
    public static string Config { get; } = Path.Combine(Directories.Data, "config.json");
    public static string LatestLog { get; } = Path.Combine(Logs, "latest.log");
    public static string ErrorLog { get; } = Path.Combine(Logs, "error.log");
    public static string DebugLog { get; } = Path.Combine(Logs, "debug.log");
}