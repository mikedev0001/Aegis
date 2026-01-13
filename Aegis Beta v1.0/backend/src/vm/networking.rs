use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),
    #[error("Invalid subnet: {0}")]
    InvalidSubnet(String),
    #[error("Bridge already exists: {0}")]
    BridgeExists(String),
    #[error("Bridge not found: {0}")]
    BridgeNotFound(String),
    #[error("Tap interface already exists: {0}")]
    TapExists(String),
    #[error("Tap interface not found: {0}")]
    TapNotFound(String),
}

pub struct NetworkManager {
    bridge_name: String,
    subnet: Ipv4Addr,
    netmask: u8,
    dhcp_start: Ipv4Addr,
    dhcp_end: Ipv4Addr,
}

impl NetworkManager {
    pub fn new(
        bridge_name: &str,
        subnet: &str,
        netmask: u8,
        dhcp_start: &str,
        dhcp_end: &str,
    ) -> Result<Self, NetworkError> {
        let subnet_addr = Ipv4Addr::from_str(subnet)
            .map_err(|_| NetworkError::InvalidSubnet(subnet.to_string()))?;
        
        let dhcp_start_addr = Ipv4Addr::from_str(dhcp_start)
            .map_err(|_| NetworkError::InvalidIp(dhcp_start.to_string()))?;
        
        let dhcp_end_addr = Ipv4Addr::from_str(dhcp_end)
            .map_err(|_| NetworkError::InvalidIp(dhcp_end.to_string()))?;
        
        // Validate addresses are in the same subnet
        if !Self::is_in_subnet(&dhcp_start_addr, &subnet_addr, netmask) ||
           !Self::is_in_subnet(&dhcp_end_addr, &subnet_addr, netmask) {
            return Err(NetworkError::InvalidSubnet(
                "DHCP range not in subnet".to_string()
            ));
        }
        
        Ok(Self {
            bridge_name: bridge_name.to_string(),
            subnet: subnet_addr,
            netmask,
            dhcp_start: dhcp_start_addr,
            dhcp_end: dhcp_end_addr,
        })
    }
    
    fn is_in_subnet(ip: &Ipv4Addr, subnet: &Ipv4Addr, mask: u8) -> bool {
        let ip_int = u32::from(*ip);
        let subnet_int = u32::from(*subnet);
        let mask_int = !((1 << (32 - mask)) - 1);
        
        (ip_int & mask_int) == (subnet_int & mask_int)
    }
    
