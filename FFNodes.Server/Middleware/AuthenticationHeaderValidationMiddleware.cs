/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;
using FFNodes.Server.Handlers;
using Microsoft.Extensions.Primitives;
using Newtonsoft.Json;

namespace FFNodes.Server.Middleware;

public class AuthenticationHeaderValidationMiddleware
{
    private readonly RequestDelegate _next;

    public AuthenticationHeaderValidationMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public async Task Invoke(HttpContext context)
    {
        context.Items["ConnectedUser"] = null;
        if (context.Request.Path.Value?.StartsWith("/api") ?? false)
        {
            if (ValidateAuthenticationCode(context))
            {
                if (context.Request.Path.Value == "/api/auth/connect" || (context.Request.Path.Value == "/api/auth/user" && context.Request.Method.Equals("POST", StringComparison.OrdinalIgnoreCase)))
                {
                    await _next(context);
                    return;
                }
                if (UserHandler.Instance.GetUserFromHeaders(context, out User user))
                {
                    context.Items["ConnectedUser"] = user;
                    UserHandler.Instance.PingUser(user);
                    await _next(context);
                }
                else
                {
                    context.Response.ContentType = "application/json";
                    context.Response.StatusCode = 401;
                    await context.Response.WriteAsync(JsonConvert.SerializeObject(new { error = "Invalid or missing user id." }));
                }
            }
            else
            {
                context.Response.ContentType = "application/json";
                context.Response.StatusCode = 401;
                await context.Response.WriteAsync(JsonConvert.SerializeObject(new { error = "Invalid or missing connection code." }));
            }
        }
    }

    private static bool ValidateAuthenticationCode(HttpContext context) =>
        context.Request.Headers.TryGetValue("Authentication", out StringValues authentication_code)
        && authentication_code.Any()
        && Data.Data.ValidConnection(authentication_code[0] ?? "");
}