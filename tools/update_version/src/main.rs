use std::fs;

fn main() {
    let new_version = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Error: Version argument is missing");
        std::process::exit(1);
    });
    println!("[FFNodes Update Version] New version: {}", new_version);

    let cargo_tomls = ["./ffnodes-client/src-tauri/Cargo.toml", "./ffmpeg/Cargo.toml", "./ffnodes-server/Cargo.toml"];
    let package_json = ["./ffnodes-client/package.json"];
    let tauri_config = "ffnodes-client/src-tauri/tauri.conf.json";

    for cargo in cargo_tomls {
        let content = match fs::read_to_string(cargo) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", cargo, e);
                std::process::exit(1);
            }
        };
        let mut toml: toml::Value = match toml::from_str(&content) {
            Ok(toml) => toml,
            Err(e) => {
                eprintln!("Error parsing TOML from {}: {}", cargo, e);
                std::process::exit(1);
            }
        };
        let package = match toml.get_mut("package") {
            Some(package) => package,
            None => {
                eprintln!("Error: No package field found in {}", cargo);
                std::process::exit(1);
            }
        };
        package["version"] = toml::Value::String(new_version.to_string());
        let new_content = toml.to_string();
        if let Err(e) = fs::write(cargo, new_content) {
            eprintln!("Error writing to {}: {}", cargo, e);
            std::process::exit(1);
        }
    }
    for package in package_json {
        let content = match fs::read_to_string(package) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", package, e);
                std::process::exit(1);
            }
        };
        let mut package_json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error parsing JSON from {}: {}", package, e);
                std::process::exit(1);
            }
        };
        package_json["version"] = serde_json::Value::String(new_version.to_string());
        let new_content = match serde_json::to_string_pretty(&package_json) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error serializing JSON for {}: {}", package, e);
                std::process::exit(1);
            }
        };
        if let Err(e) = fs::write(package, new_content) {
            eprintln!("Error writing to {}: {}", package, e);
            std::process::exit(1);
        }
    }

    {
        let content = match fs::read_to_string(tauri_config) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", tauri_config, e);
                std::process::exit(1);
            }
        };
        let mut tauri_config_json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error parsing JSON from {}: {}", tauri_config, e);
                std::process::exit(1);
            }
        };
        tauri_config_json["version"] = serde_json::Value::String(new_version.to_string());
        let new_content = match serde_json::to_string_pretty(&tauri_config_json) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error serializing JSON for {}: {}", tauri_config, e);
                std::process::exit(1);
            }
        };
        if let Err(e) = fs::write(tauri_config, new_content) {
            eprintln!("Error writing to {}: {}", tauri_config, e);
            std::process::exit(1);
        }
    }
}
