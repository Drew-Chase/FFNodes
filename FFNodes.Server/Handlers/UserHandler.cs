// LFInteractive LLC. 2021-2024﻿
using FFNodes.Core.Model;

namespace FFNodes.Server.Handlers;

public class UserHandler
{
    public static readonly UserHandler Instance = Instance ??= new UserHandler();
    private readonly List<User> users;

    private UserHandler()
    {
        users = new();
    }

    public void Load()
    {
    }
}