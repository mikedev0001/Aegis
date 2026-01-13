use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::io::Write;

use crate::security::validation::{validate_iso_path, calculate_file_hash, ValidationError};

#[derive(Debug, thiserror::Error)]
pub enum IsoError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    #[error("ISO not found: {0}")]
    NotFound(String),
    #[error("ISO already exists: {0}")]
    AlreadyExists(String),
    #[error("Upload failed: {0}")]
    UploadFailed(String),
}

pub struct IsoManager {
    iso_dir: PathBuf,
}

impl IsoManager {
    pub fn new(iso_dir: &Path) -> Self {
        Self {
            iso_dir: iso_dir.to_path_buf(),
        }
    }

    pub fn add_iso(&self, source_path: &Path, name: Option<&str>) -> Result<IsoInfo, IsoError> {
        // Validate source path
        validate_iso_path(source_path.to_str().unwrap_or(""))?;
        
        let file_name = name.map(|n| n.to_string())
            .unwrap_or_else(|| {
                source_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });
        
        let dest_path = self.iso_dir.join(&file_name);
        
        // Check if ISO already exists
        if dest_path.exists() {
            return Err(IsoError::AlreadyExists(file_name));
        }
        
        // Copy ISO file
        fs::copy(source_path, &dest_path)?;
        
        // Calculate hash
        let hash = calculate_file_hash(&dest_path)?;
        
        // Get file size
        let metadata = fs::metadata(&dest_path)?;
        let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Create info file
        let info = IsoInfo {
            name: file_name.clone(),
            path: dest_path,
            size_gb,
            hash,
            uploaded_at: chrono::Utc::now(),
        };
        
        // Save info as JSON
        let info_json = serde_json::to_string_pretty(&info)?;
        let info_path = self.iso_dir.join(format!("{}.json", file_name));
        fs::write(info_path, info_json)?;
        
        Ok(info)
    }

    pub fn upload_iso(&self, data: &[u8], filename: &str) -> Result<IsoInfo, IsoError> {
        // Validate filename
        validate_iso_path(filename)?;
        
        let dest_path = self.iso_dir.join(filename);
        
        // Check if ISO already exists
        if dest_path.exists() {
            return Err(IsoError::AlreadyExists(filename.to_string()));
        }
        
        // Write uploaded data
        let mut file = fs::File::create(&dest_path)?;
        file.write_all(data)?;
        
        // Calculate hash
        let hash = calculate_file_hash(&dest_path)?;
        
        // Get file size
        let size_gb = data.len() as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Create info
        let info = IsoInfo {
            name: filename.to_string(),
            path: dest_path,
            size_gb,
            hash,
            uploaded_at: chrono::Utc::now(),
        };
        
        // Save info
        let info_json = serde_json::to_string_pretty(&info)?;
        let info_path = self.iso_dir.join(format!("{}.json", filename));
        fs::write(info_path, info_json)?;
        
        Ok(info)
    }

    pub fn delete_iso(&self, name: &str) -> Result<(), IsoError> {
        let iso_path = self.iso_dir.join(name);
        let info_path = self.iso_dir.join(format!("{}.json", name));
        
        if !iso_path.exists() {
            return Err(IsoError::NotFound(name.to_string()));
        }
        
        // Delete ISO file
        fs::remove_file(&iso_path)?;
        
        // Delete info file if exists
        if info_path.exists() {
            fs::remove_file(&info_path)?;
        }
        
        Ok(())
    }

    pub fn get_iso(&self, name: &str) -> Result<IsoInfo, IsoError> {
        let info_path = self.iso_dir.join(format!("{}.json", name));
        
        if info_path.exists() {
            let data = fs::read_to_string(&info_path)?;
            let info: IsoInfo = serde_json::from_str(&data)?;
            Ok(info)
        } else {
            // Try to create info from ISO file
            let iso_path = self.iso_dir.join(name);
            if iso_path.exists() {
                let hash = calculate_file_hash(&iso_path)?;
                let metadata = fs::metadata(&iso_path)?;
                let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
                
                let info = IsoInfo {
                    name: name.to_string(),
                    path: iso_path,
                    size_gb,
                    hash,
                    uploaded_at: chrono::Utc::now(),
                };
                
                // Save info for next time
                let info_json = serde_json::to_string_pretty(&info)?;
                fs::write(info_path, info_json)?;
                
                Ok(info)
            } else {
                Err(IsoError::NotFound(name.to_string()))
            }
        }
    }

    pub fn list_isos(&self) -> Result<Vec<IsoInfo>, IsoError> {
        let mut isos = Vec::new();
        
        for entry in fs::read_dir(&self.iso_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let extension = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                // Skip JSON files
                if extension == "json" {
                    continue;
                }
                
                // Check if it's an ISO file by extension
                let valid_extensions = vec!["iso", "img", "qcow2", "raw"];
                if valid_extensions.contains(&extension) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(info) = self.get_iso(name) {
                            isos.push(info);
                        }
                    }
                }
            }
        }
        
        Ok(isos)
    }

    pub fn verify_iso(&self, name: &str, expected_hash: &str) -> Result<bool, IsoError> {
        let info = self.get_iso(name)?;
        Ok(info.hash == expected_hash)
    }

    pub fn get_iso_path(&self, name: &str) -> Result<PathBuf, IsoError> {
        let iso_path = self.iso_dir.join(name);
        
        if iso_path.exists() {
            Ok(iso_path)
        } else {
            Err(IsoError::NotFound(name.to_string()))
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsoInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_gb: f64,
    pub hash: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

impl IsoInfo {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}