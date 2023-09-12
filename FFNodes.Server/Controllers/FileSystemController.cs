/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

// Ignore Spelling: Checkin

using FFNodes.Core.Model;
using FFNodes.Server.Handlers;
using FFNodes.Server.Model;
using Microsoft.AspNetCore.Mvc;

namespace FFNodes.Server.Controllers;

[Route("/api/fs")]
[ApiController]
public class FileSystemController : ControllerBase
{
    private readonly User connectedUser;
    private readonly IHttpContextAccessor _httpContextAccessor;

    public FileSystemController(IHttpContextAccessor httpContextAccessor)
    {
        _httpContextAccessor = httpContextAccessor;
        connectedUser = _httpContextAccessor.HttpContext.Items["ConnectedUser"] as User;
    }

    [HttpGet("checkout")]
    public IActionResult CheckoutFile()
    {
        if (FileSystemHandler.Instance.FinishedLoading)
        {
            ProcessedFile file = FileSystemHandler.Instance.CheckoutFile(connectedUser);
            Response.Headers.Add("Content-Disposition", $"attachment; filename=\"{Path.GetFileName(file.Path)}\"");
            return new FileStreamResult(System.IO.File.OpenRead(file.Path), "application/octet-stream");
        }
        return BadRequest(new { error = "Server has not finished loading files!" });
    }

    [HttpPost("checkin")]
    [Produces("application/json")]
    [DisableRequestSizeLimit]
    public async Task<IActionResult> CheckinFiles([FromForm] IFormFile file)
    {
        try
        {
            if (FileSystemHandler.Instance.FinishedLoading)
            {
                if (FileSystemHandler.Instance.TryParseFile(file.FileName, out ProcessedFile? processedFile) && processedFile.HasValue)
                {
                    using FileStream fs = System.IO.File.OpenWrite(processedFile.Value.Path + ".tmp");
                    await file.CopyToAsync(fs);
                    await FileSystemHandler.Instance.ReportProcessedFile(connectedUser, processedFile.Value);
                    return Ok(processedFile.Value);
                }
                return BadRequest(new { error = "Could not find file with filename.", filename = file.FileName });
            }
            return BadRequest(new { error = "Server has not finished loading files!" });
        }
        catch (Exception e) { return BadRequest(new { error = e.Message }); }
    }

    [HttpGet("processed")]
    [Produces("application/json")]
    public IActionResult GetProcessedFiles([FromQuery] Guid? userId, [FromQuery] Guid fileId)
    {
        if (FileSystemHandler.Instance.FinishedLoading)
        {
            if (System.IO.File.Exists(FileSystemHandler.GetFileReportPath(userId ?? connectedUser.Id, fileId)))
            {
                ProcessedFile? file = FileSystemHandler.Instance.LoadReportedFile(userId ?? connectedUser.Id, fileId);
                if (file != null && file.HasValue)
                {
                    return Ok(file.Value);
                }
                else
                {
                    return BadRequest(new { error = "Failed to load processed file, see server logs for more information." });
                }
            }
            else
            {
                return BadRequest(new { error = "No processed file exists for user specified!", userId = userId ?? connectedUser.Id, fileId });
            }
        }
        return BadRequest(new { error = "Server has not finished loading files!" });
    }

    [HttpGet("rescan")]
    public IActionResult Rescan()
    {
        return Ok();
    }
}