// LFInteractive LLC. 2021-2024﻿
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Serilog;
using System.Reflection;

namespace FFNodes.Server.Data;

public sealed class Configuration
{
    [JsonIgnore]
    public static Configuration Instance = Instance ??= new();

    private string configFile = Path.Combine(Assembly.GetExecutingAssembly().Location, "config.json");

    [JsonProperty("port")]
    public int Port { get; set; } = 1818;

    [JsonProperty("media_directories")]
    public string[] Directories { get; set; } = Array.Empty<string>();

    [JsonProperty("scan_recursively")]
    public bool ScanRecursively { get; set; } = true;

    private Configuration()
    {
    }

    public void Save()
    {
        Log.Debug("Saving config file: {CONFIG}", configFile);
        using StreamWriter writer = File.CreateText(configFile);
        writer.Write(JsonConvert.SerializeObject(this, Formatting.Indented));
    }

    public void Load()
    {
        if (!File.Exists(configFile))
        {
            Save();
        }
        else
        {
            Log.Debug("Loading config file: {CONFIG}", configFile);
            Instance = JObject.Parse(File.ReadAllText(configFile))?.ToObject<Configuration>() ?? Instance;
        }
    }
}