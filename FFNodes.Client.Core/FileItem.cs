/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

namespace FFNodes.Client.GUI.Data;

public enum Operation
{
    Downloading,
    Uploading,
    Processing,
    Cancelling,
}

public sealed class FileItem
{
    public string FileName { get; set; }
    public float Percentage { get; set; }
    public Operation CurrentOperation { get; set; }

    public FileItem(string fileName, float percentage, Operation currentOperation)
    {
        FileName = fileName;
        Percentage = percentage;
        CurrentOperation = currentOperation;
    }
}