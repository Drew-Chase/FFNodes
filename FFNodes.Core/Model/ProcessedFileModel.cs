// LFInteractive LLC. 2021-2024﻿
namespace FFNodes.Server.Model;

public struct ProcessedFileModel
{
    public string Name { get; set; }
    public long OriginalSize { get; set; }
    public long CompressedSize { get; set; }
    public bool Successful { get; set; }
    public TimeSpan Duration { get; set; }
    public DateTime Completed { get; set; }
}