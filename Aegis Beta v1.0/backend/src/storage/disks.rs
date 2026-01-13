use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::security::validation::{validate_disk, ValidationError};

#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    #[error("QEMU-img error: {0}")]
    QemuError(String),
    #[error("Disk not found: {0}")]
    NotFound(String),
    #[error("Disk already exists: {0}")]
    AlreadyExists(String),
}

pub struct DiskManager {
    disk_dir: PathBuf,
}

impl DiskManager {
    pub fn new(disk_dir: &Path) -> Self {
        Self {
            disk_dir: disk_dir.to_path_buf(),
        }
    }

    pub fn create_disk(&self, vm_id: &str, size_gb: u32, format: DiskFormat) -> Result<PathBuf, DiskError> {
        // Validate disk size
        validate_disk(size_gb)?;
        
        let disk_path = self.disk_dir.join(format!("{}.{}", vm_id, format.extension()));
        
        // Check if disk already exists
        if disk_path.exists() {
            return Err(DiskError::AlreadyExists(vm_id.to_string()));
        }
        
        // Create disk image
        let format_str = match format {
            DiskFormat::Qcow2 => "qcow2",
            DiskFormat::Raw => "raw",
            DiskFormat::Vdi => "vdi",
            DiskFormat::Vmdk => "vmdk",
        };
        
        let output = Command::new("qemu-img")
            .arg("create")
            .arg("-f")
            .arg(format_str)
            .arg(&disk_path)
            .arg(format!("{}G", size_gb))
            .output()
            .map_err(|e| DiskError::IoError(e))?;
        
        if !output.status.success() {
            return Err(DiskError::QemuError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Set permissions (owner read/write, group read, others none)
        let mut perms = fs::metadata(&disk_path)?.permissions();
        perms.set_readonly(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o640); // rw-r-----
        }
        fs::set_permissions(&disk_path, perms)?;
        
        Ok(disk_path)
    }

    pub fn delete_disk(&self, vm_id: &str) -> Result<(), DiskError> {
        // Try different formats
        let formats = vec!["qcow2", "raw", "vdi", "vmdk"];
        
        for format in formats {
            let disk_path = self.disk_dir.join(format!("{}.{}", vm_id, format));
            if disk_path.exists() {
                fs::remove_file(&disk_path)?;
                return Ok(());
            }
        }
        
        Err(DiskError::NotFound(vm_id.to_string()))
    }

    pub fn resize_disk(&self, vm_id: &str, new_size_gb: u32) -> Result<(), DiskError> {
        validate_disk(new_size_gb)?;
        
        // Find existing disk
        let formats = vec!["qcow2", "raw", "vdi", "vmdk"];
        let mut disk_path = None;
        
        for format in &formats {
            let path = self.disk_dir.join(format!("{}.{}", vm_id, format));
            if path.exists() {
                disk_path = Some((path, format));
                break;
            }
        }
        
        let (disk_path, format) = disk_path.ok_or_else(|| DiskError::NotFound(vm_id.to_string()))?;
        
        // Create backup
        let backup_path = disk_path.with_extension(format!("{}.backup", format));
        fs::copy(&disk_path, &backup_path)?;
        
        // Resize disk
        let output = Command::new("qemu-img")
            .arg("resize")
            .arg(&disk_path)
            .arg(format!("{}G", new_size_gb))
            .output()
            .map_err(|e| DiskError::IoError(e))?;
        
        if !output.status.success() {
            // Restore from backup
            fs::copy(&backup_path, &disk_path)?;
            fs::remove_file(&backup_path)?;
            
            return Err(DiskError::QemuError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Remove backup
        fs::remove_file(&backup_path)?;
        
        Ok(())
    }

    pub fn get_disk_info(&self, vm_id: &str) -> Result<DiskInfo, DiskError> {
        let formats = vec!["qcow2", "raw", "vdi", "vmdk"];
        
        for format in &formats {
            let disk_path = self.disk_dir.join(format!("{}.{}", vm_id, format));
            if disk_path.exists() {
                let output = Command::new("qemu-img")
                    .arg("info")
                    .arg(&disk_path)
                    .output()
                    .map_err(|e| DiskError::IoError(e))?;
                
                if !output.status.success() {
                    return Err(DiskError::QemuError(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ));
                }
                
                let output_str = String::from_utf8_lossy(&output.stdout);
                return Ok(DiskInfo::from_qemu_output(&output_str, &disk_path));
            }
        }
        
        Err(DiskError::NotFound(vm_id.to_string()))
    }

    pub fn list_disks(&self) -> Result<Vec<DiskInfo>, DiskError> {
        let mut disks = Vec::new();
        
        for entry in fs::read_dir(&self.disk_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy();
                    if matches!(ext.as_ref(), "qcow2" | "raw" | "vdi" | "vmdk") {
                        if let Ok(info) = self.get_disk_info(
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                        ) {
                            disks.push(info);
                        }
                    }
                }
            }
        }
        
        Ok(disks)
    }
}

#[derive(Debug, Clone)]
pub enum DiskFormat {
    Qcow2,
    Raw,
    Vdi,
    Vmdk,
}

impl DiskFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            DiskFormat::Qcow2 => "qcow2",
            DiskFormat::Raw => "raw",
            DiskFormat::Vdi => "vdi",
            DiskFormat::Vmdk => "vmdk",
        }
    }
    
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "qcow2" => Some(DiskFormat::Qcow2),
            "raw" => Some(DiskFormat::Raw),
            "vdi" => Some(DiskFormat::Vdi),
            "vmdk" => Some(DiskFormat::Vmdk),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub path: PathBuf,
    pub format: DiskFormat,
    pub virtual_size_gb: f64,
    pub actual_size_gb: f64,
    pub backing_file: Option<PathBuf>,
    pub encrypted: bool,
    pub snapshot_count: usize,
}

