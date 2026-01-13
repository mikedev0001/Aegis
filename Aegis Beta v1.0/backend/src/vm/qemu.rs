use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::process;
use tokio::time;

use crate::security::sandbox::VMSandbox;
use super::config::VMConfig;

#[derive(Debug, thiserror::Error)]
pub enum QemuError {
    #[error("Failed to start QEMU: {0}")]
    StartFailed(String),
    #[error("QEMU process not running")]
    NotRunning,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Timeout waiting for QEMU")]
    Timeout,
}

pub struct QemuProcess {
    pid: u32,
    start_time: Instant,
    child: process::Child,
    config: VMConfig,
}

impl QemuProcess {
    pub async fn start(
        config: &VMConfig,
        disk_path: &Path,
        sandbox: VMSandbox,
    ) -> Result<Self, QemuError> {
        // Build QEMU command
        let mut cmd = Command::new("qemu-system-x86_64");
        
        // Apply sandbox if configured
        // Note: In production, this would involve more sophisticated sandboxing
        
        // Basic QEMU arguments
        cmd.arg("-enable-kvm")
            .arg("-cpu").arg(&config.cpu_type)
            .arg("-smp").arg(config.cpu_cores.to_string())
            .arg("-m").arg(format!("{}M", config.memory_mb))
            .arg("-drive").arg(format!("file={},format={}", 
                disk_path.display(), 
                match config.disk_format {
                    super::config::DiskFormat::Qcow2 => "qcow2",
                    super::config::DiskFormat::Raw => "raw",
                    super::config::DiskFormat::Vdi => "vdi",
                    super::config::DiskFormat::Vmdk => "vmdk",
                }))
            .arg("-cdrom").arg(&config.iso_path)
            .arg("-boot").arg("d")
            .arg("-vnc").arg(format!(":{}", config.vnc_port - 5900))
            .arg("-daemonize")
            .arg("-pidfile").arg(format!("/tmp/qemu-{}.pid", config.id));
        
        // Add VNC password if set
        if let Some(password) = &config.vnc_password {
            cmd.arg("-vnc").arg(format!(":{}", config.vnc_port - 5900));
            // Note: Real password handling would use -password option
        }
        
        // Add machine type
        cmd.arg("-machine").arg(&config.machine_type);
        
        // Add network
        match &config.network_type {
            super::config::NetworkType::User => {
                cmd.arg("-netdev").arg("user,id=net0")
                    .arg("-device").arg("virtio-net-pci,netdev=net0");
            }
            super::config::NetworkType::Tap(tap) => {
                cmd.arg("-netdev").arg(format!("tap,id=net0,ifname={}", tap))
                    .arg("-device").arg("virtio-net-pci,netdev=net0");
            }
            super::config::NetworkType::Bridge(bridge) => {
                cmd.arg("-netdev").arg(format!("bridge,id=net0,br={}", bridge))
                    .arg("-device").arg("virtio-net-pci,netdev=net0");
            }
            super::config::NetworkType::None => {
                // No network
            }
        }
        
        // Add BIOS
        match &config.bios {
            super::config::BiosType::SeaBios => {
                // Default, nothing to add
            }
            super::config::BiosType::Ovmf => {
                cmd.arg("-bios").arg("/usr/share/OVMF/OVMF_CODE.fd");
            }
            super::config::BiosType::Custom(path) => {
                cmd.arg("-bios").arg(path);
            }
        }
        
        // Add extra arguments
        for arg in &config.extra_args {
            cmd.arg(arg);
        }
        
        // Redirect output to log file
        let log_path = format!("/var/lib/vm-manager/logs/qemu-{}.log", config.id);
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| QemuError::IoError(e))?;
        
        cmd.stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file));
        
        // Start QEMU process
        let mut child = process::Command::from(cmd)
            .spawn()
            .map_err(|e| QemuError::StartFailed(e.to_string()))?;
        
        let pid = child.id()
            .ok_or_else(|| QemuError::StartFailed("Failed to get PID".to_string()))?;
        
        // Wait for process to start
        time::sleep(Duration::from_secs(2)).await;
        
        // Check if process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(QemuError::StartFailed(
                    format!("QEMU exited with status: {}", status)
                ));
            }
            Ok(None) => {
                // Process is still running, good
            }
            Err(e) => {
                return Err(QemuError::StartFailed(e.to_string()));
            }
        }
        
        Ok(Self {
            pid,
            start_time: Instant::now(),
            child,
            config: config.clone(),
        })
    }
    
    pub async fn stop(&mut self) -> Result<(), QemuError> {
        // Send SIGTERM
        self.child.start_kill()
            .map_err(|e| QemuError::IoError(e))?;
        
        // Wait for process to terminate
        match time::timeout(Duration::from_secs(10), self.child.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(QemuError::IoError(e)),
            Err(_) => {
                // Force kill if timeout
                let _ = self.child.kill().await;
                Err(QemuError::Timeout)
            }
        }
    }
    
    pub async fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }
    
    pub async fn get_status(&mut self) -> Result<ProcessStatus, QemuError> {
        if !self.is_running().await {
            return Err(QemuError::NotRunning);
        }
        
        // Get process info using sysinfo crate
        use sysinfo::{ProcessRefreshKind, RefreshKind, System};
        
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new()),
        );
        system.refresh_processes();
        
        if let Some(process) = system.process(sysinfo::Pid::from(self.pid as usize)) {
            Ok(ProcessStatus {
                cpu_usage: process.cpu_usage(),
                memory_mb: process.memory() / 1024 / 1024,
                uptime_seconds: self.start_time.elapsed().as_secs(),
            })
        } else {
            Err(QemuError::NotRunning)
        }
    }
    
    pub fn pid(&self) -> u32 {
        self.pid
    }
    
    pub fn config(&self) -> &VMConfig {
        &self.config
    }
}

#[derive(Debug, Clone)]
pub struct ProcessStatus {
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub uptime_seconds: u64,
}