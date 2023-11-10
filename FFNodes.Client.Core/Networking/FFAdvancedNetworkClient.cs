/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CommonLib.Events;
using Chase.CommonLib.Networking;
using FFNodes.Core.Model;
using Newtonsoft.Json.Linq;
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

    public async Task<string?> ResetConnectionCode()
    {
        using HttpRequestMessage request = new()
        {
            Method = HttpMethod.Post,
            RequestUri = new Uri($"{client.BaseAddress}api/reset-connection-code"),
        };
        JObject? json = await client.GetAsJson(request);
        if (json != null && json.ContainsKey("connection_url"))
        {
            return json["connection_url"]?.ToObject<string>();
        }
        return null;
    }

    public async Task<(bool, User? user)> LoginOrCreateUser(string username)
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
        return await client.DownloadFileAsync($"{client.BaseAddress}api/fs/checkout", Directory.CreateDirectory(ClientAppConfig.Instance.WorkingDirectory).FullName, downloadProgress);
    }

    public async Task<bool> Ping()
    {
        return (await client.GetAsync($"{client.BaseAddress}api/auth/connect")).IsSuccessStatusCode;
    }

    public async Task<bool> CheckinFile(string path, TimeSpan duration, DownloadProgressEvent? progress)
    {
        using HttpRequestMessage request = new()
        {
            RequestUri = new($"{client.BaseAddress}api/fs/checkin"),
            Method = HttpMethod.Post,
            Headers =
            {
                { "X-Duration", duration.Ticks.ToString() },
            },
        };
        using HttpResponseMessage response = await client.UploadFileAsync(request, path, progress);
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

    //public async Task<User[]> GetUsers()
    //{
    //using HttpResponseMessage response = await client.GetAsync($"{client.BaseAddress}api/auth/users");
    //if (response.IsSuccessStatusCode)
    //{
    //    string content = await response.Content.ReadAsStringAsync();
    //    return JsonSerializer.Deserialize<User[]>(content) ?? Array.Empty<User>();
    //}
    //    var json = await client.GetAsJsonArray($"{client.BaseAddress}api/auth/users");
    //    return json?.ToObject<User[]>() ?? Array.Empty<User>();
    //}

    public async Task<User[]> GetUsers()
    {
        JArray? v = await client.GetAsJsonArray($"{client.BaseAddress}api/auth/users");
        return v?.ToObject<User[]>() ?? Array.Empty<User>();
    }

    public void Dispose()
    {
        GC.SuppressFinalize(this);
        client.Dispose();
    }
}