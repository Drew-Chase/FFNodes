use anyhow::Result;
use sha2::{Digest, Sha256};
use sysinfo::System;
use uuid::Uuid;

/// Generate a stable machine identifier based on hardware characteristics
///
/// This function creates a unique identifier for the machine by combining:
/// 1. MAC address from the first non-virtual network adapter
/// 2. CPU information (processor name/brand)
///
/// The identifiers are hashed with SHA-256 for privacy, preventing reverse engineering
/// of the actual hardware IDs while maintaining uniqueness.
///
/// Fallback: If hardware access fails, generates and returns a UUID instead
pub fn generate_machine_id() -> Result<String> {
    let mut components = Vec::new();

    // Component 1: MAC Address
    if let Ok(mac) = get_mac_address() {
        components.push(mac);
        tracing::debug!("Machine ID: MAC address obtained");
    } else {
        tracing::warn!("Machine ID: Could not obtain MAC address, using fallback");
    }

    // Component 2: CPU Information
    if let Some(cpu_info) = get_cpu_info() {
        components.push(cpu_info);
        tracing::debug!("Machine ID: CPU info obtained");
    } else {
        tracing::warn!("Machine ID: Could not obtain CPU info");
    }

    // If we couldn't get any hardware info, use a generated UUID as fallback
    if components.is_empty() {
        let fallback_id = Uuid::new_v4().to_string();
        tracing::warn!(
            "Machine ID: No hardware identifiers available, using generated UUID: {}",
            fallback_id
        );
        return Ok(fallback_id);
    }

    // Hash all components together for privacy
    let combined = components.join("|");
    let hash = hash_identifier(&combined);

    tracing::info!("Machine ID generated successfully");
    Ok(hash)
}

/// Get MAC address from first non-virtual network adapter
fn get_mac_address() -> Result<String> {
    // Get all MAC addresses
    let mac_addresses = mac_address::get_mac_address()?;

    if let Some(mac) = mac_addresses {
        // Filter out common virtual adapter patterns
        let mac_string = format!("{}", mac);

        // Common virtual adapter MAC prefixes to avoid:
        // - 00:05:69 (VMware)
        // - 00:0C:29 (VMware)
        // - 00:50:56 (VMware)
        // - 00:1C:42 (Parallels)
        // - 08:00:27 (VirtualBox)
        if !is_virtual_mac(&mac_string) {
            return Ok(mac_string);
        }
    }

    // If no suitable MAC found, try getting all adapters
    let all_macs = mac_address::mac_address_by_name("Ethernet")?;
    if let Some(mac) = all_macs {
        return Ok(format!("{}", mac));
    }

    anyhow::bail!("No suitable MAC address found")
}

/// Check if MAC address belongs to a virtual adapter
fn is_virtual_mac(mac: &str) -> bool {
    let virtual_prefixes = [
        "00:05:69", "00:0C:29", "00:50:56", // VMware
        "00:1C:42", // Parallels
        "08:00:27", // VirtualBox
        "00:15:5D", // Hyper-V
    ];

    for prefix in &virtual_prefixes {
        if mac.starts_with(prefix) {
            return true;
        }
    }

    false
}

/// Get CPU information for machine identification
fn get_cpu_info() -> Option<String> {
    let mut sys = System::new_all();
    sys.refresh_cpu();

    // Get CPU brand/name
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand();
        if !brand.is_empty() {
            return Some(brand.to_string());
        }
    }

    None
}

/// Hash the combined identifier with SHA-256
fn hash_identifier(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    // Convert to hex string
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_machine_id() {
        // Should always generate some ID (hardware or fallback)
        let id = generate_machine_id().unwrap();
        assert!(!id.is_empty());
        assert!(id.len() >= 32); // Either SHA-256 (64 chars) or UUID (36 chars)
    }

    #[test]
    fn test_hash_identifier() {
        let input = "test-hardware-id";
        let hash = hash_identifier(input);

        // SHA-256 produces 64-character hex string
        assert_eq!(hash.len(), 64);

        // Same input should produce same hash
        assert_eq!(hash, hash_identifier(input));
    }

    #[test]
    fn test_is_virtual_mac() {
        assert!(is_virtual_mac("00:05:69:AB:CD:EF")); // VMware
        assert!(is_virtual_mac("08:00:27:12:34:56")); // VirtualBox
        assert!(!is_virtual_mac("A4:B1:C2:D3:E4:F5")); // Real adapter
    }
}
