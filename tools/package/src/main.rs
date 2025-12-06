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
        help = "The target package to build for"
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

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum PackageTarget {
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
    let mut child = Command::new(which::which("pnpm")?)
        .arg("build:client-tauri")
        .spawn()?;
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
    let output_file = format!(
        "dist/ffnodes-client-{}-{}.zip",
        version,
        if cfg!(target_os = "windows") {
            "windows-x64"
        } else if cfg!(target_os = "linux") {
            "linux-x64"
        } else if cfg!(target_os = "macos") {
            "macos-x64"
        } else {
            "unknown-x64"
        }
    );
    let mut writer = zip::ZipWriter::new(fs::File::create(output_file)?);
    #[cfg(target_os = "windows")]
    let artifact_file = Path::new("target/release/ffnodes.exe")?;
    #[cfg(not(target_os = "windows"))]
    let artifact_file = Path::new("target/release/ffnodes")?;

    let filename = artifact_file.file_name().unwrap().to_str().unwrap();
    writer.start_file(filename, zip::write::SimpleFileOptions::default())?;
    let bytes = fs::read(artifact_file)?;
    writer.write_all(&bytes)?;
    writer.finish()?;
    #[cfg(target_os = "windows")]
    {
        fs::copy(
            format!(
                "target/release/bundle/msi/ffnodes_{}.{}.{}_x64_en-US.msi",
                version.major, version.minor, version.patch
            ),
            format!("dist/ffnodes_client_{}_windows-x64_en-US.msi", version),
        )?;
        fs::copy(
            format!(
                "target/release/bundle/nsis/ffnodes_{}.{}.{}_x64-setup.exe",
                version.major, version.minor, version.patch
            ),
            format!("dist/ffnodes_client_{}_windows-x64_setup.exe", version),
        )?;
    }

    info!("Finished packaging client");

    Ok(())
}
async fn build_server() -> Result<()> {
    info!("Building server");
    let mut child = Command::new(which::which("pnpm")?)
        .arg("build:server")
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "build server failed with exit code {:?}",
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
    let output_file = format!(
        "dist/ffnodes-server-{}-{}.zip",
        version,
        if cfg!(target_os = "windows") {
            "windows-x64"
        } else if cfg!(target_os = "linux") {
            "linux-x64"
        } else if cfg!(target_os = "macos") {
            "macos-x64"
        } else {
            "unknown-x64"
        }
    );
    let mut writer = zip::ZipWriter::new(fs::File::create(output_file)?);
    #[cfg(target_os = "windows")]
    let artifact_file = Path::new("target/release/ffnodes_server.exe")?;
    #[cfg(not(target_os = "windows"))]
    let artifact_file = Path::new("target/release/ffnodes_server")?;

    let filename = artifact_file.file_name().unwrap().to_str().unwrap();
    writer.start_file(filename, zip::write::SimpleFileOptions::default())?;
    let bytes = fs::read(artifact_file)?;
    writer.write_all(&bytes)?;
    writer.finish()?;

    info!("Finished packaging client");

    Ok(())
}
