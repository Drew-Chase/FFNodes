/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using System.Diagnostics;

namespace FFNodes.Client.Core.Data;

public enum GPUVendor
{
    Unknown,
    NVIDIA,
    AMD,
    Intel
}

/// <summary>
/// Handles client specific operations.
/// </summary>
public static class ClientHandler
{
    /// <summary>
    /// Gets the current GPU vendor.
    /// </summary>
    /// <returns></returns>
    public static GPUVendor GetGPUVendor()
    {
        string filename, argument;
        GPUVendor vendor = GPUVendor.Unknown;
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
                    if (data.Contains("amd", StringComparison.CurrentCultureIgnoreCase))
                    {
                        vendor = GPUVendor.AMD;
                    }
                    else if (data.Contains("nvidia", StringComparison.CurrentCultureIgnoreCase))
                    {
                        vendor = GPUVendor.NVIDIA;
                    }
                    else if (data.Contains("intel", StringComparison.CurrentCultureIgnoreCase))
                    {
                        vendor = GPUVendor.Intel;
                    }
                }
                else
                {
                    int startIndex = data.IndexOf('[');
                    int endIndex = data.IndexOf(']');
                    if (startIndex >= 0 && endIndex > startIndex)
                    {
                        string content = data.Substring(startIndex + 1, endIndex - startIndex - 1);
                        if (content.Contains("nvidia", StringComparison.CurrentCultureIgnoreCase))
                        {
                            vendor = GPUVendor.NVIDIA;
                        }
                        else if (content.Contains("amd", StringComparison.CurrentCultureIgnoreCase))
                        {
                            vendor = GPUVendor.AMD;
                        }
                        else if (content.Contains("intel", StringComparison.CurrentCultureIgnoreCase))
                        {
                            vendor = GPUVendor.Intel;
                        }
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

    public static string GetVendorSpecificCodec()
    {
        GPUVendor vendor = ClientHandler.GetGPUVendor();

        return vendor switch
        {
            GPUVendor.NVIDIA => "h264_nvenc",
            GPUVendor.AMD => "h264_amf",
            GPUVendor.Intel => "h264_qsv",
            _ => "h264",
        };
    }
}