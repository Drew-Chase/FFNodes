/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Data;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Serilog;

namespace FFNodes.Server.Data;

public sealed class Configuration
{
    [JsonIgnore]
    public static Configuration Instance = Instance ??= new();

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

    [JsonIgnore]
    public DateTime StartDate { get; } = DateTime.Now;

    private Configuration()
    {
    }

    public void Save()
    {
        Log.Debug("Saving config file: {CONFIG}", Files.Config);
        using StreamWriter writer = File.CreateText(Files.Config);
        writer.Write(JsonConvert.SerializeObject(this, Formatting.Indented));
    }

    public void Load()
    {
        if (!File.Exists(Files.Config))
        {
            Save();
        }
        else
        {
            Log.Debug("Loading config file: {CONFIG}", Files.Config);
            Instance = JObject.Parse(File.ReadAllText(Files.Config))?.ToObject<Configuration>() ?? Instance;
        }
    }
}