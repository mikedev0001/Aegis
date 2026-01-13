use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nix::sched::{clone, CloneFlags};
use nix::sys::socket::{socketpair, AddressFamily, SockFlag, SockType};
use nix::unistd::{close, fork, ForkResult, Pid};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid as SysPid, ProcessRefreshKind, System};
use uuid::Uuid;
use warp::Filter;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VMState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

struct VMProcess {
    pid: u32,
    start_time: Instant,
    vnc_port: u16,
}

pub struct VMManager {
    vms: Arc<Mutex<HashMap<String, VMInstance>>>,
    next_vnc_port: AtomicU16,
    base_port: u16,
    data_dir: PathBuf,
}

struct VMInstance {
    config: VMConfig,
    status: VMStatus,
    process: Option<VMProcess>,
    disk_path: PathBuf,
}

impl VMManager {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join("isos")).unwrap();
        fs::create_dir_all(dir.join("disks")).unwrap();
        fs::create_dir_all(dir.join("configs")).unwrap();

        Self {
            vms: Arc::new(Mutex::new(HashMap::new())),
            next_vnc_port: AtomicU16::new(5900),
            base_port: 5900,
            data_dir: dir,
        }
    }

    pub fn create_vm(&self, name: &str, iso_path: &str, memory_mb: u32, cpu_cores: u32, disk_size_gb: u32) -> Result<VMConfig, String> {
        // Validate inputs
        if !Path::new(iso_path).exists() {
            return Err("ISO file does not exist".to_string());
        }
        if memory_mb < 256 || memory_mb > 32768 {
            return Err("Memory must be between 256MB and 32GB".to_string());
        }
        if cpu_cores < 1 || cpu_cores > 16 {
            return Err("CPU cores must be between 1 and 16".to_string());
        }

        let id = Uuid::new_v4().to_string();
        let vnc_port = self.allocate_vnc_port();
        
        let config = VMConfig {
            id: id.clone(),
            name: name.to_string(),
            iso_path: iso_path.to_string(),
            memory_mb,
            cpu_cores,
            disk_size_gb,
            vnc_port,
            vnc_password: None,
        };

        // Create disk image
        let disk_path = self.data_dir.join("disks").join(format!("{}.qcow2", id));
        self.create_disk_image(&disk_path, disk_size_gb)?;

        // Save config
        let config_path = self.data_dir.join("configs").join(format!("{}.json", id));
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(config_path, config_json).unwrap();

        let instance = VMInstance {
            config: config.clone(),
            status: VMStatus {
                id: id.clone(),
                name: name.to_string(),
                state: VMState::Stopped,
                pid: None,
                cpu_usage: 0.0,
                memory_mb: 0,
                vnc_port,
                uptime_seconds: 0,
            },
            process: None,
            disk_path,
        };

        self.vms.lock().unwrap().insert(id, instance);
        
        Ok(config)
    }

    pub fn start_vm(&self, vm_id: &str) -> Result<(), String> {
        let mut vms = self.vms.lock().unwrap();
        let instance = vms.get_mut(vm_id).ok_or("VM not found")?;
        
        match instance.status.state {
            VMState::Running => return Err("VM already running".to_string()),
            VMState::Starting => return Err("VM is starting".to_string()),
            _ => {}
        }

        instance.status.state = VMState::Starting;
        
        // Spawn VM in separate thread
        let config = instance.config.clone();
        let disk_path = instance.disk_path.clone();
        
        thread::spawn(move || {
            match Self::spawn_qemu_process(&config, &disk_path) {
                Ok(pid) => {
                    println!("VM {} started with PID {}", config.id, pid);
                }
                Err(e) => {
                    eprintln!("Failed to start VM {}: {}", config.id, e);
                }
            }
        });

        Ok(())
    }

    pub fn stop_vm(&self, vm_id: &str) -> Result<(), String> {
        let mut vms = self.vms.lock().unwrap();
        let instance = vms.get_mut(vm_id).ok_or("VM not found")?;
        
        if let Some(process) = &instance.process {
            // Send SIGTERM to QEMU
            let pid = Pid::from_raw(process.pid as i32);
            nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM).unwrap();
            
            instance.process = None;
            instance.status.state = VMState::Stopped;
            instance.status.pid = None;
            instance.status.cpu_usage = 0.0;
            instance.status.memory_mb = 0;
        }
        
        Ok(())
    }

    pub fn delete_vm(&self, vm_id: &str) -> Result<(), String> {
        let mut vms = self.vms.lock().unwrap();
        
        // Stop VM if running
        if let Some(instance) = vms.get(vm_id) {
            if matches!(instance.status.state, VMState::Running) {
                self.stop_vm(vm_id)?;
            }
        }
        
        // Remove disk
        if let Some(instance) = vms.get(vm_id) {
            let _ = fs::remove_file(&instance.disk_path);
        }
        
        // Remove config
        let config_path = self.data_dir.join("configs").join(format!("{}.json", vm_id));
        let _ = fs::remove_file(config_path);
        
        vms.remove(vm_id);
        
        Ok(())
    }

    pub fn list_vms(&self) -> Vec<VMStatus> {
        let vms = self.vms.lock().unwrap();
        vms.values().map(|i| i.status.clone()).collect()
    }

    pub fn get_vm_status(&self, vm_id: &str) -> Option<VMStatus> {
        let vms = self.vms.lock().unwrap();
        vms.get(vm_id).map(|i| i.status.clone())
    }

    fn allocate_vnc_port(&self) -> u16 {
        self.next_vnc_port.fetch_add(1, Ordering::SeqCst)
    }

    fn create_disk_image(&self, path: &Path, size_gb: u32) -> Result<(), String> {
        let output = Command::new("qemu-img")
            .arg("create")
            .arg("-f")
            .arg("qcow2")
            .arg(path)
            .arg(format!("{}G", size_gb))
            .output()
            .map_err(|e| format!("Failed to create disk: {}", e))?;
        
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        
        Ok(())
    }

    fn spawn_qemu_process(config: &VMConfig, disk_path: &Path) -> Result<u32, String> {
        let mut cmd = Command::new("qemu-system-x86_64");
        
        // Basic QEMU arguments
        cmd.args(&[
            "-enable-kvm",
            "-cpu", "host",
            "-smp", &config.cpu_cores.to_string(),
            "-m", &config.memory_mb.to_string(),
            "-drive", &format!("file={},format=qcow2", disk_path.display()),
            "-cdrom", &config.iso_path,
            "-boot", "d",
            "-vnc", &format!(":{}", config.vnc_port - 5900),
            "-daemonize",
            "-pidfile", &format!("/tmp/vm-{}.pid", config.id),
        ]);
        
        // Add networking
        cmd.args(&["-netdev", "user,id=net0", "-device", "virtio-net-pci,netdev=net0"]);
        
        // Start QEMU process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn QEMU: {}", e))?;
        
        let pid = child.id();
        
        // Wait a bit for VM to start
        thread::sleep(Duration::from_secs(2));
        
        Ok(pid)
    }
}

