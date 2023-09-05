// LFInteractive LLC. 2021-2024﻿
using FFNodes.Server.Data;
using Serilog;
using Serilog.Events;

namespace FFNodes.Server
{
    public static class Application
    {
        private static void Main()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.Console(LogEventLevel.Verbose)
                .WriteTo.File(Files.DebugLog, LogEventLevel.Verbose, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000)
                .WriteTo.File(Files.LatestLog, LogEventLevel.Information, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000)
                .CreateLogger();

            AppDomain.CurrentDomain.ProcessExit += (s, e) =>
            {
                Log.Information("Stopping Server");
                Log.CloseAndFlush();
            };

            Log.Information("Starting FFNodes Server");

            Configuration.Instance.Load();

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
                    Log.Information("Connection url is {CONNECTION}", $"ffn://127.0.0.1:{Configuration.Instance.Port}/{Data.Data.ConnectionString}");
                }).Build().Run();
        }
    }

    internal class Startup
    {
        public void Configure(IApplicationBuilder app, IWebHostEnvironment evn)
        {
            app.UseForwardedHeaders();
            app.UseMvc();
            app.UseRouting();
            app.UseStaticFiles();
            app.UseDefaultFiles();
            app.UseSerilogRequestLogging();
        }

        public void ConfigureServices(IServiceCollection service)
        {
            service.AddMvc(action =>
            {
                action.EnableEndpointRouting = false;
            });
        }
    }
}