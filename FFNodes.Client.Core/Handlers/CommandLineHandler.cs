/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CLIParser;
using FFNodes.Server.Data;

namespace FFNodes.Client.Core.Handlers;

public class CommandLineHandler
{
    public static void HandleCommandLine(string[] args)
    {
        OptionsManager manager = new("FFNodes");
        manager.Add(new() { ShortName = "c", LongName = "connection", HasArgument = true, Required = false, Description = "Sets the connection url" });
        manager.Add(new() { ShortName = "u", LongName = "user", HasArgument = true, Required = false, Description = "Sets the connected users id" });
        OptionsParser parser = manager.Parse(args);

        if (parser != null)
        {
            if (parser.IsPresent("c", out string connectionUrl))
            {
                Configuration.Instance.ConnectionUrl = connectionUrl;
                Configuration.Instance.Save();
            }
            if (parser.IsPresent("u", out string userString) && Guid.TryParse(userString, out Guid userId))
            {
                Configuration.Instance.UserId = userId;
                Configuration.Instance.Save();
            }
        }
    }
}