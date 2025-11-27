use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

/// Validates that a path does not contain path traversal sequences
/// and is a canonicalized absolute path
pub fn validate_path_security(path: &Path) -> Result<PathBuf> {
    // Convert to absolute canonical path
    let canonical = path.canonicalize()
        .map_err(|e| anyhow!("Failed to canonicalize path: {}", e))?;

    // Check for path traversal components
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(anyhow!("Path contains parent directory traversal (..)"));
        }
    }

    Ok(canonical)
}

/// Validates that a path is within an allowed base directory
pub fn validate_path_within_base(path: &Path, allowed_bases: &[PathBuf]) -> Result<PathBuf> {
    let canonical = validate_path_security(path)?;

    // Check if path is within any allowed base directory
    let is_within_allowed_base = allowed_bases.iter().any(|base| {
        canonical.starts_with(base)
    });

    if !is_within_allowed_base {
        return Err(anyhow!(
            "Path is not within any allowed directory. Path: {:?}, Allowed bases: {:?}",
            canonical,
            allowed_bases
        ));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_path_security_rejects_parent_dir() {
        let path = Path::new("../etc/passwd");
        assert!(validate_path_security(path).is_err());
    }

    #[test]
    fn test_validate_path_within_base() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test.txt");

        // Create a test file
        std::fs::write(&test_file, b"test").ok();

        let result = validate_path_within_base(&test_file, &[temp_dir.clone()]);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(&test_file).ok();
    }
}
