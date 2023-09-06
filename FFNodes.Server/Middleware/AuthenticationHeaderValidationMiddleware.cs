/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

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
        if (context.Request.Headers.TryGetValue("code", out StringValues values) && values.Any() && Data.Data.ValidConnection(values[0] ?? ""))
        {
            await _next(context);
        }
        else
        {
            context.Response.ContentType = "application/json";
            context.Response.StatusCode = 401;
            await context.Response.WriteAsync(JsonConvert.SerializeObject(new { error = "Invalid or missing connection code." }));
            return;
        }
    }
}