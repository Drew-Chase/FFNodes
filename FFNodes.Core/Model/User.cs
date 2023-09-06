/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Server.Model;

namespace FFNodes.Core.Model;

public struct User
{
    public Guid Id { get; set; }
    public string Username { get; set; }
    public TimeSpan ActiveTime { get; set; }
    public DateTime Join { get; set; }
    public long Saved { get; set; }
    public ProcessedFile[] Files { get; set; }
}