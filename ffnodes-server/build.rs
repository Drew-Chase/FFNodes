use std::fs;

fn main() {
	for entry in walkdir::WalkDir::new("src") {
		let entry = entry.unwrap();
		if entry.file_type().is_file() {
			println!("cargo:rerun-if-changed={}", entry.path().display());
		}
	}
	fs::create_dir_all("../target/dev-env/server").expect("failed to create target directory");
}