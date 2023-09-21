/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

// Ignore Spelling: Checkin

using FFNodes.Core.Model;
using FFNodes.Server.Handlers;
using Microsoft.AspNetCore.Mvc;
using System.Text.RegularExpressions;

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
        Response.Headers.Add("Content-Disposition", $"attachment; filename=\"error.json\"");
        return BadRequest(new { error = "Server has not finished loading files!" });
    }

    [HttpPost("checkin")]
    [Produces("application/json")]
    [DisableRequestSizeLimit]
    public async Task<IActionResult> CheckinFiles()
    {
        try
        {
            if (FileSystemHandler.Instance.FinishedLoading)
            {
                string? contentDisposition = Request.Headers["Content-Disposition"].ToString();

                if (!string.IsNullOrWhiteSpace(contentDisposition))
                {
                    string fileName = Regex.Match(contentDisposition, "filename=\"(.+?)\"").Groups[1].Value;
                    if (!string.IsNullOrWhiteSpace(fileName))
                    {
                        if (FileSystemHandler.Instance.TryParseFile(fileName, out ProcessedFile? processedFile) && processedFile.HasValue)
                        {
                            string? directory = Path.GetDirectoryName(processedFile.Value.Path);
                            if (!string.IsNullOrWhiteSpace(directory))
                            {
#if DEBUG
                                string path = Path.Combine(directory, $"{Path.GetFileNameWithoutExtension(fileName)}_tmp{Path.GetExtension(fileName)}");
#else
                                string path = Path.Combine(directory, fileName);
                                System.IO.File.Delete(processedFile.Value.Path);
#endif
                                using (FileStream fs = System.IO.File.OpenWrite(path))
                                {
                                    await Request.Body.CopyToAsync(fs);
                                }
                                TimeSpan duration = TimeSpan.Zero;

                                // Check for X-Duration header.
                                if (Request.Headers.ContainsKey("X-Duration"))
                                {
                                    string durationString = Request.Headers["X-Duration"].ToString();
                                    if (!string.IsNullOrWhiteSpace(durationString) && long.TryParse(durationString, out long ticks) && ticks > 0)
                                    {
                                        duration = TimeSpan.FromTicks(ticks);
                                    }
                                }
                                await FileSystemHandler.Instance.ReportProcessedFile(connectedUser, processedFile.Value, path, duration);
                                return Ok(processedFile.Value);
                            }
                            return BadRequest(new { error = "Parent directory of file could not be parsed.", filename = fileName });
                        }
                        return BadRequest(new { error = "Could not find file with filename.", filename = fileName });
                    }
                    return BadRequest(new { error = "Could parse the filename from the Content Disposition header", ContentDisposition = contentDisposition });
                }
                return BadRequest(new { error = "File name not found in the Content Disposition header" });
            }
            return BadRequest(new { error = "Server has not finished loading files!" });
        }
        catch (Exception e) { return BadRequest(new { error = e.Message }); }
    }

    [HttpGet("processed")]
    [Produces("application/json")]
    public IActionResult GetProcessedFiles([FromQuery] Guid fileId)
    {
        if (FileSystemHandler.Instance.FinishedLoading)
        {
            if (UserHandler.Instance.UsersDatabaseFile.Exists(fileId))
            {
                ProcessedFile? file = FileSystemHandler.Instance.LoadReportedFile(fileId);
                if (file != null && file.HasValue)
                {
                    return Ok(file.Value);
                }
                return BadRequest(new { error = "Failed to load processed file, see server logs for more information." });
            }
            return BadRequest(new { error = "No processed file exists!", fileId });
        }
        return BadRequest(new { error = "Server has not finished loading files!" });
    }

    [HttpGet("rescan")]
    public async Task<IActionResult> Rescan()
    {
        if (connectedUser.IsAdmin)
        {
            await FileSystemHandler.Instance.Load();
            return Ok();
        }
        return Unauthorized(new { error = "User is not authorized" });
    }
}