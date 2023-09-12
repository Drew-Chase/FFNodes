/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using CommunityToolkit.Maui;
using FFNodes.Client.Core.Handlers;
using FFNodes.Core;
using FFNodes.Core.Data;
using FFNodes.Server.Data;
using Microsoft.Extensions.Logging;
using Serilog;
using Serilog.Events;

namespace FFNodes.Client.GUI;

public static class MauiProgram
{
    public static MauiApp CreateMauiApp()
    {
        MutexHandler.HandleMutex("FFNodes.Client.GUI");

        TimeSpan flushTime = TimeSpan.FromSeconds(30);
        Log.Logger = new LoggerConfiguration()
            .MinimumLevel.Verbose()
            .WriteTo.Console(LogEventLevel.Verbose)
            .WriteTo.File(Files.DebugLog, LogEventLevel.Verbose, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
            .WriteTo.File(Files.LatestLog, LogEventLevel.Information, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
            .WriteTo.File(Files.ErrorLog, LogEventLevel.Error, buffered: true, rollingInterval: RollingInterval.Day, rollOnFileSizeLimit: true, fileSizeLimitBytes: 5_000_000, flushToDiskInterval: flushTime)
            .CreateLogger();

        AppDomain.CurrentDomain.ProcessExit += (s, e) =>
        {
            Configuration.Instance.Save();
            Log.Information("Stopping FFNodes");
            Log.CloseAndFlush();
        };

        AppDomain.CurrentDomain.UnhandledException += (s, e) =>
        {
            if (e.ExceptionObject is Exception exception)
            {
                CrashHandler.HandleCrash(exception);
            }
            else
            {
                Log.CloseAndFlush();
            }
        };

        CommandLineHandler.HandleCommandLine(Environment.GetCommandLineArgs()[1..]); // get the command line arguments, excluding the first argument which is the executable path.

        MauiAppBuilder builder = MauiApp.CreateBuilder();
        builder.UseMauiApp<App>().UseMauiCommunityToolkit();

        builder.Services.AddMauiBlazorWebView();

#if DEBUG
        builder.Services.AddBlazorWebViewDeveloperTools();
        builder.Logging.AddDebug();
#endif

        Configuration.Instance.Load();

        return builder.Build();
    }
}