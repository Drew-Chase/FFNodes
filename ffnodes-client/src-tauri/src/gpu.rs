use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub name: String,
    pub encoder_h264: String,
    pub encoder_h265: String,
}

impl GpuInfo {
    pub fn fallback() -> Self {
        Self {
            vendor: "Software".to_string(),
            name: "Software Encoder".to_string(),
            encoder_h264: "libx264".to_string(),
            encoder_h265: "libx265".to_string(),
        }
    }
}

#[cfg(target_os = "windows")]
pub fn detect_gpu() -> anyhow::Result<GpuInfo> {
    use wmi::{COMLibrary, WMIConnection};

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    // Query for video controllers
    let results: Vec<std::collections::HashMap<String, wmi::Variant>> =
        wmi_con.raw_query("SELECT Name, AdapterCompatibility FROM Win32_VideoController")?;

    for gpu in results {
        let name = gpu
            .get("Name")
            .and_then(|v| match v {
                wmi::Variant::String(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        let vendor = gpu
            .get("AdapterCompatibility")
            .and_then(|v| match v {
                wmi::Variant::String(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        // Check for NVIDIA
        if vendor.to_lowercase().contains("nvidia") || name.to_lowercase().contains("nvidia") {
            return Ok(GpuInfo {
                vendor: "NVIDIA".to_string(),
                name,
                encoder_h264: "h264_nvenc".to_string(),
                encoder_h265: "hevc_nvenc".to_string(),
            });
        }

        // Check for AMD
        if vendor.to_lowercase().contains("amd")
            || vendor.to_lowercase().contains("advanced micro devices")
            || name.to_lowercase().contains("amd")
            || name.to_lowercase().contains("radeon")
        {
            return Ok(GpuInfo {
                vendor: "AMD".to_string(),
                name,
                encoder_h264: "h264_amf".to_string(),
                encoder_h265: "hevc_amf".to_string(),
            });
        }

        // Check for Intel
        if vendor.to_lowercase().contains("intel") || name.to_lowercase().contains("intel") {
            return Ok(GpuInfo {
                vendor: "Intel".to_string(),
                name,
                encoder_h264: "h264_qsv".to_string(),
                encoder_h265: "hevc_qsv".to_string(),
            });
        }
    }

    // Fallback to software encoding
    Ok(GpuInfo::fallback())
}

#[cfg(not(target_os = "windows"))]
pub fn detect_gpu() -> anyhow::Result<GpuInfo> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    // Try to detect GPU from system info
    // This is a simplified approach for non-Windows systems
    // You might want to use platform-specific APIs for better detection

    // For macOS, VideoToolbox is available
    #[cfg(target_os = "macos")]
    {
        return Ok(GpuInfo {
            vendor: "Apple".to_string(),
            name: "VideoToolbox".to_string(),
            encoder_h264: "h264_videotoolbox".to_string(),
            encoder_h265: "hevc_videotoolbox".to_string(),
        });
    }

    // For Linux, try to detect GPU
    #[cfg(target_os = "linux")]
    {
        // Check for NVIDIA
        if std::path::Path::new("/proc/driver/nvidia/version").exists() {
            return Ok(GpuInfo {
                vendor: "NVIDIA".to_string(),
                name: "NVIDIA GPU".to_string(),
                encoder_h264: "h264_nvenc".to_string(),
                encoder_h265: "hevc_nvenc".to_string(),
            });
        }

        // Check for AMD
        if std::path::Path::new("/sys/class/drm/card0/device/vendor").exists() {
            if let Ok(vendor_id) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
                if vendor_id.trim() == "0x1002" {
                    // AMD vendor ID
                    return Ok(GpuInfo {
                        vendor: "AMD".to_string(),
                        name: "AMD GPU".to_string(),
                        encoder_h264: "h264_amf".to_string(),
                        encoder_h265: "hevc_amf".to_string(),
                    });
                } else if vendor_id.trim() == "0x8086" {
                    // Intel vendor ID
                    return Ok(GpuInfo {
                        vendor: "Intel".to_string(),
                        name: "Intel GPU".to_string(),
                        encoder_h264: "h264_qsv".to_string(),
                        encoder_h265: "hevc_qsv".to_string(),
                    });
                }
            }
        }

        // Try VAAPI as fallback for Linux
        if std::path::Path::new("/dev/dri").exists() {
            return Ok(GpuInfo {
                vendor: "Linux".to_string(),
                name: "VAAPI".to_string(),
                encoder_h264: "h264_vaapi".to_string(),
                encoder_h265: "hevc_vaapi".to_string(),
            });
        }
    }

    // Fallback to software encoding
    Ok(GpuInfo::fallback())
}
