/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Client.GUI.Data;

namespace FFNodes.Client.Core.Events;

public class FileItemProgressUpdateEventArgs : EventArgs
{
    public FileItem Item { get; set; }

    public FileItemProgressUpdateEventArgs(FileItem item)
    {
        Item = item;
    }
}