impl DiskInfo {
    fn from_qemu_output(output: &str, path: &Path) -> Self {
        let mut info = DiskInfo {
            path: path.to_path_buf(),
            format: DiskFormat::Raw,
            virtual_size_gb: 0.0,
            actual_size_gb: 0.0,
            backing_file: None,
            encrypted: false,
            snapshot_count: 0,
        };
        
        for line in output.lines() {
            let line = line.trim();
            
            if line.starts_with("file format:") {
                let format_str = line.split(':').nth(1).unwrap_or("").trim();
                info.format = DiskFormat::from_extension(format_str).unwrap_or(DiskFormat::Raw);
            } else if line.starts_with("virtual size:") {
                if let Some(size_part) = line.split('(').nth(1) {
                    if let Some(size_str) = size_part.split(' ').next() {
                        if let Ok(size_bytes) = size_str.parse::<f64>() {
                            info.virtual_size_gb = size_bytes / (1024.0 * 1024.0 * 1024.0);
                        }
                    }
                }
            } else if line.starts_with("disk size:") {
                if let Some(size_str) = line.split(':').nth(1) {
                    let parts: Vec<&str> = size_str.trim().split(' ').collect();
                    if parts.len() >= 2 {
                        if let Ok(size) = parts[0].parse::<f64>() {
                            let unit = parts[1].to_lowercase();
                            info.actual_size_gb = match unit.as_str() {
                                "k" => size / (1024.0 * 1024.0),
                                "m" => size / 1024.0,
                                "g" => size,
                                "t" => size * 1024.0,
                                _ => size / (1024.0 * 1024.0 * 1024.0),
                            };
                        }
                    }
                }
            } else if line.starts_with("backing file:") {
                if let Some(backing_path) = line.split(':').nth(1) {
                    let path_str = backing_path.trim();
                    if !path_str.is_empty() && path_str != "(null)" {
                        info.backing_file = Some(PathBuf::from(path_str));
                    }
                }
            } else if line.contains("encrypted: yes") {
                info.encrypted = true;
            } else if line.contains("Snapshot list:") {
                // Count snapshot lines
                info.snapshot_count = output.lines()
                    .filter(|l| l.contains("snapshot"))
                    .count();
            }
        }
        
        info
    }
}