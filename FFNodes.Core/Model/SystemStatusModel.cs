/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Newtonsoft.Json;

namespace FFNodes.Core.Model;

public struct SystemStatusModel
{
    [JsonProperty("uptime")]
    public TimeSpan Uptime { get; }

    [JsonProperty("loading")]
    public bool Loading { get; }

    [JsonProperty("connected_users")]
    public User[] ConnectedUsers { get; }

    [JsonProperty("connection_url")]
    public string ConnectionUrl { get; set; }

    [JsonProperty("ffmpeg_command")]
    public string FFMpegCommand { get; set; }

    [JsonProperty("only_keep_if_smaller")]
    public bool OnlyKeepIfSmaller { get; set; }

    /// <summary>
    /// Creates a new instance of the SystemStatusModel struct.
    /// </summary>
    /// <param name="uptime">The application runtime</param>
    /// <param name="loading">If the file system is still loading results</param>
    /// <param name="connectedUsers">A list of all connected users</param>
    /// <param name="connectionUrl">The connection url</param>
    public SystemStatusModel(TimeSpan uptime, bool loading, User[] connectedUsers, string connectionUrl, string ffmpegCommand, bool onlyKeepIfSmaller)
    {
        Uptime = uptime;
        Loading = loading;
        ConnectedUsers = connectedUsers;
        ConnectionUrl = connectionUrl;
        FFMpegCommand = ffmpegCommand;
        OnlyKeepIfSmaller = onlyKeepIfSmaller;
    }
}