    pub fn create_bridge(&self) -> Result<(), NetworkError> {
        // Check if bridge already exists
        if self.bridge_exists()? {
            return Err(NetworkError::BridgeExists(self.bridge_name.clone()));
        }
        
        // Create bridge
        let output = Command::new("ip")
            .args(&["link", "add", &self.bridge_name, "type", "bridge"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Set bridge up
        let output = Command::new("ip")
            .args(&["link", "set", &self.bridge_name, "up"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Assign IP to bridge
        let cidr = format!("{}/{}", self.subnet, self.netmask);
        let output = Command::new("ip")
            .args(&["addr", "add", &cidr, "dev", &self.bridge_name])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Setup NAT
        self.setup_nat()?;
        
        // Setup DHCP server (dnsmasq)
        self.setup_dhcp()?;
        
        Ok(())
    }
    
    pub fn delete_bridge(&self) -> Result<(), NetworkError> {
        if !self.bridge_exists()? {
            return Err(NetworkError::BridgeNotFound(self.bridge_name.clone()));
        }
        
        // Set bridge down
        let _ = Command::new("ip")
            .args(&["link", "set", &self.bridge_name, "down"])
            .output();
        
        // Delete bridge
        let output = Command::new("ip")
            .args(&["link", "delete", &self.bridge_name])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Cleanup iptables rules
        self.cleanup_nat()?;
        
        Ok(())
    }
    
    pub fn create_tap(&self, tap_name: &str) -> Result<(), NetworkError> {
        // Check if tap already exists
        if self.tap_exists(tap_name)? {
            return Err(NetworkError::TapExists(tap_name.to_string()));
        }
        
        // Create tap interface
        let output = Command::new("ip")
            .args(&["tuntap", "add", tap_name, "mode", "tap"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Set tap up
        let output = Command::new("ip")
            .args(&["link", "set", tap_name, "up"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Add tap to bridge
        let output = Command::new("ip")
            .args(&["link", "set", tap_name, "master", &self.bridge_name])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn delete_tap(&self, tap_name: &str) -> Result<(), NetworkError> {
        if !self.tap_exists(tap_name)? {
            return Err(NetworkError::TapNotFound(tap_name.to_string()));
        }
        
        // Remove tap from bridge
        let _ = Command::new("ip")
            .args(&["link", "set", tap_name, "nomaster"])
            .output();
        
        // Set tap down
        let _ = Command::new("ip")
            .args(&["link", "set", tap_name, "down"])
            .output();
        
        // Delete tap
        let output = Command::new("ip")
            .args(&["tuntap", "delete", tap_name, "mode", "tap"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    fn bridge_exists(&self) -> Result<bool, NetworkError> {
        let output = Command::new("ip")
            .args(&["link", "show", &self.bridge_name])
            .output()?;
        
        Ok(output.status.success())
    }
    
    fn tap_exists(&self, tap_name: &str) -> Result<bool, NetworkError> {
        let output = Command::new("ip")
            .args(&["link", "show", tap_name])
            .output()?;
        
        Ok(output.status.success())
    }
    
    fn setup_nat(&self) -> Result<(), NetworkError> {
        // Enable IP forwarding
        let _ = std::fs::write("/proc/sys/net/ipv4/ip_forward", "1");
        
        // Setup iptables rules
        let rules = vec![
            // NAT rule
            format!("-t nat -A POSTROUTING -s {}/{} -j MASQUERADE", 
                self.subnet, self.netmask),
            // Forwarding rules
            format!("-A FORWARD -i {} -j ACCEPT", self.bridge_name),
            format!("-A FORWARD -o {} -j ACCEPT", self.bridge_name),
        ];
        
        for rule in rules {
            let output = Command::new("iptables")
                .args(rule.split_whitespace())
                .output()?;
            
            if !output.status.success() {
                return Err(NetworkError::CommandFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn cleanup_nat(&self) -> Result<(), NetworkError> {
        // Remove iptables rules
        let rules = vec![
            // NAT rule
            format!("-t nat -D POSTROUTING -s {}/{} -j MASQUERADE", 
                self.subnet, self.netmask),
            // Forwarding rules
            format!("-D FORWARD -i {} -j ACCEPT", self.bridge_name),
            format!("-D FORWARD -o {} -j ACCEPT", self.bridge_name),
        ];
        
        for rule in rules {
            let _ = Command::new("iptables")
                .args(rule.split_whitespace())
                .output();
        }
        
        Ok(())
    }
    
    fn setup_dhcp(&self) -> Result<(), NetworkError> {
        // Create dnsmasq configuration
        let config = format!(
            "interface={}\n\
             bind-interfaces\n\
             dhcp-range={},{}\n\
             dhcp-option=option:router,{}\n\
             dhcp-option=option:dns-server,8.8.8.8,8.8.4.4\n\
             server=8.8.8.8\n\
             server=8.8.4.4\n\
             log-dhcp\n\
             quiet-dhcp\n",
            self.bridge_name,
            self.dhcp_start,
            self.dhcp_end,
            self.subnet
        );
        
        let config_path = format!("/etc/dnsmasq.d/{}.conf", self.bridge_name);
        std::fs::write(&config_path, config)?;
        
        // Start dnsmasq
        let output = Command::new("systemctl")
            .args(&["restart", "dnsmasq"])
            .output()?;
        
        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn allocate_ip(&self) -> Result<Ipv4Addr, NetworkError> {
        // Simple IP allocation: start from DHCP start + 1
        // In production, you'd want to track allocated IPs
        let next_ip = self.dhcp_start;
        Ok(next_ip)
    }
    
    pub fn list_bridges() -> Result<Vec<String>, NetworkError> {
        let output = Command::new("ip")
            .args(&["link", "show", "type", "bridge"])
            .output()?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let bridges: Vec<String> = output_str.lines()
            .filter_map(|line| {
                line.split(':').nth(1).map(|s| s.trim().to_string())
            })
            .collect();
        
        Ok(bridges)
    }
    
    pub fn list_taps(&self) -> Result<Vec<String>, NetworkError> {
        let output = Command::new("ip")
            .args(&["link", "show", "type", "tuntap"])
            .output()?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let taps: Vec<String> = output_str.lines()
            .filter_map(|line| {
                line.split(':').nth(1).map(|s| s.trim().to_string())
            })
            .filter(|tap| tap.starts_with("tap"))
            .collect();
        
        Ok(taps)
    }
}