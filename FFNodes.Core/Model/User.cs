/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Newtonsoft.Json;

namespace FFNodes.Core.Model;

public sealed class User
{
    [JsonProperty("id")]
    public Guid Id { get; set; } = Guid.NewGuid();

    [JsonProperty("username")]
    public string Username { get; set; }

    [JsonProperty("active_time")]
    public TimeSpan ActiveTime { get; set; } = TimeSpan.Zero;

    [JsonProperty("joined")]
    public DateTime Joined { get; set; } = DateTime.Now;

    [JsonProperty("last_online")]
    public DateTime LastOnline { get; set; } = DateTime.Now;

    [JsonProperty("saved")]
    public long Saved { get; set; } = 0;

    [JsonProperty("admin")]
    public bool IsAdmin { get; set; } = false;

    [JsonProperty("processed_files")]
    public Guid[] Files { get; set; } = Array.Empty<Guid>();

    [JsonIgnore]
    public bool IsOnline => (DateTime.Now - LastOnline).TotalSeconds < 30;

    public User(string username)
    {
        Username = username ?? throw new ArgumentNullException(nameof(username));
    }
}