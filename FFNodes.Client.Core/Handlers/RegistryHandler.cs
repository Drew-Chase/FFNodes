/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Data;
using Microsoft.Win32;

namespace FFNodes.Client.Core.Handlers;

public static class RegistryHandler
{
    public static void RegisterProtocol()
    {
        if (OperatingSystem.IsWindows())
        {
            using RegistryKey key = Registry.CurrentUser.CreateSubKey(@"SOFTWARE\Classes\ffn");
            key.SetValue(string.Empty, "URL:FFNodes Protocol");
            key.SetValue("URL Protocol", string.Empty);
            using RegistryKey commandKey = key.CreateSubKey(@"shell\open\command");
            commandKey.SetValue(string.Empty, $"\"{ApplicationData.Executable}\" -c \"%1\"");
        }
    }

    public static bool IsProtocolExecutableValid()
    {
        string defaultCommand = $"\"{ApplicationData.Executable}\" -c \"%1\"";
        if (OperatingSystem.IsWindows())
        {
            using RegistryKey? key = Registry.CurrentUser.OpenSubKey(@"SOFTWARE\Classes\ffn\shell\open\command");
            if (key != null)
            {
                if (key.GetValue(string.Empty) is string command)
                {
                    return command == defaultCommand;
                }
            }
        }

        return false;
    }

    public static void UnregisterProtocol()
    {
        if (OperatingSystem.IsWindows())
        {
            Registry.CurrentUser.DeleteSubKeyTree(@"SOFTWARE\Classes\ffn", false);
        }
    }

    public static bool IsProtocolRegistered()
    {
        if (OperatingSystem.IsWindows())
        {
            using RegistryKey? key = Registry.CurrentUser.OpenSubKey(@"SOFTWARE\Classes\ffn");
            return key != null;
        }
        return false;
    }
}