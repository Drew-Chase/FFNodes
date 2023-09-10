/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using System.Reflection;

namespace FFNodes.Core.Data;

public static class Directories
{
    public static string Root { get; } = Directory.GetParent(Assembly.GetExecutingAssembly().Location)?.FullName ?? Directory.GetCurrentDirectory();
    public static string Data { get; } = Directory.CreateDirectory(Path.Combine(Root, "Data")).FullName;
    public static string Users { get; } = Directory.CreateDirectory(Path.Combine(Data, "Users")).FullName;
    public static string Logs { get; } = Directory.CreateDirectory(Path.Combine(Root, "Logs")).FullName;
}