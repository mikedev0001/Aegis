🛡️ Aegis VM Manager

A lightweight, web-based virtual machine manager with direct QEMU/KVM control.
Designed for performance, security, and simplicity — no libvirt overhead.








✨ Features

🌐 Web-based VM management dashboard

⚙️ Direct QEMU/KVM control (no libvirt)

🖥️ Browser-based VNC console (noVNC)

🔒 Strong VM isolation with Linux namespaces

🧠 Low resource usage (works on low-end hardware)

📊 Real-time VM status & WebSocket updates

VM Management

Create VMs from ISO files

Configure CPU, RAM, and disk size

Start / Stop / Delete VMs

Web console access

Security

Process sandboxing

Input validation & sanitization

Command injection protection

Resource & network isolation

📋 Requirements
Hardware

x86_64 CPU with Intel VT-x / AMD-V

2 GB RAM minimum (4 GB recommended)

20 GB free disk space

Software

Linux (Ubuntu 20.04+, Debian 11+, Fedora 34+, Arch)

QEMU 5.0+

KVM enabled

Rust 1.70+

Root access (setup only)

🚀 Quick Install (Recommended)
git clone https://github.com/yourusername/aegis-vm-manager.git
cd aegis-vm-manager
sudo make setup


Access the dashboard:
👉 http://localhost:3030

🖥️ Usage
Create Your First VM

Open the dashboard

Click Create VM

Select an ISO

Configure CPU / RAM / Disk

Click Create & Start

Open the Console to install the OS

⚙️ Configuration

Main config file:

/etc/vm-manager/default.toml


Example:

[server]
host = "127.0.0.1"
port = 3030

[qemu]
enable_kvm = true

[limits]
max_vms = 10
max_memory_mb = 32768
max_cpu_cores = 16

📡 API
REST Endpoints
Method	Endpoint	Description
GET	/api/health	Health check
GET	/api/vms	List VMs
POST	/api/vms	Create VM
POST	/api/vms/{id}/start	Start VM
POST	/api/vms/{id}/stop	Stop VM
DELETE	/api/vms/{id}	Delete VM
WebSocket
ws://localhost:3030/ws

🗂️ Project Structure
aegis-vm-manager/
├── backend/     # Rust backend
├── frontend/    # Web UI
├── config/      # Configuration
├── scripts/     # Setup scripts
├── systemd/     # Services
└── docs/        # Documentation

🚨 Troubleshooting

KVM not available

lsmod | grep kvm
sudo modprobe kvm_intel   # or kvm_amd


Permission denied

sudo usermod -aG kvm,libvirt $USER
logout


View logs

sudo journalctl -u vm-manager -f

🔒 Security Notes

Use HTTPS via reverse proxy (Nginx/Apache)

Change default ports

Apply firewall rules

Enforce VM limits

Keep system updated

🤝 Contributing

Fork the repository

Create a feature branch

Commit your changes

Open a Pull Request

📄 License

MIT License — see LICENSE

🙏 Acknowledgments

QEMU / KVM

noVNC

Rust community

⚡ Quick Start Recap
git clone <repo-url>
cd aegis-vm-manager
sudo make setup


👉 http://localhost:3030

Enjoy managing your VMs with Aegis 🚀
