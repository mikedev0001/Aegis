use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMConfig {
    pub id: String,
    pub name: String,
    pub iso_path: String,
    pub memory_mb: u32,
    pub cpu_cores: u32,
    pub disk_size_gb: u32,
    pub vnc_port: u16,
    pub vnc_password: Option<String>,
    pub network_type: NetworkType,
    pub disk_format: DiskFormat,
    pub machine_type: String,
    pub cpu_type: String,
    pub bios: BiosType,
    pub extra_args: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVMRequest {
    pub name: String,
    pub iso_path: String,
    pub memory_mb: u32,
    pub cpu_cores: u32,
    pub disk_size_gb: u32,
    pub vnc_password: Option<String>,
    pub network_type: NetworkType,
    pub disk_format: Option<DiskFormat>,
    pub machine_type: Option<String>,
    pub cpu_type: Option<String>,
    pub bios: Option<BiosType>,
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVMRequest {
    pub name: Option<String>,
    pub memory_mb: Option<u32>,
    pub cpu_cores: Option<u32>,
    pub vnc_password: Option<String>,
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMStatus {
    pub id: String,
    pub name: String,
    pub state: VMState,
    pub pid: Option<u32>,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub vnc_port: u16,
    pub uptime_seconds: u64,
    pub disk_usage_gb: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VMState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Paused,
    Suspended,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    User,
    Tap(String),
    Bridge(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiosType {
    SeaBios,
    Ovmf,
    Custom(String),
}

impl VMConfig {
    pub fn new(req: CreateVMRequest, vnc_port: u16) -> Self {
        let now = chrono::Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            iso_path: req.iso_path,
            memory_mb: req.memory_mb,
            cpu_cores: req.cpu_cores,
            disk_size_gb: req.disk_size_gb,
            vnc_port,
            vnc_password: req.vnc_password,
            network_type: req.network_type,
            disk_format: req.disk_format.unwrap_or(DiskFormat::Qcow2),
            machine_type: req.machine_type.unwrap_or_else(|| "pc".to_string()),
            cpu_type: req.cpu_type.unwrap_or_else(|| "host".to_string()),
            bios: req.bios.unwrap_or(BiosType::SeaBios),
            extra_args: req.extra_args.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn update(&mut self, req: UpdateVMRequest) {
        if let Some(name) = req.name {
            self.name = name;
        }
        
        if let Some(memory_mb) = req.memory_mb {
            self.memory_mb = memory_mb;
        }
        
        if let Some(cpu_cores) = req.cpu_cores {
            self.cpu_cores = cpu_cores;
        }
        
        if let Some(vnc_password) = req.vnc_password {
            self.vnc_password = Some(vnc_password);
        }
        
        if let Some(extra_args) = req.extra_args {
            self.extra_args = extra_args;
        }
        
        self.updated_at = chrono::Utc::now();
    }
    
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = self.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
        
        std::fs::write(path, json)
    }
    
    pub fn load_from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(config)
    }
}