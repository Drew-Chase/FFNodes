// LFInteractive LLC. 2021-2024﻿
namespace FFNodes.Server.Model;

public struct ProcessedFile
{
    public string Name { get; set; }
    public string Path { get; set; }
    public long OriginalSize { get; set; }
    public long CompressedSize { get; set; }
    public bool Successful { get; set; }
    public TimeSpan Duration { get; set; }
    public DateTime Completed { get; set; }
    public bool HasProcessed { get; set; }

    public ProcessedFile(string path)
    {
        FileInfo info = new(path);
        Name = System.IO.Path.GetFileName(path);
        Path = path;
        OriginalSize = info.Length;
        HasProcessed = false;
        Completed = DateTime.MinValue;
        Duration = TimeSpan.Zero;
        CompressedSize = info.Length;
        Successful = false;
    }
}