// LFInteractive LLC. 2021-2024﻿
namespace FFNodes.Client;

public partial class App : Application
{
    public App()
    {
        InitializeComponent();

        MainPage = new MainPage();
    }

    protected override Window CreateWindow(IActivationState activationState)
    {
        Window window = base.CreateWindow(activationState);

        window.Title = "FFNodes";
        window.MinimumWidth = 1280;
        window.MinimumHeight = 720;

        return window;
    }
}