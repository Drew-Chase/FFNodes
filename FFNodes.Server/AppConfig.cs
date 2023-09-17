/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CommonLib.FileSystem.Configuration;
using Newtonsoft.Json;
using Serilog.Events;

namespace FFNodes.Server.Data;

public sealed class AppConfig : AppConfigBase
{
    [JsonIgnore]
    public static readonly AppConfig Instance = Instance ??= new();

    [JsonProperty("port")]
    public int Port { get; set; } = 1818;

    [JsonProperty("media_directories")]
    public string[] Directories { get; set; } = Array.Empty<string>();

    [JsonProperty("scan_recursively")]
    public bool ScanRecursively { get; set; } = true;

    [JsonProperty("authorization_token")]
    public Guid AuthorizationToken { get; set; } = Guid.NewGuid();

    [JsonProperty("host")]
    public string Host { get; set; } = "localhost";

    [JsonProperty("ffmpeg_command")]
    public string FFmpegCommand { get; set; } = @"-y -hwaccel auto -i ""{INPUT}"" -c:v {CODEC} -c:a aac -filter_complex ""[0:a]dialoguenhance=original=0.5:enhance=2:voice=8, dynaudnorm[aout]"" -map ""[aout]"" -map 0:v ""{OUTPUT}.{EXTENSION}""";

    [JsonProperty("users")]
    public Guid[] Users { get; set; } = Array.Empty<Guid>();

    [JsonProperty("log-level")]
    public LogEventLevel DefaultLogLevel { get; set; } = LogEventLevel.Information;

    [JsonIgnore]
    public DateTime StartDate { get; } = DateTime.Now;
}