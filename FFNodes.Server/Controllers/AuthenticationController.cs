/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;
using FFNodes.Server.Handlers;
using Microsoft.AspNetCore.Mvc;
using static FFNodes.Server.Data.Data;

namespace FFNodes.Server.Controllers;

[Route("/api/auth")]
[ApiController]
public class AuthenticationController : ControllerBase
{
    [HttpPost("connect")]
    public IActionResult HandleConnectionRequest([FromHeader] string code)
    {
        return ValidConnection(code) ? Ok() : Unauthorized();
    }

    [HttpGet("user")]
    [Produces("application/json")]
    public IActionResult GetUser([FromQuery] Guid id, [FromHeader] string code) => ValidConnection(code) ? Ok(UserHandler.Instance.GetUser(id)) : Unauthorized(new { error = "Invalid or missing connection code." });

    public IActionResult CreateUser([FromHeader] string code, [FromBody] User user) => !ValidConnection(code) ? Unauthorized(new { error = "Invalid or missing connection code." }) : UserHandler.Instance.CreateUser(user) ? Ok(user) : BadRequest(new { error = "User already exists!" });
}