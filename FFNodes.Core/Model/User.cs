/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Server.Model;

namespace FFNodes.Core.Model;

public sealed class User
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Username { get; set; }
    public TimeSpan ActiveTime { get; set; } = TimeSpan.Zero;
    public DateTime Join { get; set; } = DateTime.Now;
    public DateTime LastOnline { get; set; } = DateTime.Now;
    public long Saved { get; set; } = 0;
    public bool IsAdmin { get; set; } = false;
    public ProcessedFile[] Files { get; set; } = Array.Empty<ProcessedFile>();

    public User(string username)
    {
        Username = username ?? throw new ArgumentNullException(nameof(username));
    }
}