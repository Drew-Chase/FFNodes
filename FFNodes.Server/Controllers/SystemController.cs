/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;
using FFNodes.Server.Data;
using FFNodes.Server.Handlers;
using Microsoft.AspNetCore.Mvc;

namespace FFNodes.Server.Controllers
{
    [Route("/api")]
    [ApiController]
    public class SystemController : ControllerBase
    {
        private readonly User connectedUser;
        private readonly IHttpContextAccessor _httpContextAccessor;

        public SystemController(IHttpContextAccessor httpContextAccessor)
        {
            _httpContextAccessor = httpContextAccessor;
            connectedUser = _httpContextAccessor.HttpContext.Items["ConnectedUser"] as User;
        }

        [HttpGet()]
        [Produces("application/json")]
        public IActionResult GetStatus() => Ok(new SystemStatusModel(DateTime.Now - Configuration.Instance.StartDate, !FileSystemHandler.Instance.FinishedLoading, UserHandler.Instance.GetConnectedUsers(), Data.Data.ConnectionUrl));

        [HttpPost("reset-connection-code")]
        [Produces("application/json")]
        public IActionResult ResetConnectionCode()
        {
            if (connectedUser.IsAdmin)
            {
                Data.Data.ResetConnectionCode();
                return Ok(new
                {
                    connection_url = Data.Data.ConnectionUrl,
                });
            }
            return BadRequest(new { error = "You do not have valid permissions." });
        }
    }
}