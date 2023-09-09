/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.Networking;
using FFNodes.Core.Model;

namespace FFNodes.Client.Core.Networking;

public class FFNetworkClient : IDisposable
{
    private readonly NetworkClient client;

    public FFNetworkClient(string connectionUrl, Guid userId)
    {
        client = new NetworkClient();
        Uri uri = new(connectionUrl);
        client.DefaultRequestHeaders.Add("Authentication", uri.LocalPath[1..]);
        client.DefaultRequestHeaders.Add("User-ID", userId.ToString());
        client.BaseAddress = new Uri($"http://{uri.Host}:{uri.Port}");
    }

    public async Task<SystemStatusModel?> GetSystemStatus()
    {
        return (await client.GetAsJson($"{client.BaseAddress}api"))?.ToObject<SystemStatusModel>();
    }

    public void Dispose()
    {
        GC.SuppressFinalize(this);
        client.Dispose();
    }
}