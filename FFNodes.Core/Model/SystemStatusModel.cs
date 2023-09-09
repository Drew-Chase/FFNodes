/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

namespace FFNodes.Core.Model;

public struct SystemStatusModel
{
    public TimeSpan Uptime { get; }
    public bool Loading { get; }
    public User[] ConnectedUsers { get; }
    public string ConnectionUrl { get; set; }

    /// <summary>
    /// Creates a new instance of the SystemStatusModel struct.
    /// </summary>
    /// <param name="uptime">The application runtime</param>
    /// <param name="loading">If the file system is still loading results</param>
    /// <param name="connectedUsers">A list of all connected users</param>
    /// <param name="connectionUrl">The connection url</param>
    public SystemStatusModel(TimeSpan uptime, bool loading, User[] connectedUsers, string connectionUrl)
    {
        Uptime = uptime;
        Loading = loading;
        ConnectedUsers = connectedUsers;
        ConnectionUrl = connectionUrl;
    }
}