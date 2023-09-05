// LFInteractive LLC. 2021-2024﻿
namespace FFNodes.Server.Data;

using static Directories;

public static class Files
{
    public static string Config { get; } = Path.Combine(Data, "config.json");
    public static string LatestLog { get; } = Path.Combine(Logs, "latest.log");
    public static string DebugLog { get; } = Path.Combine(Logs, "debug.log");
}