#[tokio::main]
async fn main() {
    // Initialize VM manager
    let vm_manager = Arc::new(VMManager::new("/var/lib/vm-manager"));
    
    // Clone manager for routes
    let vm_manager_filter = warp::any().map(move || vm_manager.clone());
    
    // API routes
    let list_vms = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::get())
        .and(vm_manager_filter.clone())
        .and_then(handle_list_vms);
    
    let create_vm = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::post())
        .and(warp::body::json())
        .and(vm_manager_filter.clone())
        .and_then(handle_create_vm);
    
    let start_vm = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("start"))
        .and(warp::post())
        .and(vm_manager_filter.clone())
        .and_then(handle_start_vm);
    
    let stop_vm = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("stop"))
        .and(warp::post())
        .and(vm_manager_filter.clone())
        .and_then(handle_stop_vm);
    
    let delete_vm = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::delete())
        .and(vm_manager_filter.clone())
        .and_then(handle_delete_vm);
    
    let get_vnc_url = warp::path("api")
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("vnc"))
        .and(warp::get())
        .and(vm_manager_filter.clone())
        .and_then(handle_get_vnc_url);
    
    // Serve static files
    let static_files = warp::fs::dir("./frontend");
    
    // Combine routes
    let routes = list_vms
        .or(create_vm)
        .or(start_vm)
        .or(stop_vm)
        .or(delete_vm)
        .or(get_vnc_url)
        .or(static_files);
    
    println!("Server starting on http://127.0.0.1:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// Handler functions
async fn handle_list_vms(vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    let vms = vm_manager.list_vms();
    Ok(warp::reply::json(&vms))
}

async fn handle_create_vm(body: CreateVMRequest, vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match vm_manager.create_vm(
        &body.name,
        &body.iso_path,
        body.memory_mb,
        body.cpu_cores,
        body.disk_size_gb,
    ) {
        Ok(config) => Ok(warp::reply::json(&config)),
        Err(e) => Ok(warp::reply::json(&ErrorResponse { error: e })),
    }
}

async fn handle_start_vm(vm_id: String, vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match vm_manager.start_vm(&vm_id) {
        Ok(()) => Ok(warp::reply::json(&SuccessResponse { success: true })),
        Err(e) => Ok(warp::reply::json(&ErrorResponse { error: e })),
    }
}

async fn handle_stop_vm(vm_id: String, vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match vm_manager.stop_vm(&vm_id) {
        Ok(()) => Ok(warp::reply::json(&SuccessResponse { success: true })),
        Err(e) => Ok(warp::reply::json(&ErrorResponse { error: e })),
    }
}

async fn handle_delete_vm(vm_id: String, vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    match vm_manager.delete_vm(&vm_id) {
        Ok(()) => Ok(warp::reply::json(&SuccessResponse { success: true })),
        Err(e) => Ok(warp::reply::json(&ErrorResponse { error: e })),
    }
}

async fn handle_get_vnc_url(vm_id: String, vm_manager: Arc<VMManager>) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(status) = vm_manager.get_vm_status(&vm_id) {
        Ok(warp::reply::json(&VncUrlResponse {
            url: format!("ws://127.0.0.1:6080/websockify?host=127.0.0.1&port={}", status.vnc_port),
        }))
    } else {
        Ok(warp::reply::json(&ErrorResponse { error: "VM not found".to_string() }))
    }
}

#[derive(Deserialize)]
struct CreateVMRequest {
    name: String,
    iso_path: String,
    memory_mb: u32,
    cpu_cores: u32,
    disk_size_gb: u32,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
}

#[derive(Serialize)]
struct VncUrlResponse {
    url: String,
}