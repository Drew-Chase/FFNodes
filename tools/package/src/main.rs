use anyhow::{Result, anyhow};
use clap::Parser;
use log::{debug, error, info};
use regex::Regex;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::process::{Command, exit};
use tokio::join;
use which::Path;

#[derive(clap::Parser, Debug)]
pub struct PackageCommandlineArgs {
    #[arg(
        long = "bin",
        alias = "package",
        help = "The target package to build for",
        default_value = "both"
    )]
    pub bin: PackageTarget,
    #[arg(
        long = "no-clean",
        help = "This will not delete the dist directory before building."
    )]
    pub no_clean: bool,
    #[arg(
        long = "wsl",
        help = "This will build for wsl (this will still build for the current directory)."
    )]
    pub wsl: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Default)]
pub enum PackageTarget {
    #[default]
    Both,
    Server,
    Client,
}

impl Display for PackageTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageTarget::Both => write!(f, "Both"),
            PackageTarget::Server => write!(f, "Server"),
            PackageTarget::Client => write!(f, "Client"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Debug)
        .init();
    if !is_pnpm_installed() {
        error!(
            "pnpm is not installed, please install pnpm with `npm install -g pnpm` or visit the official pnpm website for installation instructions: https://pnpm.io/."
        );
        exit(1);
    }
    // create output
    let args = PackageCommandlineArgs::parse();
    let output_directory = "./dist";
    if !args.no_clean {
        let _ = fs::remove_dir_all(output_directory);
    }
    fs::create_dir_all(output_directory)?;

    match args.bin {
        PackageTarget::Client => build_client().await?,
        PackageTarget::Server => build_server().await?,
        PackageTarget::Both => {
            let result = join!(tokio::spawn(build_client()), tokio::spawn(build_server()));
            if let Err(e) = result.1 {
                error!("Failed to build client: {}", e);
            }
            if let Err(e) = result.0 {
                error!("Failed to build server: {}", e);
            }
        }
    }

    if args.wsl {
        build_wsl(args.bin).await?;
    }

    Ok(())
}

fn is_pnpm_installed() -> bool {
    debug!("Checking if pnpm is installed");
    which::which("pnpm").is_ok()
}

fn get_platform_arch(target: &Option<String>) -> Result<(String, String)> {
    let (platform, arch) = if let Some(target_str) = target {
        // Parse target triple
        if target_str.contains("windows") {
            ("windows".to_string(), if target_str.contains("aarch64") { "arm64".to_string() } else { "x64".to_string() })
        } else if target_str.contains("linux") {
            ("linux".to_string(), if target_str.contains("aarch64") { "arm64".to_string() } else { "x64".to_string() })
        } else if target_str.contains("darwin") || target_str.contains("apple") {
            ("macos".to_string(), if target_str.contains("aarch64") { "arm64".to_string() } else { "x64".to_string() })
        } else {
            ("unknown".to_string(), "x64".to_string())
        }
    } else {
        // Fall back to compile-time detection
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "unknown"
        }.to_string();

        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x64"
        }.to_string();

        (platform, arch)
    };

    Ok((platform, arch))
}

