/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CommonLib.Math;

namespace FFNodes.Server.Data;

public static class Data
{
    public static string ConnectionString { get; private set; } = new Crypt().Encrypt(ServerAppConfig.Instance.AuthorizationToken.ToString("N"));
    public static string ConnectionUrl { get; private set; } = $"ffn://{ServerAppConfig.Instance.Host}:{ServerAppConfig.Instance.Port}/{ConnectionString}";

    public static bool ValidConnection(string code) => code.Equals(ConnectionString);

    public static void ResetConnectionCode()
    {
        ServerAppConfig.Instance.AuthorizationToken = Guid.NewGuid();
        ServerAppConfig.Instance.Save();
        ConnectionString = new Crypt().Encrypt(ServerAppConfig.Instance.AuthorizationToken.ToString("N"));
        ConnectionUrl = $"ffn://{ServerAppConfig.Instance.Host}:{ServerAppConfig.Instance.Port}/{ConnectionString}";
    }
}