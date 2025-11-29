use std::fs;
use walkdir::WalkDir;

fn main() {
	for entry in WalkDir::new("src") {
		let entry = entry.unwrap();
		if entry.file_type().is_file() {
			println!("cargo:rerun-if-changed={}", entry.path().display());
		}
	}
	fs::create_dir_all("target/dev-env").expect("failed to create target directory");
	fs::create_dir_all("target/wwwroot").expect("failed to create wwwroot directory");
}