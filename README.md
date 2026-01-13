Aegis VM Manager
A lightweight, web-based virtual machine management tool with direct QEMU/KVM control. Built for performance, security, and simplicity.

https://img.shields.io/badge/Status-Beta-orange
https://img.shields.io/badge/Backend-Rust-red
https://img.shields.io/badge/Frontend-Vanilla%2520JS-yellow
https://img.shields.io/badge/License-MIT-green

🚀 Features
Core Features
Web-based Management: Clean, minimal dashboard for VM operations

Direct QEMU/KVM Control: No libvirt overhead, maximum performance

VNC Console in Browser: Embedded noVNC support for VM access

Multiple VM Isolation: Secure namespace-based sandboxing

Low Resource Usage: Optimized for low-end hardware

VM Management
Create VMs from ISO files (local or uploaded)

Configure RAM, CPU cores, disk size

Start/Stop/Delete VM operations

Real-time status monitoring

Console access via WebSocket

Security
Process sandboxing with namespaces

Input validation and sanitization

Command injection prevention

Resource limits enforcement

Network isolation

📋 Requirements
Hardware
x86_64 CPU with virtualization support (Intel VT-x or AMD-V)

Minimum 2GB RAM (4GB+ recommended)

20GB+ free disk space

Software
OS: Linux (Ubuntu 20.04+, Debian 11+, Fedora 34+, or Arch Linux)

KVM: Kernel-based Virtual Machine support

QEMU: 5.0 or newer

Rust: 1.70+ (for building backend)

Root Access: Required for setup

🛠️ Quick Installation
One-Command Install (Recommended)
bash
# Clone the repository
git clone https://github.com/yourusername/aegis-vm-manager.git
cd aegis-vm-manager

# Run automated setup (requires sudo)
sudo make setup
Wait for the script to complete, then access: http://localhost:3030

Step-by-Step Manual Install
Check virtualization support:

bash
grep -E -c '(vmx|svm)' /proc/cpuinfo  # Should return > 0
sudo apt-get install cpu-checker && kvm-ok
Install dependencies:

bash
sudo apt-get update
sudo apt-get install -y \
    qemu-kvm \
    qemu-utils \
    libvirt-daemon-system \
    bridge-utils \
    websockify \
    build-essential \
    libssl-dev
Add user to required groups:

bash
sudo usermod -aG kvm $USER
sudo usermod -aG libvirt $USER
# Log out and back in for changes to take effect
Build and install:

bash
# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build backend
cd backend
cargo build --release
sudo cp target/release/vm-manager /usr/local/bin/

# Setup directories
sudo mkdir -p /var/lib/vm-manager/{isos,disks,configs,logs}
sudo chown -R $USER:$USER /var/lib/vm-manager

# Install systemd service
sudo cp ../systemd/vm-manager.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable vm-manager
sudo systemctl start vm-manager
Access the web interface:

Open browser to: http://localhost:3030

📖 Usage Guide
Creating Your First VM
Prepare an ISO:

bash
# Download a test ISO (e.g., Alpine Linux)
sudo wget https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/x86_64/alpine-standard-3.18.0-x86_64.iso \
  -P /var/lib/vm-manager/isos/
Access Web Interface:

Open http://localhost:3030

Click "Create VM" button

Configure VM:

Name: my-first-vm

ISO Path: /var/lib/vm-manager/isos/alpine-standard-3.18.0-x86_64.iso

Memory: 1024 MB

CPU Cores: 2

Disk Size: 10 GB

Click "Create & Start"

Access Console:

Wait for VM status to show "Running"

Click "Console" button

Install OS through web interface

Example VM Configurations
Ubuntu Server:

bash
# Download Ubuntu Server ISO
wget https://releases.ubuntu.com/22.04/ubuntu-22.04.3-live-server-amd64.iso \
  -P /var/lib/vm-manager/isos/
Memory: 2048 MB

CPU Cores: 2

Disk: 20 GB

Windows 10:

Memory: 4096 MB

CPU Cores: 4

Disk: 50 GB

BIOS: OVMF (for UEFI)

🔧 Configuration
Main Configuration File
Edit /etc/vm-manager/default.toml:

toml
[server]
host = "127.0.0.1"  # Change to "0.0.0.0" for network access
port = 3030

[qemu]
path = "/usr/bin/qemu-system-x86_64"
enable_kvm = true

[network]
default_bridge = "br0"
nat_network = "192.168.122.0/24"

[limits]
max_vms = 10
max_memory_mb = 32768
max_cpu_cores = 16
Environment Variables
Create .env file:

bash
cp .env.example .env
nano .env
Key variables:

SERVER_HOST: Bind address (0.0.0.0 for all interfaces)

SERVER_PORT: Web interface port

DATA_DIR: Path for VM data

LOG_LEVEL: Debug level (error, warn, info, debug)

📡 API Reference
REST API Endpoints
Method	Endpoint	Description
GET	/api/health	Service health check
GET	/api/vms	List all VMs
GET	/api/vms/{id}	Get VM details
POST	/api/vms	Create new VM
POST	/api/vms/{id}/start	Start VM
POST	/api/vms/{id}/stop	Stop VM
DELETE	/api/vms/{id}	Delete VM
GET	/api/vms/{id}/vnc	Get VNC URL
WebSocket API
Connect to ws://localhost:3030/ws for real-time updates:

