/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Server.Data;
using FFNodes.Server.Handlers;
using Microsoft.AspNetCore.Mvc;

namespace FFNodes.Server.Controllers
{
    [Route("/api")]
    [ApiController]
    public class SystemController : ControllerBase
    {
        [HttpGet()]
        [Produces("application/json")]
        public IActionResult GetStatus()
        {
            return Ok(new
            {
                uptime = (DateTime.Now - Configuration.Instance.StartDate),
                loading = !ProcessHandler.Instance.FinishedLoading,
                connected_users = UserHandler.Instance.GetConnectedUsers().Select(user => user.Id),
                connection_url = Data.Data.ConnectionUrl,
            });
        }

        [HttpPost("reset-connection-code")]
        [Produces("application/json")]
        public IActionResult ResetConnectionCode()
        {
            Data.Data.ResetConnectionCode();
            return Ok(new
            {
                connection_url = Data.Data.ConnectionUrl,
            });
        }
    }
}