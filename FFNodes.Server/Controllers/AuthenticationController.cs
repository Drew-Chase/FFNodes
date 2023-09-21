/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;
using FFNodes.Server.Handlers;
using Microsoft.AspNetCore.Mvc;

namespace FFNodes.Server.Controllers;

[Route("/api/auth")]
[ApiController]
public class AuthenticationController : ControllerBase
{
    private readonly User connectedUser;
    private readonly IHttpContextAccessor _httpContextAccessor;

    public AuthenticationController(IHttpContextAccessor httpContextAccessor)
    {
        _httpContextAccessor = httpContextAccessor;
        connectedUser = _httpContextAccessor.HttpContext.Items["ConnectedUser"] as User;
    }

    [HttpGet("connect")]
    public IActionResult HandleConnectionRequest() => Ok();

    [HttpGet("user")]
    [Produces("application/json")]
    public IActionResult GetUser([FromQuery] Guid? id, [FromQuery] string? username) => Ok(id != null ? UserHandler.Instance.GetUser(id.Value) : !string.IsNullOrWhiteSpace(username) ? UserHandler.Instance.GetUser(username) : connectedUser);

    [HttpPost("user")]
    [Produces("application/json")]
    public IActionResult GetOrCreateUser([FromBody] User user) => UserHandler.Instance.CreateUser(user) ? Ok(user) : Ok(UserHandler.Instance.GetUser(user.Username));

    [HttpGet("users")]
    [Produces("application/json")]
    public IActionResult GetUsers() => Ok(UserHandler.Instance.GetUsers());
}