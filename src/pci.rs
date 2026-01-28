use std::fs;
use std::path::PathBuf;
use std::io;
use std::fmt;

#[derive(Debug)]
pub enum PciError {
    NotFound,
    PermissionDenied(PathBuf),
    IoError(io::Error),
}

impl fmt::Display for PciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PciError::NotFound => write!(f, "NVIDIA PCI device not found"),
            PciError::PermissionDenied(path) => write!(f, "Permission denied accessing {:?}", path),
            PciError::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl std::error::Error for PciError {}

#[derive(Clone)]
pub struct PciDevice {
    pub address_path: PathBuf,
}

impl PciDevice {
    /// Finds the first NVIDIA GPU device on the system via sysfs.
    pub fn find_nvidia() -> Result<Self, PciError> {
        let entries = fs::read_dir("/sys/bus/pci/devices").map_err(PciError::IoError)?;

        entries
            .filter_map(|e| e.ok())
            .find(|entry| {
                let path = entry.path().join("vendor");
                fs::read_to_string(path)
                    .map(|v| v.trim() == "0x10de")
                    .unwrap_or(false)
            })
            .map(|entry| PciDevice { address_path: entry.path() })
            .ok_or(PciError::NotFound)
    }

    /// Returns the current operational state of the device (e.g., "active" or "suspended").
    pub fn get_runtime_status(&self) -> String {
        fs::read_to_string(self.address_path.join("power/runtime_status"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
    /// Returns the current power management policy (e.g., "auto" or "on").
    pub fn get_runtime_control(&self) -> String {
    fs::read_to_string(self.address_path.join("power/control"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Sets the current power management policy (e.g., "auto" or "on").
    pub fn set_runtime_control(&self, mode: &str) -> Result<(), PciError> {
        let path = self.address_path.join("power/control");
        fs::write(&path, mode).map_err(|e| {
            if e.kind() == io::ErrorKind::PermissionDenied {
                PciError::PermissionDenied(path)
            } else {
                PciError::IoError(e)
            }
        })
    }
}