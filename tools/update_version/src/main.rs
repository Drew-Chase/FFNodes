use regex::Regex;
use std::fs;
use std::process::Command;

fn main() {
    // Regex for TOML: matches `version = "x.y.z"` (first occurrence after [package])
    let toml_version_regex = Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#).unwrap();
    // Regex for JSON: matches "version": "x.y.z"
    let json_version_regex = Regex::new(r#""version"\s*:\s*"[^"]*""#).unwrap();

    let new_version = std::env::args().nth(1).unwrap_or_else(|| {
        if let Ok(content) = fs::read_to_string("./ffnodes-server/Cargo.toml") {
            if let Some(version) = toml_version_regex.find(&content) {
                let version = version.as_str().trim_start_matches("version = \"").trim_end_matches("\"");
                let mut semver = semver::Version::parse(version).unwrap();
                semver.patch += 1;
                return semver.to_string();
            } else {
                eprintln!("Error: Version not found in Cargo.toml")
            }
        } else {
            eprintln!("Error: Cargo.toml file not found or invalid format")
        }
        eprintln!("Error: Version argument is missing");
        std::process::exit(1);
    });
    
    let new_version = semver::Version::parse(&new_version).unwrap();
    println!("[FFNodes Update Version] New version: {}", new_version);

    let cargo_tomls = ["./ffnodes-client/src-tauri/Cargo.toml", "./ffmpeg/Cargo.toml", "./ffnodes-server/Cargo.toml"];
    let package_json = ["./ffnodes-client/package.json", "./ffnodes-server/package.json", "./package.json"];
    let tauri_config = "ffnodes-client/src-tauri/tauri.conf.json";

    // Update Cargo.toml files
    for cargo in cargo_tomls {
        let content = match fs::read_to_string(cargo) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", cargo, e);
                std::process::exit(1);
            }
        };

        // Replace only the first occurrence (which should be in [package])
        let new_content = toml_version_regex.replace(&content, format!(r#"version = "{}""#, new_version));

        if let Err(e) = fs::write(cargo, new_content.as_ref()) {
            eprintln!("Error writing to {}: {}", cargo, e);
            std::process::exit(1);
        }
        println!("Updated {}", cargo);
    }

    // Update package.json files
    for package in package_json {
        let content = match fs::read_to_string(package) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", package, e);
                std::process::exit(1);
            }
        };

        // Replace only the first occurrence
        let new_content = json_version_regex.replace(&content, format!(r#""version": "{}""#, new_version));

        if let Err(e) = fs::write(package, new_content.as_ref()) {
            eprintln!("Error writing to {}: {}", package, e);
            std::process::exit(1);
        }
        println!("Updated {}", package);
    }

    // Update tauri.conf.json
    {
        let content = match fs::read_to_string(tauri_config) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", tauri_config, e);
                std::process::exit(1);
            }
        };

        // Replace only the first occurrence
        let new_content =
            json_version_regex.replace(&content, format!(r#""version": "{}.{}.{}""#, new_version.major, new_version.minor, new_version.patch));

        if let Err(e) = fs::write(tauri_config, new_content.as_ref()) {
            eprintln!("Error writing to {}: {}", tauri_config, e);
            std::process::exit(1);
        }
        println!("Updated {}", tauri_config);

        Command::new("cargo").arg("check").output().expect("failed to execute process");

        // Create tag
        let mut files = Vec::with_capacity(cargo_tomls.len() + package_json.len() + 1);
        files.push(tauri_config);
        files.extend(cargo_tomls);
        files.extend(package_json);
        files.push("Cargo.lock");
        Command::new("git").arg("add").args(files).output().expect("failed to execute process");
        Command::new("git").arg("commit").arg("-m").arg(format!("Updated version to v{}", new_version)).output().expect("failed to execute process");
        Command::new("git").arg("push").output().expect("failed to execute process");
        Command::new("git").arg("tag").arg(format!("v{}", new_version)).output().expect("failed to execute process");
        Command::new("git").arg("push").arg("--tags").output().expect("failed to execute process");
        println!("Pushed to git!")
    }
}
