use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use tracing::{debug, warn, error};

/// Validates that a path does not contain path traversal sequences
/// and is a canonicalized absolute path
pub fn validate_path_security(path: &Path) -> Result<PathBuf> {
    debug!("Validating path security: path={:?}", path);

    // Convert to absolute canonical path
    let canonical = path.canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize path: path={:?}, error={}", path, e);
            anyhow!("Failed to canonicalize path: {}", e)
        })?;

    // Check for path traversal components
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            warn!("Path validation failed - contains parent directory traversal: path={:?}", path);
            return Err(anyhow!("Path contains parent directory traversal (..)"));
        }
    }

    debug!("Path validation successful: canonical={:?}", canonical);
    Ok(canonical)
}

/// Validates that a path does not contain path traversal sequences
/// This version works for paths that don't exist yet (e.g., files about to be created)
pub fn validate_path_security_non_existent(path: &Path) -> Result<PathBuf> {
    debug!("Validating non-existent path security: path={:?}", path);

    // Check for path traversal components
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            warn!("Path validation failed - contains parent directory traversal: path={:?}", path);
            return Err(anyhow!("Path contains parent directory traversal (..)"));
        }
    }

    // Convert to absolute path (doesn't require file to exist)
    let absolute = if path.is_absolute() {
        debug!("Path is already absolute: path={:?}", path);
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir()
            .map_err(|e| {
                error!("Failed to get current directory: error={}", e);
                anyhow!("Failed to get current directory: {}", e)
            })?;
        let abs_path = cwd.join(path);
        debug!("Converted relative path to absolute: path={:?}, absolute={:?}", path, abs_path);
        abs_path
    };

    debug!("Path validation successful: absolute={:?}", absolute);
    Ok(absolute)
}

/// Validates that a path is within an allowed base directory
pub fn validate_path_within_base(path: &Path, allowed_bases: &[PathBuf]) -> Result<PathBuf> {
    debug!("Validating path is within allowed bases: path={:?}, allowed_bases={:?}", path, allowed_bases);

    let canonical = validate_path_security(path)?;

    // Check if path is within any allowed base directory
    let is_within_allowed_base = allowed_bases.iter().any(|base| {
        let matches = canonical.starts_with(base);
        debug!("Checking if path starts with base: canonical={:?}, base={:?}, matches={}", canonical, base, matches);
        matches
    });

    if !is_within_allowed_base {
        warn!(
            "Path validation failed - not within allowed directories: canonical={:?}, allowed_bases={:?}",
            canonical,
            allowed_bases
        );
        return Err(anyhow!(
            "Path is not within any allowed directory. Path: {:?}, Allowed bases: {:?}",
            canonical,
            allowed_bases
        ));
    }

    debug!("Path is within allowed base directories: canonical={:?}", canonical);
    Ok(canonical)
}

/// Validates that a path is within an allowed base directory
/// This version works for paths that don't exist yet
pub fn validate_path_within_base_non_existent(path: &Path, allowed_bases: &[PathBuf]) -> Result<PathBuf> {
    debug!("Validating non-existent path is within allowed bases: path={:?}, allowed_bases={:?}", path, allowed_bases);

    let absolute = validate_path_security_non_existent(path)?;

    // Check if path is within any allowed base directory
    let is_within_allowed_base = allowed_bases.iter().any(|base| {
        let matches = absolute.starts_with(base);
        debug!("Checking if absolute path starts with base: absolute={:?}, base={:?}, matches={}", absolute, base, matches);
        matches
    });

    if !is_within_allowed_base {
        warn!(
            "Path validation failed - not within allowed directories: absolute={:?}, allowed_bases={:?}",
            absolute,
            allowed_bases
        );
        return Err(anyhow!(
            "Path is not within any allowed directory. Path: {:?}, Allowed bases: {:?}",
            absolute,
            allowed_bases
        ));
    }

    debug!("Path is within allowed base directories: absolute={:?}", absolute);
    Ok(absolute)
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
