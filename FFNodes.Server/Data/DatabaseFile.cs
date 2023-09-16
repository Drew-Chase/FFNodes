/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Newtonsoft.Json;
using System.IO.Compression;

namespace FFNodes.Server.Data;

public class DatabaseFile : IDisposable
{
    private readonly ZipArchive baseStream;

    public DatabaseFile(string filePath)
    {
        baseStream = ZipFile.Open(filePath, ZipArchiveMode.Update);
    }

    public static DatabaseFile Open(string filePath)
    {
        return new(filePath);
    }

    public void WriteEntry(Guid key, object value)
    {
        ZipArchiveEntry zipEntry = baseStream.GetEntry(ParseEntryPath(key)) ?? baseStream.CreateEntry(ParseEntryPath(key), CompressionLevel.SmallestSize);
        using Stream stream = zipEntry.Open();
        using StreamWriter writer = new(stream);
        writer.Write(JsonConvert.SerializeObject(value));
        writer.Flush();
        stream.Flush();
    }

    public T? ReadEntry<T>(Guid key)
    {
        ZipArchiveEntry? zipEntry = baseStream.GetEntry(ParseEntryPath(key));
        if (zipEntry != null)
        {
            using Stream stream = zipEntry.Open();
            using StreamReader reader = new(stream);
            return JsonConvert.DeserializeObject<T>(reader.ReadToEnd());
        }
        return default;
    }

    public bool Exists(Guid key)
    {
        ZipArchiveEntry? zipEntry = baseStream.GetEntry(ParseEntryPath(key));
        return zipEntry != null;
    }

    public void Dispose()
    {
        baseStream.Dispose();
        GC.SuppressFinalize(this);
    }

    private static string ParseEntryPath(Guid key) => Path.Combine(key.ToString("N")[..2], key.ToString("N"));
}