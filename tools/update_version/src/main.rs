use regex::Regex;
use std::fs;
use std::process::Command;
use std::str::from_utf8;

fn main() {
    let new_version = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Error: Version argument is missing");
        std::process::exit(1);
    });
    println!("[FFNodes Update Version] New version: {}", new_version);

    let cargo_tomls = ["./ffnodes-client/src-tauri/Cargo.toml", "./ffmpeg/Cargo.toml", "./ffnodes-server/Cargo.toml"];
    let package_json = ["./ffnodes-client/package.json"];
    let tauri_config = "ffnodes-client/src-tauri/tauri.conf.json";

    // Regex for TOML: matches `version = "x.y.z"` (first occurrence after [package])
    let toml_version_regex = Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#).unwrap();

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

    // Regex for JSON: matches "version": "x.y.z"
    let json_version_regex = Regex::new(r#""version"\s*:\s*"[^"]*""#).unwrap();

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
        let new_content = json_version_regex.replace(&content, format!(r#""version": "{}""#, new_version));

        if let Err(e) = fs::write(tauri_config, new_content.as_ref()) {
            eprintln!("Error writing to {}: {}", tauri_config, e);
            std::process::exit(1);
        }
        println!("Updated {}", tauri_config);

        // Create tag
        let output = Command::new("git").arg("tag").arg(format!("v{}", new_version)).output().expect("failed to execute process");
        println!("[FFNodes Update Version] {:?}", from_utf8(output.stdout.as_slice()).unwrap());
        let output = Command::new("git").arg("push").arg("--tags").output().expect("failed to execute process");
        println!("[FFNodes Update Version] {:?}", from_utf8(output.stdout.as_slice()).unwrap());
    }
}
