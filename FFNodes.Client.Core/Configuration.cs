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
    public delegate void ConfigurationSavedEventHandler(object sender, EventArgs e);

    public event ConfigurationSavedEventHandler ConfigurationSaved;

    [JsonIgnore]
    public static Configuration Instance = Instance ??= new();

    [JsonProperty("user-id")]
    public Guid UserId { get; set; }

    [JsonProperty("connection-url")]
    public string ConnectionUrl { get; set; }

    private Configuration()
    {
    }

    public void Save()
    {
        Log.Debug("Saving config file: {CONFIG}", Files.Config);
        using StreamWriter writer = File.CreateText(Files.Config);
        writer.Write(JsonConvert.SerializeObject(this, Formatting.Indented));
        writer.Flush();

        ConfigurationSaved?.Invoke(this, EventArgs.Empty);
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