javascript
const ws = new WebSocket('ws://localhost:3030/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('VM Update:', data);
};

// Subscribe to VM updates
ws.send(JSON.stringify({
  type: 'subscribe',
  vm_id: 'vm-uuid-here'
}));
🗂️ Project Structure
text
aegis-vm-manager/
├── backend/                 # Rust backend
│   ├── src/                # Source code
│   │   ├── api/            # HTTP/WebSocket API
│   │   ├── vm/             # VM management
│   │   ├── security/       # Security modules
│   │   ├── storage/        # ISO/disk management
│   │   └── utils/          # Utilities
│   └── Cargo.toml          # Rust dependencies
├── frontend/               # Web interface
│   ├── index.html          # Main dashboard
│   ├── css/                # Stylesheets
│   ├── js/                 # JavaScript modules
│   └── lib/noVNC/          # VNC client
├── config/                 # Configuration files
├── scripts/                # Setup scripts
├── systemd/                # Service files
└── docs/                   # Documentation
🚨 Troubleshooting
Common Issues
1. "KVM not available"
bash
# Enable virtualization in BIOS
# Check KVM modules
lsmod | grep kvm

# Load modules
sudo modprobe kvm
sudo modprobe kvm_intel  # Intel
# OR
sudo modprobe kvm_amd    # AMD
2. "Permission denied"
bash
# Verify group membership
groups $USER

# Add to groups
sudo usermod -aG kvm,libvirt $USER
newgrp kvm  # Apply group changes without logout
3. "Port already in use"
bash
# Find process using port
sudo lsof -i :3030
sudo lsof -i :6080

# Kill process or change port in config
4. "QEMU not found"
bash
# Install QEMU
sudo apt-get install qemu-system-x86 qemu-utils
5. "Cannot create network bridge"
bash
# Temporarily disable NetworkManager
sudo systemctl stop NetworkManager

# Or install bridge utilities
sudo apt-get install bridge-utils
Logs and Debugging
bash
# View service logs
sudo journalctl -u vm-manager -f

# Run in debug mode
RUST_LOG=debug vm-manager

# Check websockify logs
sudo journalctl -u vm-websockify -f
🔒 Security Considerations
Production Deployment
Enable Authentication:

Use reverse proxy (nginx/apache) with auth

Implement API key authentication

Network Security:

Change default ports

Use firewall rules

Enable TLS/HTTPS

VM Isolation:

Enable sandboxing in config

Use separate network namespace

Set resource limits

Regular Updates:

Keep system packages updated

Monitor security advisories

Configuration Security
toml
[security]
require_vnc_password = true
sandbox_vms = true
isolate_network = true
max_vms_per_user = 5
📊 Performance Tuning
For Low-End Hardware
Reduce VM overhead:

toml
[qemu]
enable_kvm = true
default_cpu = "host"
machine_type = "pc-i440fx-2.9"  # Lighter than Q35
Optimize disk:

bash
# Use qcow2 with compression
qemu-img create -f qcow2 -o compression_type=zstd disk.qcow2 20G
Memory management:

Enable KSM (Kernel Samepage Merging)

Use virtio drivers for better performance

Resource Limits
Edit /etc/vm-manager/default.toml:

toml
[limits]
max_vms = 5                    # Maximum concurrent VMs
max_memory_mb = 8192           # Total memory for all VMs
max_cpu_cores = 8              # Total CPU cores for all VMs
max_disk_gb = 200              # Total disk space
🧪 Testing
Run Tests
bash
# Unit tests
cd backend && cargo test

# Integration tests
cargo test --test integration

# E2E tests (requires running service)
cd tests/e2e && node basic_flow.js
Test VM Creation
bash
# Using curl to test API
curl -X POST http://localhost:3030/api/vms \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-vm",
    "iso_path": "/var/lib/vm-manager/isos/test.iso",
    "memory_mb": 512,
    "cpu_cores": 1,
    "disk_size_gb": 10
  }'
🤝 Contributing
Fork the repository

Create a feature branch

bash
git checkout -b feature/amazing-feature
Commit changes

bash
git commit -m 'Add amazing feature'
Push to branch

bash
git push origin feature/amazing-feature
Open a Pull Request

Development Setup
bash
# Clone and setup dev environment
git clone <your-fork-url>
cd aegis-vm-manager

# Install dev dependencies
make install-deps

# Build and run
make run

# Run tests
make test
📄 License
MIT License - see LICENSE file for details.

🙏 Acknowledgments
QEMU/KVM: Virtualization foundation

noVNC: HTML5 VNC client

Rust Community: Excellent libraries and tooling

Contributors: Everyone who helps improve this project

📞 Support
Issues: GitHub Issues

Discussions: GitHub Discussions

Wiki: Project Wiki

Quick Start Recap:

bash
git clone <repo-url>
cd aegis-vm-manager
sudo make setup
# Access: http://localhost:3030
Enjoy managing your VMs with Aegis! 🚀
