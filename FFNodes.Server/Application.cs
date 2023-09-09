/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CLIParser;
using FFNodes.Server.Data;
using FFNodes.Server.Handlers;
using FFNodes.Server.Middleware;
using Microsoft.AspNetCore.Http.Features;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Serilog;
using Serilog.Events;

namespace FFNodes.Server
{
    public static class Application
    {
        private static void Main()
        {
            if (Environment.GetCommandLineArgs().Any())
            {
                OptionsManager optionsManager = new("FFNodes");
                optionsManager.Add(new() { ShortName = "p", LongName = "port", Required = false, HasArgument = true, Description = "runs the server on and sets the port" });
                optionsManager.Add(new() { ShortName = "c", LongName = "host", Required = false, HasArgument = true, Description = "runs the server on and sets the host" });

                OptionsParser parser = optionsManager.Parse();
                if (parser != null)
                {
                    if (parser.IsPresent("port", out string portString))
                    {
                        if (int.TryParse(portString, out int port))
                        {
                            Configuration.Instance.Port = port;
                            Configuration.Instance.Save();
                        }
                        else
                        {
                            Console.ForegroundColor = ConsoleColor.Red;
                            Console.Error.WriteLine($"Invalid port number: {portString}");
                            Console.ResetColor();
                            return;
                        }
                    }

                    if (parser.IsPresent("host", out string host))
                    {
                        Configuration.Instance.Host = host;
                        Configuration.Instance.Save();
                    }
                }
            }

            TimeSpan flushTime = TimeSpan.FromSeconds(30);
            Log.Logger = new LoggerConfiguration()
                .MinimumLevel.Verbose()
                .WriteTo.Console(LogEventLevel.Verbose)
                .WriteTo.File(Files.DebugLog, LogEventLevel.Verbose, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
                .WriteTo.File(Files.LatestLog, LogEventLevel.Information, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
                .CreateLogger();

            AppDomain.CurrentDomain.ProcessExit += (s, e) =>
            {
                Log.Information("Stopping Server");
                Log.CloseAndFlush();
            };

            Log.Information("Starting FFNodes Server");

            // Loads the configuration from the file system
            Configuration.Instance.Load();

            // Loads all users from the file system
            UserHandler.Instance.Load();

            // Creates a new task to load the process handler
            Task.Run(FileSystemHandler.Instance.Load);

            Host.CreateDefaultBuilder()
                .UseSerilog()
                .ConfigureWebHostDefaults(builder =>
                {
                    builder.UseIISIntegration();
                    builder.UseContentRoot(Directory.GetCurrentDirectory());
                    builder.UseKestrel(options =>
                    {
                        options.ListenAnyIP(Configuration.Instance.Port);
                    });
                    builder.UseStartup<Startup>();

                    Log.Information("Server running at {SERVER}", $"http://127.0.0.1:{Configuration.Instance.Port}");
                    Log.Information("Connection url is {CONNECTION}", Data.Data.ConnectionUrl);
                }).Build().Run();
        }
    }

    internal class Startup
    {
        public void Configure(IApplicationBuilder app, IWebHostEnvironment evn)
        {
            app.UseMiddleware<AuthenticationHeaderValidationMiddleware>();
            app.UseForwardedHeaders();
            app.UseMvc();
            app.UseRouting();
            app.UseStaticFiles();
            app.UseDefaultFiles();
            app.UseSerilogRequestLogging();
        }

        public void ConfigureServices(IServiceCollection services)
        {
            services.Configure<FormOptions>(options =>
            {
                options.MultipartBodyLengthLimit = long.MaxValue;
            });

            services.AddSingleton<IHttpContextAccessor, HttpContextAccessor>();

            services.Configure<KestrelServerOptions>(options =>
            {
                options.Limits.MaxConcurrentUpgradedConnections = null;
                options.Limits.MaxConcurrentConnections = null;
                options.Limits.MaxRequestBodySize = null;
            });
            services.AddMvc(action =>
            {
                action.EnableEndpointRouting = false;
            });
        }
    }
}