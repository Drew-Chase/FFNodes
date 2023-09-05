// LFInteractive LLC. 2021-2024﻿
using FFNodes.Client.Services;
using FFNodes.Client.TaskbarNotification.Interop;

namespace FFNodes.Client.Platforms.Windows;

public class TrayService : ITrayService
{
    private WindowsTrayIcon tray;

    public Action ClickHandler { get; set; }

    public void Initialize()
    {
        tray = new WindowsTrayIcon("Platforms/Windows/icon.svg")
        {
            LeftClick = () =>
            {
                WindowExtensions.BringToFront();
                ClickHandler?.Invoke();
            }
        };
    }
}