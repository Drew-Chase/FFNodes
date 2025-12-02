pub mod builders;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use log::trace;
use tokio::join;
use tokio::process::Command;
use tokio::io::BufReader;
use tokio::sync::Mutex;
use crate::builders::{FFmpegBuilder, FFprobeBuilder};

const FFBINARIES_API: &str = "https://ffbinaries.com/api/v1/version/latest";
const META_DIR: &str = "meta/ffmpeg";

#[derive(Debug, Deserialize)]
struct FFBinariesResponse {
	#[allow(dead_code)]
	version: String,
	bin: HashMap<String, PlatformBinaries>,
}

#[derive(Debug, Deserialize)]
struct PlatformBinaries {
	ffmpeg: String,
	ffprobe: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FFMpeg {
	#[serde(rename = "ffmpeg")]
	ffmpeg_path: PathBuf,
	#[serde(rename = "ffprobe")]
	ffprobe_path: PathBuf,
}

impl FFMpeg {
	pub fn new(ffmpeg_path: impl Into<PathBuf>, ffprobe_path: impl Into<PathBuf>) -> Self {
		Self {
			ffmpeg_path: ffmpeg_path.into(),
			ffprobe_path: ffprobe_path.into(),
		}
	}

	pub fn ffmpeg_command_builder(&self)->FFmpegBuilder{
        FFmpegBuilder::new(std::sync::Arc::new(self.clone()))
    }
	pub fn ffprobe_command_builder(&self)->FFprobeBuilder{
        FFprobeBuilder::new(std::sync::Arc::new(self.clone()))
    }

	/// Get the path to the ffmpeg binary
	pub fn ffmpeg_path(&self) -> &PathBuf {
		&self.ffmpeg_path
	}

	/// Get the path to the ffprobe binary
	pub fn ffprobe_path(&self) -> &PathBuf {
		&self.ffprobe_path
	}

	pub async fn fetch(&mut self) -> Result<()> {
		// Step 1: Check for local binaries in meta/ffmpeg directory
		if let Some((ffmpeg_path, ffprobe_path)) = Self::check_local_binaries().await {
			self.ffmpeg_path = ffmpeg_path;
			self.ffprobe_path = ffprobe_path;
			return Ok(());
		}

		// Step 2: Try to find binaries in system PATH
		let (ffmpeg, ffprobe) = join!(Self::find_ffmpeg(), Self::find_ffprobe(),);

		match (ffmpeg?, ffprobe?) {
			(Some(ffmpeg_path), Some(ffprobe_path)) => {
				self.ffmpeg_path = ffmpeg_path;
				self.ffprobe_path = ffprobe_path;
				Ok(())
			}
			// Step 3: If not found in PATH, download them
			_ => {
				let (ffmpeg_path, ffprobe_path) = Self::download_binaries().await?;
				self.ffmpeg_path = ffmpeg_path;
				self.ffprobe_path = ffprobe_path;
				Ok(())
			}
		}
	}

	pub async fn find_ffmpeg() -> Result<Option<PathBuf>> {
		let mut cmd = {
			#[cfg(target_os = "windows")]
			{
				Command::new("where")
			}
			#[cfg(not(target_os = "windows"))]
			{
				Command::new("which")
			}
		};
		cmd.arg("ffmpeg");

		// On Windows, prevent creating a new console window
		#[cfg(target_os = "windows")]
		{
			cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
		}

		let output = cmd.output().await?;

		let output = String::from_utf8(output.stdout)?;
		let output = output.trim();
		match tokio::fs::try_exists(&output).await {
			Ok(true) => Ok(Some(PathBuf::from(output))),
			_ => Ok(None),
		}
	}
	pub async fn find_ffprobe() -> Result<Option<PathBuf>> {
		let mut cmd = {
			#[cfg(target_os = "windows")]
			{
				Command::new("where")
			}
			#[cfg(not(target_os = "windows"))]
			{
				Command::new("which")
			}
		};
		cmd.arg("ffprobe");

		// On Windows, prevent creating a new console window
		#[cfg(target_os = "windows")]
		{
			cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
		}

		let output = cmd.output().await?;

		let output: String = String::from_utf8(output.stdout)?;
		let output = output.trim();
		match tokio::fs::try_exists(&output).await {
			Ok(true) => Ok(Some(PathBuf::from(output))),
			_ => Ok(None),
		}
	}

