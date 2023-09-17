/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CommonLib.Events;
using Chase.CommonLib.Networking;
using FFNodes.Core.Model;
using FFNodes.Server.Data;
using System.Text;
using System.Text.Json;

namespace FFNodes.Client.Core.Networking;

public class FFAdvancedNetworkClient : IDisposable
{
    private readonly AdvancedNetworkClient client;

    public FFAdvancedNetworkClient(string connectionUrl) : this()
    {
        Uri uri = new(connectionUrl);
        client.DefaultRequestHeaders.Add("Authentication", uri.LocalPath[1..]);
        client.BaseAddress = new Uri($"http://{uri.Host}:{uri.Port}");
    }

    public FFAdvancedNetworkClient(string connectionUrl, Guid userId) : this(connectionUrl)
    {
        client.DefaultRequestHeaders.Add("User-ID", userId.ToString());
    }

    private FFAdvancedNetworkClient()
    {
        client = new AdvancedNetworkClient();
    }

    public async Task<SystemStatusModel?> GetSystemStatus()
    {
        return (await client.GetAsJson($"{client.BaseAddress}api"))?.ToObject<SystemStatusModel>();
    }

    public async Task<(bool, User? user)> CreateUser(string username)
    {
        using HttpRequestMessage request = new(HttpMethod.Post, $"{client.BaseAddress}api/auth/user")
        {
            Content = new StringContent(JsonSerializer.Serialize(new User(username)), Encoding.UTF8, "application/json")
        };
        try
        {
            User? user = (await client.GetAsJson(request))?.ToObject<User>();
            return (user != null, user);
        }
        catch
        {
            return (false, null);
        }
    }

    public async Task<string> CheckoutFile(DownloadProgressEvent downloadProgress)
    {
        return await client.DownloadFileAsync($"{client.BaseAddress}api/fs/checkout", Directory.CreateDirectory(AppConfig.Instance.WorkingDirectory).FullName, downloadProgress);
    }

    public async Task<bool> Ping()
    {
        return (await client.GetAsync($"{client.BaseAddress}api/auth/connect")).IsSuccessStatusCode;
    }

    public async Task<bool> CheckinFile(string path, DownloadProgressEvent? progress)
    {
        using HttpResponseMessage response = await client.UploadFileAsync($"{client.BaseAddress}api/fs/checkin", path, progress);
        return response.IsSuccessStatusCode;
    }

    public async Task<(bool, User? user)> LogInUser(Guid userId)
    {
        try
        {
            client.DefaultRequestHeaders.Add("User-ID", userId.ToString());
            User? user = (await client.GetAsJson($"{client.BaseAddress}api/auth/user"))?.ToObject<User>();
            return (user != null, user);
        }
        catch
        {
            return (false, null);
        }
    }

    public async Task<User[]> GetUsers() => (await client.GetAsJson($"{client.BaseAddress}api/auth/users"))?.ToObject<User[]>() ?? Array.Empty<User>();

    public void Dispose()
    {
        GC.SuppressFinalize(this);
        client.Dispose();
    }
}