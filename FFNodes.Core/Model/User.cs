// LFInteractive LLC. 2021-2024﻿
using FFNodes.Server.Model;

namespace FFNodes.Core.Model;

public struct User
{
    public Guid Id { get; set; }
    public string Username { get; set; }
    public TimeSpan ActiveTime { get; set; }
    public DateTime Join { get; set; }
    public long Saved { get; set; }
    public ProcessedFile[] Files { get; set; }
}