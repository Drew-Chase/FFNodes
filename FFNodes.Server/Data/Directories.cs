// LFInteractive LLC. 2021-2024﻿
using System.Reflection;

namespace FFNodes.Server.Data;

public static class Directories
{
    public static string Root { get; } = Directory.GetParent(Assembly.GetExecutingAssembly().Location)?.FullName ?? Directory.GetCurrentDirectory();
    public static string Data { get; } = Directory.CreateDirectory(Path.Combine(Root, "Data")).FullName;
    public static string Users { get; } = Directory.CreateDirectory(Path.Combine(Data, "Users")).FullName;
    public static string Logs { get; } = Directory.CreateDirectory(Path.Combine(Root, "Logs")).FullName;
}