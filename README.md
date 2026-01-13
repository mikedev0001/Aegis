# 🛡️ Aegis VM Manager

A **lightweight, web-based virtual machine manager** with **direct QEMU/KVM control**.  
Designed for **performance**, **security**, and **simplicity** — no libvirt overhead.

![Status](https://img.shields.io/badge/Status-Beta-orange)
![Backend](https://img.shields.io/badge/Backend-Rust-red)
![Frontend](https://img.shields.io/badge/Frontend-Vanilla%20JS-yellow)
![License](https://img.shields.io/badge/License-MIT-green)

---

## ✨ Features

- 🌐 Web-based VM management dashboard
- ⚙️ Direct QEMU/KVM control (no libvirt)
- 🖥️ Browser-based VNC console (noVNC)
- 🔒 Strong VM isolation with Linux namespaces
- 🧠 Low resource usage (works on low-end hardware)
- 📊 Real-time VM status & WebSocket updates

### VM Management
- Create VMs from ISO files
- Configure CPU, RAM, and disk size
- Start / Stop / Delete VMs
- Web console access

### Security
- Process sandboxing
- Input validation & sanitization
- Command injection protection
- Resource & network isolation

---

## 📋 Requirements

### Hardware
- x86_64 CPU with Intel VT-x / AMD-V
- 2 GB RAM minimum (4 GB recommended)
- 20 GB free disk space

### Software
- Linux (Ubuntu 20.04+, Debian 11+, Fedora 34+, Arch)
- QEMU 5.0+
- KVM enabled
- Rust 1.70+
- Root access (setup only)

---

## 🚀 Quick Install (Recommended)



## Warning 
this current version is beta so many functions will not work

```bash
git clone https://github.com/yourusername/aegis-vm-manager.git
cd aegis-vm-manager
sudo make setup
