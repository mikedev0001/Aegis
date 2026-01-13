🛡️ Aegis VM Manager

A lightweight, web-based virtual machine manager with direct QEMU/KVM control.
Built for performance, security, and simplicity—without libvirt overhead.








🚀 Features
Core

🌐 Web-Based Dashboard – Clean and minimal UI

⚙️ Direct QEMU/KVM Control – No libvirt, maximum performance

🖥️ Browser VNC Console – Embedded noVNC

🔒 Strong VM Isolation – Namespace-based sandboxing

🧠 Low Resource Usage – Designed for low-end hardware

VM Management

Create VMs from ISO files (local or uploaded)

Configure RAM, CPU cores, and disk size

Start / Stop / Delete VMs

Real-time status monitoring

Console access via WebSocket

Security

Process sandboxing using Linux namespaces

Strict input validation & sanitization

Command injection prevention

Resource limits enforcement

Network isolation

📋 Requirements
Hardware

x86_64 CPU with virtualization support (Intel VT-x / AMD-V)

Minimum: 2 GB RAM

Recommended: 4 GB+ RAM

20 GB+ free disk space

Software

OS: Linux (Ubuntu 20.04+, Debian 11+, Fedora 34+, Arch)

KVM: Enabled kernel virtualization

QEMU: v5.0+

Rust: v1.70+ (backend build)

Root Access: Required for setup

🛠️ Installation
🚀 One-Command Install (Recommended)
git clone https://github.com/yourusername/aegis-vm-manager.git
cd aegis-vm-manager
sudo make setup


Then open:
👉 http://localhost:3030

🔧 Manual Installation
1. Check virtualization support
grep -E -c '(vmx|svm)' /proc/cpuinfo
sudo apt install cpu-checker && kvm-ok

2. Install dependencies
sudo apt update
sudo apt install -y \
  qemu-kvm qemu-utils libvirt-daemon-system \
  bridge-utils websockify build-essential libssl-dev

3. Add user to groups
sudo usermod -aG kvm,libvirt $USER
logout

4. Build backend
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

cd backend
cargo build --release
sudo cp target/release/vm-manager /usr/local/bin/

5. Setup directories & service
sudo mkdir -p /var/lib/vm-manager/{isos,disks,configs,logs}
sudo chown -R $USER:$USER /var/lib/vm-manager

sudo cp systemd/vm-manager.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now vm-manager

📖 Usage Guide
Creating Your First VM
1. Download an ISO
wget https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/x86_64/alpine-standard-3.18.0-x86_64.iso \
  -P /var/lib/vm-manager/isos/

2. Open Dashboard

👉 http://localhost:3030

3. Create VM

Name: my-first-vm

ISO Path: /var/lib/vm-manager/isos/alpine-standard-3.18.0-x86_64.iso

Memory: 1024 MB

CPU: 2 cores

Disk: 10 GB

Click Create & Start

4. Access Console

Wait for status: Running

Click Console

Install OS directly from browser

⚙️ Configuration
Main Config File

📄 /etc/vm-manager/default.toml

[server]
host = "127.0.0.1"
port = 3030

[qemu]
path = "/usr/bin/qemu-system-x86_64"
enable_kvm = true

[limits]
max_vms = 10
max_memory_mb = 32768
max_cpu_cores = 16

Environment Variables
cp .env.example .env


SERVER_HOST

SERVER_PORT

DATA_DIR

LOG_LEVEL

📡 API Reference
REST API
Method	Endpoint	Description
GET	/api/health	Health check
GET	/api/vms	List VMs
POST	/api/vms	Create VM
POST	/api/vms/{id}/start	Start VM
POST	/api/vms/{id}/stop	Stop VM
DELETE	/api/vms/{id}	Delete VM
WebSocket
const ws = new WebSocket('ws://localhost:3030/ws');
ws.onmessage = e => console.log(JSON.parse(e.data));

🗂️ Project Structure
aegis-vm-manager/
├── backend/      # Rust backend
├── frontend/     # Web UI
├── config/       # Config files
├── scripts/      # Setup scripts
├── systemd/      # Services
└── docs/         # Documentation

🚨 Troubleshooting
KVM Not Available
lsmod | grep kvm
sudo modprobe kvm kvm_intel   # or kvm_amd

Permission Denied
groups $USER
sudo usermod -aG kvm,libvirt $USER

Port in Use
sudo lsof -i :3030

Logs
sudo journalctl -u vm-manager -f
RUST_LOG=debug vm-manager

🔒 Security Recommendations

Use reverse proxy (Nginx) with authentication

Enable TLS / HTTPS

Change default ports

Apply firewall rules

Enforce VM resource limits

Keep system updated

📊 Performance Tuning
[qemu]
default_cpu = "host"
machine_type = "pc-i440fx-2.9"

qemu-img create -f qcow2 -o compression_type=zstd disk.qcow2 20G

🧪 Testing
cd backend
cargo test

curl -X POST http://localhost:3030/api/vms \
  -H "Content-Type: application/json" \
  -d '{"name":"test-vm","memory_mb":512,"cpu_cores":1,"disk_size_gb":10}'

🤝 Contributing

Fork the repo

Create a branch

Commit changes

Open a Pull Request

📄 License

MIT License — see LICENSE

🙏 Acknowledgments

QEMU / KVM

noVNC

Rust Community

All contributors ❤️

⚡ Quick Start
git clone <repo-url>
cd aegis-vm-manager
sudo make setup


👉 http://localhost:3030

Enjoy managing your VMs with Aegis! 🚀
