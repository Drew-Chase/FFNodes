// LFInteractive LLC. 2021-2024﻿
using CLMath;

namespace FFNodes.Server.Data;

public static class Data
{
    public static string ConnectionString { get; } = CLAESMath.EncryptStringAES($"{Environment.MachineName}")[..2];
}