	/// Detects the current platform and returns the corresponding platform string for the API
	fn get_platform_string() -> Result<String> {
		#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
		return Ok("windows-64".to_string());

		#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
		return Ok("windows-32".to_string());

		#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
		return Ok("linux-64".to_string());

		#[cfg(all(target_os = "linux", target_arch = "x86"))]
		return Ok("linux-32".to_string());

		#[cfg(all(target_os = "linux", target_arch = "arm"))]
		return Ok("linux-armhf".to_string());

		#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
		return Ok("linux-arm-64".to_string());

		#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
		return Ok("osx-64".to_string());

		#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
		return Ok("osx-arm-64".to_string());

		#[cfg(not(any(
			all(target_os = "windows", target_pointer_width = "64"),
			all(target_os = "windows", target_pointer_width = "32"),
			all(target_os = "linux", target_arch = "x86_64"),
			all(target_os = "linux", target_arch = "x86"),
			all(target_os = "linux", target_arch = "arm"),
			all(target_os = "linux", target_arch = "aarch64"),
			all(target_os = "macos", target_arch = "x86_64"),
			all(target_os = "macos", target_arch = "aarch64")
		)))]
		return Err(anyhow::anyhow!("Unsupported platform"));
	}

	/// Downloads and extracts a binary from a URL to the specified destination
	async fn download_and_extract(
		url: &str,
		dest_dir: &PathBuf,
		binary_name: &str,
	) -> Result<PathBuf> {
		// Create destination directory if it doesn't exist
		tokio::fs::create_dir_all(dest_dir).await?;

		// Download the zip file
		let response = reqwest::get(url).await?;
		let bytes = response.bytes().await?;

		// Create a temporary file path for the zip
		let zip_path = dest_dir.join(format!("{}.zip", binary_name));
		tokio::fs::write(&zip_path, &bytes).await?;

		// Extract the zip file in a blocking task to avoid Send issues
		let dest_dir_clone = dest_dir.clone();
		let zip_path_clone = zip_path.clone();
		tokio::task::spawn_blocking(move || -> Result<()> {
			let file = std::fs::File::open(&zip_path_clone)?;
			let mut archive = zip::ZipArchive::new(file)?;

			// Find and extract the binary (it should be the only file or the main executable)
			for i in 0..archive.len() {
				let mut file = archive.by_index(i)?;
				let outpath = dest_dir_clone.join(file.name());

				if file.is_dir() {
					std::fs::create_dir_all(&outpath)?;
				} else {
					if let Some(parent) = outpath.parent() {
						std::fs::create_dir_all(parent)?;
					}
					let mut outfile = std::fs::File::create(&outpath)?;
					std::io::copy(&mut file, &mut outfile)?;

					// Make executable on Unix systems
					#[cfg(unix)]
					{
						use std::os::unix::fs::PermissionsExt;
						let mut permissions = outfile.metadata()?.permissions();
						permissions.set_mode(0o755);
						std::fs::set_permissions(&outpath, permissions)?;
					}
				}
			}
			Ok(())
		})
		.await??;

		// Clean up zip file
		tokio::fs::remove_file(&zip_path).await?;

		// Return the path to the extracted binary
		#[cfg(target_os = "windows")]
		let binary_path = dest_dir.join(format!("{}.exe", binary_name));
		#[cfg(not(target_os = "windows"))]
		let binary_path = dest_dir.join(binary_name);

		Ok(binary_path)
	}

	/// Downloads both ffmpeg and ffprobe binaries if they don't exist locally
	async fn download_binaries() -> Result<(PathBuf, PathBuf)> {
		let platform = Self::get_platform_string()?;

		// Fetch the API to get download URLs
		let response = reqwest::get(FFBINARIES_API).await?;
		let data: FFBinariesResponse = response.json().await?;

		// Get the URLs for the current platform
		let platform_bins = data
			.bin
			.get(&platform)
			.ok_or_else(|| anyhow::anyhow!("Platform {} not supported by ffbinaries", platform))?;

		let dest_dir = PathBuf::from(META_DIR);

		// Download both binaries in parallel
		let (ffmpeg_path, ffprobe_path) = join!(
            Self::download_and_extract(&platform_bins.ffmpeg, &dest_dir, "ffmpeg"),
            Self::download_and_extract(&platform_bins.ffprobe, &dest_dir, "ffprobe"),
        );

		Ok((ffmpeg_path?, ffprobe_path?))
	}

	/// Checks if binaries exist in the meta directory
	async fn check_local_binaries() -> Option<(PathBuf, PathBuf)> {
		#[cfg(target_os = "windows")]
		let (ffmpeg_name, ffprobe_name) = ("ffmpeg.exe", "ffprobe.exe");
		#[cfg(not(target_os = "windows"))]
		let (ffmpeg_name, ffprobe_name) = ("ffmpeg", "ffprobe");

		let ffmpeg_path = PathBuf::from(META_DIR).join(ffmpeg_name);
		let ffprobe_path = PathBuf::from(META_DIR).join(ffprobe_name);

		match (
			tokio::fs::try_exists(&ffmpeg_path).await,
			tokio::fs::try_exists(&ffprobe_path).await,
		) {
			(Ok(true), Ok(true)) => Some((ffmpeg_path, ffprobe_path)),
			_ => None,
		}
	}

	pub async fn exec_ffmpeg(
		&self,
		args: &[&str],
		working_dir: impl Into<PathBuf>,
		sender: tokio::sync::mpsc::Sender<String>,
	) -> Result<()> {
		self.exec(true, args, working_dir, sender, None).await
	}

	pub async fn exec_ffmpeg_with_pid(
		&self,
		args: &[&str],
		working_dir: impl Into<PathBuf>,
		sender: tokio::sync::mpsc::Sender<String>,
		pid_storage: Arc<Mutex<Option<u32>>>,
	) -> Result<()> {
		self.exec(true, args, working_dir, sender, Some(pid_storage)).await
	}

	pub async fn exec_ffprobe(
		&self,
		args: &[&str],
		working_dir: impl Into<PathBuf>,
		sender: tokio::sync::mpsc::Sender<String>,
	) -> Result<()> {
		self.exec(false, args, working_dir, sender, None).await
	}

	pub fn command(&self, ffmpeg: bool,
	               args: &[&str],
	               working_dir: impl Into<PathBuf>,) ->Command{

		let binary_path = if ffmpeg {
			&self.ffmpeg_path
		} else {
			&self.ffprobe_path
		};

		let working_dir: PathBuf = working_dir.into();

		let mut cmd = Command::new(binary_path);
		cmd.args(args)
		   .current_dir(&working_dir)
		   .stdout(std::process::Stdio::piped())
		   .stderr(std::process::Stdio::piped());

		// On Windows, prevent creating a new console window
		#[cfg(target_os = "windows")]
		{
			cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
		}

		log::debug!("Executing: {:?} with args: {:?} in {:?}", binary_path, args, working_dir);
		cmd
	}

	pub async fn exec(
		&self,
		ffmpeg: bool,
		args: &[&str],
		working_dir: impl Into<PathBuf>,
		sender: tokio::sync::mpsc::Sender<String>,
		pid_storage: Option<Arc<Mutex<Option<u32>>>>,
	) -> Result<()> {
		let working_dir: PathBuf = working_dir.into();

		// Build command string for error reporting
		let binary_path = if ffmpeg { &self.ffmpeg_path } else { &self.ffprobe_path };
		let cmd_string = format!("{} {}",
			binary_path.display(),
			args.join(" ")
		);

		let mut cmd = self.command(ffmpeg, args, &working_dir);
		let mut child = cmd.spawn()?;

		// Store the process ID if requested
		if let Some(pid_storage) = pid_storage {
			if let Some(pid) = child.id() {
				let mut lock = pid_storage.lock().await;
				*lock = Some(pid);
				log::debug!("Stored FFmpeg process ID: {}", pid);
			}
		}

		let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
		let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

		// Buffers to collect output for error reporting
		let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
		let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

		let sender_clone = sender.clone();
		let stdout_buf_clone = stdout_buffer.clone();
		let stdout_reader = async move {
			use tokio::io::AsyncReadExt;
			let mut reader = BufReader::new(stdout);

			if ffmpeg {
				// For FFmpeg: Stream output in chunks for real-time progress
				let mut chunk = vec![0u8; 8192]; // 8KB chunks
				let mut line_buffer = String::new();

				loop {
					match reader.read(&mut chunk).await {
						Ok(0) => {
							// EOF - send any remaining incomplete line
							if !line_buffer.is_empty() {
								trace!("ffmpeg stdout: {}", line_buffer.trim());
								let _ = sender_clone.send(line_buffer.clone()).await;
								// Save to buffer for error reporting
								stdout_buf_clone.lock().await.extend_from_slice(line_buffer.as_bytes());
							}
							break;
						}
						Ok(n) => {
							// Save raw bytes to buffer for error reporting
							stdout_buf_clone.lock().await.extend_from_slice(&chunk[..n]);

							// Convert chunk to string and append to line buffer
							let data = String::from_utf8_lossy(&chunk[..n]);
							line_buffer.push_str(&data);

							// Process all complete lines in the buffer
							while let Some(newline_pos) = line_buffer.find('\n') {
								let line = line_buffer[..=newline_pos].to_string();
								line_buffer.drain(..=newline_pos);

								if !line.trim().is_empty() {
									trace!("ffmpeg stdout: {}", line.trim());
									let _ = sender_clone.send(line).await;
								}
							}
						}
						Err(_) => break,
					}
				}
			} else {
				// For FFprobe: Read entire output at once
				let mut buffer = Vec::new();

				// Read entire output into buffer
				if reader.read_to_end(&mut buffer).await.is_ok() && !buffer.is_empty() {
					// Save to buffer for error reporting
					stdout_buf_clone.lock().await.extend_from_slice(&buffer);

					let output = String::from_utf8_lossy(&buffer);
					trace!("ffprobe stdout: {}", output);
					// Send the entire output as one message
					let _ = sender_clone.send(output.to_string()).await;
				}
			}
		};

		let stderr_buf_clone = stderr_buffer.clone();
		let stderr_reader = async move {
			use tokio::io::AsyncReadExt;
			let mut reader = BufReader::new(stderr);

			if ffmpeg {
				// For FFmpeg: Stream output in chunks for real-time progress
				let mut chunk = vec![0u8; 8192]; // 8KB chunks
				let mut line_buffer = String::new();

				loop {
					match reader.read(&mut chunk).await {
						Ok(0) => {
							// EOF - send any remaining incomplete line
							if !line_buffer.is_empty() {
								trace!("ffmpeg stderr: {}", line_buffer.trim());
								let _ = sender.send(line_buffer.clone()).await;
								// Save to buffer for error reporting
								stderr_buf_clone.lock().await.extend_from_slice(line_buffer.as_bytes());
							}
							break;
						}
						Ok(n) => {
							// Save raw bytes to buffer for error reporting
							stderr_buf_clone.lock().await.extend_from_slice(&chunk[..n]);

							// Convert chunk to string and append to line buffer
							let data = String::from_utf8_lossy(&chunk[..n]);
							line_buffer.push_str(&data);

							// Process all complete lines in the buffer
							while let Some(newline_pos) = line_buffer.find('\n') {
								let line = line_buffer[..=newline_pos].to_string();
								line_buffer.drain(..=newline_pos);

								if !line.trim().is_empty() {
									trace!("ffmpeg stderr: {}", line.trim());
									let _ = sender.send(line).await;
								}
							}
						}
						Err(_) => break,
					}
				}
			} else {
				// For FFprobe: Read entire output at once
				let mut buffer = Vec::new();

				// Read entire output into buffer
				if reader.read_to_end(&mut buffer).await.is_ok() && !buffer.is_empty() {
					// Save to buffer for error reporting
					stderr_buf_clone.lock().await.extend_from_slice(&buffer);

					let output = String::from_utf8_lossy(&buffer);
					trace!("ffprobe stderr: {}", output);
					// Send the entire output as one message
					let _ = sender.send(output.to_string()).await;
				}
			}
		};

		let (status, _, _) = join!(child.wait(), stdout_reader, stderr_reader);
		let status = status?;

		// CRITICAL FIX: Check exit status
		if !status.success() {
			let code = status.code().unwrap_or(-1);

			// Collect captured output for error reporting
			let stdout_bytes = stdout_buffer.lock().await;
			let stderr_bytes = stderr_buffer.lock().await;

			let stdout_str = String::from_utf8_lossy(&stdout_bytes);
			let stderr_str = String::from_utf8_lossy(&stderr_bytes);

			// Limit output to last 2000 chars to avoid excessive logs
			let stdout_display = if stdout_str.len() > 2000 {
				format!("...{}", &stdout_str[stdout_str.len() - 2000..])
			} else {
				stdout_str.to_string()
			};

			let stderr_display = if stderr_str.len() > 2000 {
				format!("...{}", &stderr_str[stderr_str.len() - 2000..])
			} else {
				stderr_str.to_string()
			};

			// Log the full error details
			log::error!("FFmpeg crashed with exit code: {}", code);
			log::error!("Command: {}", cmd_string);
			if !stdout_display.is_empty() {
				log::error!("=== STDOUT ===\n{}", stdout_display);
			}
			if !stderr_display.is_empty() {
				log::error!("=== STDERR ===\n{}", stderr_display);
			}

			// Create detailed error message
			let mut error_msg = format!("Process exited with code: {}\nCommand: {}", code, cmd_string);
			if !stderr_display.is_empty() {
				error_msg.push_str(&format!("\n\nSTDERR:\n{}", stderr_display));
			}
			if !stdout_display.is_empty() {
				error_msg.push_str(&format!("\n\nSTDOUT:\n{}", stdout_display));
			}

			return Err(anyhow::anyhow!("{}", error_msg));
		}

		Ok(())
	}
}
