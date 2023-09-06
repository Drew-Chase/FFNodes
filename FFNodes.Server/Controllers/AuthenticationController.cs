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
    [HttpPost("connect")]
    public IActionResult HandleConnectionRequest() => Ok();

    [HttpGet("user")]
    [Produces("application/json")]
    public IActionResult GetUser([FromQuery] Guid id) => Ok(UserHandler.Instance.GetUser(id));

    [HttpPost("user")]
    [Produces("application/json")]
    public IActionResult CreateUser([FromBody] User user) => UserHandler.Instance.CreateUser(user) ? Ok(user) : BadRequest(new { error = "User already exists!" });

    [HttpGet("users")]
    [Produces("application/json")]
    public IActionResult GetUsers() => Ok(UserHandler.Instance.GetUsers());
}