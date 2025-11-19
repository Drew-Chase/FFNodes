use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::join;
use tokio::process::Command;
use tokio_interactive::AsynchronousInteractiveProcess;

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FFMpeg {
    #[serde(rename = "ffmpeg")]
    pub ffmpeg_path: PathBuf,
    #[serde(rename = "ffprobe")]
    pub ffprobe_path: PathBuf,
}

impl FFMpeg {
    pub fn new(ffmpeg_path: impl Into<PathBuf>, ffprobe_path: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            ffprobe_path: ffprobe_path.into(),
        }
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
        let output = {
            #[cfg(target_os = "windows")]
            {
                Command::new("where")
            }
            #[cfg(not(target_os = "windows"))]
            {
                Command::new("which")
            }
        }
        .arg("ffmpeg")
        .output()
        .await?;

        let output = String::from_utf8(output.stdout)?;
        let output = output.trim();
        match tokio::fs::try_exists(&output).await {
            Ok(true) => Ok(Some(PathBuf::from(output))),
            _ => Ok(None),
        }
    }
    pub async fn find_ffprobe() -> Result<Option<PathBuf>> {
        let output = {
            #[cfg(target_os = "windows")]
            {
                Command::new("where")
            }
            #[cfg(not(target_os = "windows"))]
            {
                Command::new("which")
            }
        }
        .arg("ffprobe")
        .output()
        .await?;

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

        // Extract the zip file
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Find and extract the binary (it should be the only file or the main executable)
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.name());

            if file.is_dir() {
                tokio::fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(parent) = outpath.parent() {
                    tokio::fs::create_dir_all(parent).await?;
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
        self.exec(true, args, working_dir, sender).await
    }

    pub async fn exec_ffprobe(
        &self,
        args: &[&str],
        working_dir: impl Into<PathBuf>,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> Result<()> {
        self.exec(false, args, working_dir, sender).await
    }

    pub async fn exec(
        &self,
        ffmpeg: bool,
        args: &[&str],
        working_dir: impl Into<PathBuf>,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> Result<()> {
        let pid = AsynchronousInteractiveProcess::new(
            if ffmpeg {
                &self.ffmpeg_path
            } else {
                &self.ffprobe_path
            }
            .to_string_lossy()
            .to_string(),
        )
        .with_arguments(args.to_vec())
        .with_working_directory(working_dir)
        .start()
        .await?;
        if let Some(mut process) = AsynchronousInteractiveProcess::get_process_by_pid(pid).await {
            tokio::spawn(async move {
                while process.is_process_running().await {
                    match process.receive_output().await {
                        Ok(Some(output)) => sender.send(output).await?,
                        Err(e) => return Err(e),
                        _ => continue,
                    };
                }
                Ok(())
            });
        }
        Ok(())
    }
}
