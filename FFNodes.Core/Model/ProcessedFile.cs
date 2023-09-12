/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

namespace FFNodes.Server.Model;

public struct ProcessedFile
{
    public Guid Id { get; set; }
    public string Path { get; set; }
    public long OriginalSize { get; set; }
    public long CompressedSize { get; set; }
    public bool Successful { get; set; }
    public TimeSpan Duration { get; set; }
    public DateTime Completed { get; set; }
    public bool HasProcessed { get; set; }

    public ProcessedFile(string path)
    {
        Id = Guid.NewGuid();
        FileInfo info = new(path);
        Path = path;
        OriginalSize = info.Length;
        HasProcessed = false;
        Completed = DateTime.MinValue;
        Duration = TimeSpan.Zero;
        CompressedSize = info.Length;
        Successful = false;
    }
}