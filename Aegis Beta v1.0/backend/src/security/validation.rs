use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use regex::Regex;
use blake3::Hasher;

use crate::vm::config::{CreateVMRequest, UpdateVMRequest};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid VM name: {0}")]
    InvalidName(String),
    #[error("Invalid ISO path: {0}")]
    InvalidIsoPath(String),
    #[error("Invalid memory size: {0} MB (must be between 256 and 32768)")]
    InvalidMemory(u32),
    #[error("Invalid CPU cores: {0} (must be between 1 and 16)")]
    InvalidCpu(u32),
    #[error("Invalid disk size: {0} GB (must be between 10 and 1000)")]
    InvalidDisk(u32),
    #[error("Invalid VNC port: {0} (must be between 5900 and 5999)")]
    InvalidVncPort(u16),
    #[error("Path contains invalid characters or traversal attempts: {0}")]
    InvalidPath(String),
    #[error("ISO file hash mismatch")]
    IsoHashMismatch,
    #[error("ISO file too large (max 10GB)")]
    IsoTooLarge,
    #[error("Command injection attempt detected")]
    CommandInjection,
}

pub fn validate_vm_config(config: &CreateVMRequest) -> Result<(), ValidationError> {
    // Validate VM name
    validate_vm_name(&config.name)?;
    
    // Validate ISO path
    validate_iso_path(&config.iso_path)?;
    
    // Validate resource limits
    validate_memory(config.memory_mb)?;
    validate_cpu(config.cpu_cores)?;
    validate_disk(config.disk_size_gb)?;
    
    Ok(())
}

pub fn validate_vm_name(name: &str) -> Result<(), ValidationError> {
    let name_regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]{1,31}$").unwrap();
    
    if !name_regex.is_match(name) {
        return Err(ValidationError::InvalidName(
            "Name must be 2-32 characters, start with alphanumeric, and contain only a-z, A-Z, 0-9, _, -".to_string()
        ));
    }
    
    // Check for reserved names
    let reserved = vec!["none", "null", "all", "default", "system"];
    if reserved.contains(&name.to_lowercase().as_str()) {
        return Err(ValidationError::InvalidName(
            "Name is reserved".to_string()
        ));
    }
    
    Ok(())
}

pub fn validate_iso_path(path: &str) -> Result<(), ValidationError> {
    let path = Path::new(path);
    
    // Check for path traversal attempts
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(ValidationError::InvalidPath(
            "Path contains parent directory traversal".to_string()
        ));
    }
    
    // Check file extension
    let extension = path.extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();
    
    let valid_extensions = vec!["iso", "img", "qcow2", "raw"];
    if !valid_extensions.contains(&extension.as_str()) {
        return Err(ValidationError::InvalidIsoPath(
            format!("Invalid file extension: .{} (must be .iso, .img, .qcow2, or .raw)", extension)
        ));
    }
    
    // Check if file exists (for local paths)
    if !path.exists() && !path.starts_with("http://") && !path.starts_with("https://") {
        return Err(ValidationError::InvalidIsoPath(
            "File does not exist".to_string()
        ));
    }
    
    // Check file size if it exists
    if path.exists() {
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
            if size_gb > 10.0 {
                return Err(ValidationError::IsoTooLarge);
            }
        }
    }
    
    Ok(())
}

pub fn validate_memory(memory_mb: u32) -> Result<(), ValidationError> {
    if memory_mb < 256 || memory_mb > 32768 {
        Err(ValidationError::InvalidMemory(memory_mb))
    } else {
        Ok(())
    }
}

pub fn validate_cpu(cpu_cores: u32) -> Result<(), ValidationError> {
    if cpu_cores < 1 || cpu_cores > 16 {
        Err(ValidationError::InvalidCpu(cpu_cores))
    } else {
        Ok(())
    }
}

pub fn validate_disk(disk_gb: u32) -> Result<(), ValidationError> {
    if disk_gb < 10 || disk_gb > 1000 {
        Err(ValidationError::InvalidDisk(disk_gb))
    } else {
        Ok(())
    }
}

pub fn validate_vnc_port(port: u16) -> Result<(), ValidationError> {
    if port < 5900 || port > 5999 {
        Err(ValidationError::InvalidVncPort(port))
    } else {
        Ok(())
    }
}

pub fn sanitize_command(input: &str) -> Result<String, ValidationError> {
    // Check for command injection attempts
    let dangerous_patterns = vec![
        (";", "semicolon"),
        ("&&", "double ampersand"),
        ("||", "double pipe"),
        ("`", "backtick"),
        ("$(", "command substitution"),
        ("|", "pipe"),
        (">", "output redirect"),
        ("<", "input redirect"),
        ("&", "background process"),
    ];
    
    for (pattern, name) in dangerous_patterns {
        if input.contains(pattern) {
            return Err(ValidationError::CommandInjection);
        }
    }
    
    // Remove any non-printable characters
    let sanitized: String = input.chars()
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .collect();
    
    Ok(sanitized.trim().to_string())
}

pub fn calculate_file_hash(path: &Path) -> Result<String, ValidationError> {
    let mut hasher = Hasher::new();
    let mut file = std::fs::File::open(path)
        .map_err(|_| ValidationError::InvalidIsoPath("Cannot open file".to_string()))?;
    
    std::io::copy(&mut file, &mut hasher)
        .map_err(|_| ValidationError::InvalidIsoPath("Cannot read file".to_string()))?;
    
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn validate_iso_hash(path: &Path, expected_hash: &str) -> Result<(), ValidationError> {
    let actual_hash = calculate_file_hash(path)?;
    
    if actual_hash != expected_hash {
        Err(ValidationError::IsoHashMismatch)
    } else {
        Ok(())
    }
}

pub fn validate_network_config(bridge: &str, subnet: &str) -> Result<(), ValidationError> {
    // Validate bridge name
    let bridge_regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]{0,15}$").unwrap();
    if !bridge_regex.is_match(bridge) {
        return Err(ValidationError::InvalidPath(
            "Invalid bridge name".to_string()
        ));
    }
    
    // Validate subnet
    let subnet_regex = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/\d{1,2}$").unwrap();
    if !subnet_regex.is_match(subnet) {
        return Err(ValidationError::InvalidPath(
            "Invalid subnet format (expected CIDR notation)".to_string()
        ));
    }
    
    Ok(())
}