// LFInteractive LLC. 2021-2024﻿
using FFNodes.Client.Services;
using Microsoft.Extensions.Logging;
using Microsoft.Maui.LifecycleEvents;

namespace FFNodes.Client;

public static class MauiProgram
{
    public static MauiApp CreateMauiApp()
    {
        var builder = MauiApp.CreateBuilder();
        builder.UseMauiApp<App>();
        builder.Services.AddMauiBlazorWebView();
        builder.ConfigureLifecycleEvents(lifecycle =>
        {
#if WINDOWS
            lifecycle.AddWindows(windows =>
                windows.OnPlatformMessage((app, args) =>
                {
                    try
                    {
                        if (Platforms.Windows.WindowExtensions.Hwnd == IntPtr.Zero)
                        {
                            Platforms.Windows.WindowExtensions.Hwnd = args.Hwnd;
                            Platforms.Windows.WindowExtensions.SetIcon("Platforms/Windows/icon.svg");
                        }
                        app.ExtendsContentIntoTitleBar = false;
                    }
                    catch { }
                }));
#endif
        });
#if DEBUG
        builder.Services.AddBlazorWebViewDeveloperTools();
        builder.Logging.AddDebug();
#endif

#if WINDOWS
        builder.Services.AddSingleton<ITrayService, Platforms.Windows.TrayService>();
#endif
        return builder.Build();
    }
}