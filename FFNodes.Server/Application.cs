/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using Chase.CLIParser;
using FFNodes.Core;
using FFNodes.Core.Data;
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
            // Loads the configuration from the file system
            AppConfig.Instance.Initialize(Files.Config);

            int port = AppConfig.Instance.Port;

            LogEventLevel logEventLevel = AppConfig.Instance.DefaultLogLevel;
            if (Environment.GetCommandLineArgs().Any())
            {
                OptionsManager optionsManager = new("FFNodes");
                optionsManager.Add(new() { ShortName = "p", LongName = "port", Required = false, HasArgument = true, Description = "runs the server on the port" });
                optionsManager.Add(new() { ShortName = "c", LongName = "host", Required = false, HasArgument = true, Description = "runs the server with the specified host" });
                optionsManager.Add(new() { ShortName = "v", LongName = "loglevel", Required = false, HasArgument = true, Description = $"Sets the minimum log level, valid options are {string.Join(", ", ((LogEventLevel[])Enum.GetValues(typeof(LogEventLevel))).Select(i => i.ToString()))}. Default is {logEventLevel}" });

                OptionsParser parser = optionsManager.Parse();
                if (parser != null)
                {
                    if (parser.IsPresent("port", out string portString))
                    {
                        if (!int.TryParse(portString, out port))
                        {
                            Console.ForegroundColor = ConsoleColor.Red;
                            Console.Error.WriteLine($"Invalid port number: {portString}");
                            Console.ResetColor();
                            return;
                        }
                    }

                    if (parser.IsPresent("host", out string host))
                    {
                        AppConfig.Instance.Host = host;
                    }

                    if (parser.IsPresent("v", out string levelString))
                    {
                        if (Enum.TryParse(levelString, true, out LogEventLevel level))
                        {
                            logEventLevel = level;
                        }
                        else
                        {
                            Console.ForegroundColor = ConsoleColor.Red;
                            Console.Error.WriteLine($"Invalid log level: {levelString}");
                            Console.ResetColor();
                            return;
                        }
                    }
                }
            }

            TimeSpan flushTime = TimeSpan.FromSeconds(30);
            Log.Logger = new LoggerConfiguration()
                .MinimumLevel.Verbose()
                .WriteTo.Console(logEventLevel)
                .WriteTo.File(Files.DebugLog, LogEventLevel.Verbose, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
                .WriteTo.File(Files.LatestLog, LogEventLevel.Information, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
                .WriteTo.File(Files.ErrorLog, LogEventLevel.Error, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
                .CreateLogger();

            AppDomain.CurrentDomain.ProcessExit += (s, e) =>
            {
                Log.Information("Stopping Server");
                Log.CloseAndFlush();
            };
            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                if (e.ExceptionObject is Exception exception)
                {
                    CrashHandler.HandleCrash(exception);
                }
            };

            Log.Information("Starting FFNodes Server");

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
                        options.ListenAnyIP(port);
                    });
                    builder.UseStartup<Startup>();

                    Log.Information("Server running at {SERVER}", $"http://{AppConfig.Instance.Host}:{port}");
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