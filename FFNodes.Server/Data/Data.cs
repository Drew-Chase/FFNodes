/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using CLMath;

namespace FFNodes.Server.Data;

public static class Data
{
    public static string ConnectionString { get; private set; } = CLAESMath.EncryptStringAES(Configuration.Instance.AuthorizationToken.ToString("N")).Replace("==", "");
    public static string ConnectionUrl { get; private set; } = $"ffn://{Configuration.Instance.Host}:{Configuration.Instance.Port}/{ConnectionString}";

    public static bool ValidConnection(string code) => code.Equals(ConnectionString);

    public static void ResetConnectionCode()
    {
        Configuration.Instance.AuthorizationToken = Guid.NewGuid();
        Configuration.Instance.Save();
        ConnectionString = CLAESMath.EncryptStringAES(Configuration.Instance.AuthorizationToken.ToString("N")).Replace("==", "");
        ConnectionUrl = $"ffn://{Configuration.Instance.Host}:{Configuration.Instance.Port}/{ConnectionString}";
    }
}