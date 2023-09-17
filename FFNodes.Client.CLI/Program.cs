/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CLIParser;
using Chase.CommonLib.Math;
using FFNodes.Client.Core.Networking;
using FFNodes.Core.Model;

namespace FFNodes.Client.CLI;

internal class Program
{
    private static async Task Main(string[] args)
    {
        OptionsManager optionsManager = new("FFNodes");
        optionsManager.Add(new Option() { ShortName = "s", LongName = "status", HasArgument = false, Required = false, Description = "Gets the status of the server." });
        optionsManager.Add(new Option() { ShortName = "c", LongName = "connection", HasArgument = true, Required = true, Description = "The connection url provided by the server owner." });
        optionsManager.Add(new Option() { ShortName = "u", LongName = "user", HasArgument = true, Required = true, Description = "The connecting user's id." });
        optionsManager.Add(new Option() { ShortName = "ss", LongName = "start", HasArgument = false, Required = false, Description = "Start processing files" });
        OptionsParser parser = optionsManager.Parse(args);

        if (parser != null)
        {
            if (parser.IsPresent("c", out string connectionUrl) && parser.IsPresent("u", out string userIdString) && Guid.TryParse(userIdString, out Guid userId))
            {
                using FFAdvancedNetworkClient client = new(connectionUrl, userId);
                if (parser.IsPresent("s"))
                {
                    SystemStatusModel? status;
                    if ((status = await client.GetSystemStatus()).HasValue)
                    {
                        Console.ForegroundColor = ConsoleColor.Green;
                        Console.Write("Uptime: ");
                        Console.ForegroundColor = ConsoleColor.Blue;
                        Console.WriteLine(status.Value.Uptime);

                        Console.ForegroundColor = ConsoleColor.Green;
                        Console.Write("Is Loading: ");
                        Console.ForegroundColor = ConsoleColor.Blue;
                        Console.WriteLine(status.Value.Loading);

                        Console.ForegroundColor = ConsoleColor.Green;
                        Console.Write("Connection Url: ");
                        Console.ForegroundColor = ConsoleColor.Blue;
                        Console.WriteLine(status.Value.ConnectionUrl);

                        Console.ForegroundColor = ConsoleColor.Green;
                        Console.Write("Connected Users: ");
                        Console.ForegroundColor = ConsoleColor.Blue;
                        Console.WriteLine(string.Join(',', status.Value.ConnectedUsers?.Select(i => i.Username) ?? Array.Empty<string>()));
                        Console.ResetColor();
                    }
                }
                else if (parser.IsPresent("ss"))
                {
                    int max = 0;
                    await client.CheckoutFile((s, e) =>
                    {
                        Console.Write(new string(' ', max));
                        Console.CursorLeft = 0;
                        string line = $"Downloading: {e.FileName} - {AdvancedFileInfo.SizeToString((long)e.BytesPerSecond, unit: FileSizeUnit.Bits)}ps - {e.Percentage:P2}";
                        max = Math.Max(max, line.Length);
                        Console.Write(line);
                    });
                }
            }
        }
        await Console.Out.WriteLineAsync("Press any key to continue...");
        Console.ReadKey();
    }
}