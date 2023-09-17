/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CommonLib.FileSystem.Configuration;
using FFNodes.Core.Data;
using Newtonsoft.Json;

namespace FFNodes.Server.Data;

public sealed class AppConfig : AppConfigBase<AppConfig>
{
    [JsonProperty("user-id")]
    public Guid UserId { get; set; }

    [JsonProperty("connection-url")]
    public string ConnectionUrl { get; set; }

    [JsonProperty("concurrent-connections")]
    public int ConcurrentConnections { get; set; } = 1;

    [JsonProperty("codec")]
    public string Codec { get; set; } = "auto";

    [JsonProperty("working-directory")]
    public string WorkingDirectory { get; set; } = System.IO.Path.Combine(Directories.Data, "tmp");
}