// LFInteractive LLC. 2021-2024﻿
namespace FFNodes.Client.Services;

public interface ITrayService
{
    Action ClickHandler { get; set; }

    void Initialize();
}