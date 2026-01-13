use std::collections::HashSet;
use std::net::{TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("No available ports in range")]
    NoPortsAvailable,
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid port range: {0} - {1}")]
    InvalidRange(u16, u16),
}

pub struct PortManager {
    used_ports: Arc<Mutex<HashSet<u16>>>,
    min_port: u16,
    max_port: u16,
}

impl PortManager {
    pub fn new(min_port: u16, max_port: u16) -> Result<Self, PortError> {
        if min_port >= max_port || min_port == 0 || max_port > 65535 {
            return Err(PortError::InvalidRange(min_port, max_port));
        }
        
        Ok(Self {
            used_ports: Arc::new(Mutex::new(HashSet::new())),
            min_port,
            max_port,
        })
    }
    
    pub fn allocate_port(&self) -> Result<u16, PortError> {
        let mut used_ports = self.used_ports.lock().unwrap();
        
        for port in self.min_port..=self.max_port {
            if !used_ports.contains(&port) && self.is_port_available(port)? {
                used_ports.insert(port);
                return Ok(port);
            }
        }
        
        Err(PortError::NoPortsAvailable)
    }
    
    pub fn allocate_specific_port(&self, port: u16) -> Result<(), PortError> {
        if port < self.min_port || port > self.max_port {
            return Err(PortError::InvalidRange(self.min_port, self.max_port));
        }
        
        let mut used_ports = self.used_ports.lock().unwrap();
        
        if used_ports.contains(&port) {
            return Err(PortError::PortInUse(port));
        }
        
        if !self.is_port_available(port)? {
            return Err(PortError::PortInUse(port));
        }
        
        used_ports.insert(port);
        Ok(())
    }
    
    pub fn release_port(&self, port: u16) {
        let mut used_ports = self.used_ports.lock().unwrap();
        used_ports.remove(&port);
    }
    
    pub fn is_port_available(&self, port: u16) -> Result<bool, PortError> {
        // Check TCP
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => {}
            Err(_) => return Ok(false),
        }
        
        // Check UDP
        match UdpSocket::bind(("127.0.0.1", port)) {
            Ok(_) => {}
            Err(_) => return Ok(false),
        }
        
        // Check if port is in privileged range and we have permission
        if port < 1024 {
            // Try to bind to check permissions
            match std::process::Command::new("true").status() {
                Ok(_) => {}
                Err(_) => return Ok(false),
            }
        }
        
        Ok(true)
    }
    
    pub fn scan_available_ports(&self) -> Result<Vec<u16>, PortError> {
        let mut available = Vec::new();
        
        for port in self.min_port..=self.max_port {
            if self.is_port_available(port)? {
                available.push(port);
            }
        }
        
        Ok(available)
    }
    
    pub fn get_used_ports(&self) -> Vec<u16> {
        let used_ports = self.used_ports.lock().unwrap();
        used_ports.iter().copied().collect()
    }
    
    pub fn cleanup(&self) {
        let mut used_ports = self.used_ports.lock().unwrap();
        used_ports.clear();
    }
}

// Helper function to check if a service is listening on a port
pub fn check_service_on_port(port: u16, timeout: Duration) -> bool {
    use std::net::TcpStream;
    
    match TcpStream::connect_timeout(&"127.0.0.1:0".parse().unwrap(), timeout) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Helper function to find an ephemeral port
pub fn find_ephemeral_port() -> Result<u16, PortError> {
    // Bind to port 0 to get an OS-assigned ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

// Network port ranges for different services
pub mod port_ranges {
    pub const VNC: (u16, u16) = (5900, 5999);
    pub const SPICE: (u16, u16) = (5900, 5999);
    pub const SSH: (u16, u16) = (2200, 2299);
    pub const HTTP: (u16, u16) = (8080, 8099);
    pub const WEBSOCKET: (u16, u16) = (6080, 6099);
}