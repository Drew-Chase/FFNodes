/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using System.Diagnostics;

namespace FFNodes.Client.Data;

/// <summary>
/// Handles client specific operations.
/// </summary>
public static class ClientHandler
{
    /// <summary>
    /// Gets the current GPU vendor.
    /// </summary>
    /// <returns></returns>
    public static string GetGPUVendor()
    {
        string filename, argument, vendor = "";
        if (OperatingSystem.IsWindows())
        {
            filename = "powershell";
            argument = "Get-CimInstance -ClassName Win32_VideoController | Select-Object -Property Name";
        }
        else
        {
            filename = "lspci";
            argument = "-nn | grep VGA | cut -d ' ' -f 3 | cut -d ':' -f 1 | xargs -I{} -n1 lspci -s {} -v -nn | grep 'Kernel driver in use:'";
        }

        Process process = new()
        {
            StartInfo = new()
            {
                FileName = filename,
                Arguments = argument,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            },
            EnableRaisingEvents = true,
        };

        process.OutputDataReceived += (s, e) =>
        {
            string? data = e.Data;
            if (!string.IsNullOrWhiteSpace(data))
            {
                if (OperatingSystem.IsWindows())
                {
                    vendor = data;
                }
                else
                {
                    int startIndex = data.IndexOf('[');
                    int endIndex = data.IndexOf(']');
                    if (startIndex >= 0 && endIndex > startIndex)
                    {
                        vendor = data.Substring(startIndex + 1, endIndex - startIndex - 1);
                    }
                }
            }
        };

        process.Start();
        process.BeginOutputReadLine();
        process.BeginErrorReadLine();
        process.WaitForExit();

        return vendor;
    }
}