async fn build_wsl(bin: PackageTarget) -> Result<()> {
    let mut child = Command::new("wsl")
        .arg("bash")
        .arg("-ic")
        .arg(format!("cargo pkg --bin {} --no-clean", bin))
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "build wsl failed with exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

async fn build_client() -> Result<()> {
    info!("Building client");
    let target = std::env::var("CARGO_BUILD_TARGET").ok();

    let mut command = Command::new(which::which("pnpm")?);
    command.arg("--filter").arg("ffnodes_client");

    if let Some(target_arch) = &target {
        command.arg("tauri").arg("build").arg("--target").arg(target_arch);
    } else {
        command.arg("tauri-build");
    }

    let mut child = command.spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "build client failed with exit code {:?}",
            status.code()
        ));
    }
    debug!("Build client succeeded ✔!");
    info!("Packaging client");
    let toml_version_regex = Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#)?;
    let cargo = fs::read_to_string("./ffnodes-client/src-tauri/Cargo.toml")?;
    let version = toml_version_regex
        .find(&cargo)
        .ok_or_else(|| anyhow!("Cargo.toml not found in Cargo.toml file!"))?
        .as_str()
        .trim_start_matches("version = \"")
        .trim_end_matches("\"");
    let version = semver::Version::parse(version)?;

    // Determine platform and architecture
    let (platform, arch) = get_platform_arch(&target)?;
    let output_file = format!("dist/ffnodes-client-{}-{}-{}.zip", version, platform, arch);

    let mut writer = zip::ZipWriter::new(fs::File::create(output_file)?);

    // Determine artifact path based on target
    let base_path = if let Some(t) = &target {
        format!("target/{}/release", t)
    } else {
        "target/release".to_string()
    };

    #[cfg(target_os = "windows")]
    let artifact_file = Path::new(format!("{}/ffnodes.exe", base_path))?;
    #[cfg(not(target_os = "windows"))]
    let artifact_file = Path::new(format!("{}/ffnodes", base_path))?;

    let filename = artifact_file.file_name().unwrap().to_str().unwrap();
    writer.start_file(filename, zip::write::SimpleFileOptions::default())?;
    let bytes = fs::read(artifact_file)?;
    writer.write_all(&bytes)?;
    writer.finish()?;

    // Copy platform-specific installers
    let arch_str = if arch == "arm64" { "arm64" } else { "x64" };
    let bundle_base = if let Some(t) = &target {
        format!("target/{}/release/bundle", t)
    } else {
        "target/release/bundle".to_string()
    };

    #[cfg(target_os = "windows")]
    {
        // Copy MSI installer
        let msi_src = format!(
            "{}/msi/ffnodes_{}.{}.{}_{}_en-US.msi",
            bundle_base, version.major, version.minor, version.patch, arch_str
        );
        let msi_dst = format!("dist/ffnodes_client_{}_windows-{}_en-US.msi", version, arch_str);
        if std::path::Path::new(&msi_src).exists() {
            fs::copy(&msi_src, &msi_dst)?;
        }

        // Copy NSIS installer
        let nsis_src = format!(
            "{}/nsis/ffnodes_{}.{}.{}_{}-setup.exe",
            bundle_base, version.major, version.minor, version.patch, arch_str
        );
        let nsis_dst = format!("dist/ffnodes_client_{}_windows-{}_setup.exe", version, arch_str);
        if std::path::Path::new(&nsis_src).exists() {
            fs::copy(&nsis_src, &nsis_dst)?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Copy AppImage
        let appimage_src = format!(
            "{}/appimage/ffnodes_{}.{}.{}_{}.AppImage",
            bundle_base, version.major, version.minor, version.patch, arch_str
        );
        let appimage_dst = format!("dist/ffnodes_client_{}_linux-{}.AppImage", version, arch_str);
        if std::path::Path::new(&appimage_src).exists() {
            fs::copy(&appimage_src, &appimage_dst)?;
        }

        // Copy DEB package
        let deb_src = format!(
            "{}/deb/ffnodes_{}.{}.{}_{}.deb",
            bundle_base, version.major, version.minor, version.patch, arch_str
        );
        let deb_dst = format!("dist/ffnodes_client_{}_linux-{}.deb", version, arch_str);
        if std::path::Path::new(&deb_src).exists() {
            fs::copy(&deb_src, &deb_dst)?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Copy DMG
        let dmg_src = format!(
            "{}/dmg/ffnodes_{}.{}.{}_{}.dmg",
            bundle_base, version.major, version.minor, version.patch, arch_str
        );
        let dmg_dst = format!("dist/ffnodes_client_{}_macos-{}.dmg", version, arch_str);
        if std::path::Path::new(&dmg_src).exists() {
            fs::copy(&dmg_src, &dmg_dst)?;
        }

        // Copy APP bundle (as tar.gz for easier distribution)
        let app_src = format!(
            "{}/macos/ffnodes.app",
            bundle_base
        );
        if std::path::Path::new(&app_src).exists() {
            let app_dst = format!("dist/ffnodes_client_{}_macos-{}.app.tar.gz", version, arch_str);
            let mut tar_gz = Command::new("tar")
                .args(["-czf", &app_dst, "-C", &format!("{}/macos", bundle_base), "ffnodes.app"])
                .spawn()?;
            tar_gz.wait()?;
        }
    }

    info!("Finished packaging client");

    Ok(())
}
async fn build_server() -> Result<()> {
    info!("Building server");
    let target = std::env::var("CARGO_BUILD_TARGET").ok();

    // Build frontend first
    let mut child = Command::new(which::which("pnpm")?)
        .arg("--filter")
        .arg("ffnodes_server")
        .arg("build:frontend")
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "build server frontend failed with exit code {:?}",
            status.code()
        ));
    }

    // Build server API with optional target
    let mut command = Command::new("cargo");
    command.arg("build").arg("--release").arg("-p").arg("ffnodes_server");

    if let Some(target_arch) = &target {
        command.arg("--target").arg(target_arch);
    }

    let mut child = command.spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "build server api failed with exit code {:?}",
            status.code()
        ));
    }
    debug!("Build server succeeded ✔!");

    info!("Packaging server");
    let toml_version_regex = Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#)?;
    let cargo = fs::read_to_string("./ffnodes-server/Cargo.toml")?;
    let version = toml_version_regex
        .find(&cargo)
        .ok_or_else(|| anyhow!("Cargo.toml not found in Cargo.toml file!"))?
        .as_str()
        .trim_start_matches("version = \"")
        .trim_end_matches("\"");
    let version = semver::Version::parse(version)?;

    // Determine platform and architecture
    let (platform, arch) = get_platform_arch(&target)?;
    let output_file = format!("dist/ffnodes-server-{}-{}-{}.zip", version, platform, arch);

    let mut writer = zip::ZipWriter::new(fs::File::create(output_file)?);

    // Determine artifact path based on target
    let base_path = if let Some(t) = &target {
        format!("target/{}/release", t)
    } else {
        "target/release".to_string()
    };

    #[cfg(target_os = "windows")]
    let artifact_file = Path::new(format!("{}/ffnodes_server.exe", base_path))?;
    #[cfg(not(target_os = "windows"))]
    let artifact_file = Path::new(format!("{}/ffnodes_server", base_path))?;

    let filename = artifact_file.file_name().unwrap().to_str().unwrap();
    writer.start_file(filename, zip::write::SimpleFileOptions::default())?;
    let bytes = fs::read(artifact_file)?;
    writer.write_all(&bytes)?;
    writer.finish()?;

    info!("Finished packaging client");

    Ok(())
}
