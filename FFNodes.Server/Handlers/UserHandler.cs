/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;

namespace FFNodes.Server.Handlers;

public class UserHandler
{
    public static readonly UserHandler Instance = Instance ??= new UserHandler();
    private readonly List<User> users;

    private UserHandler()
    {
        users = new();
    }

    public void Load()
    {
    }
}