#!/bin/bash

# Minimal dependency installer for development

# For Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    qemu-kvm \
    qemu-utils \
    build-essential \
    pkg-config \
    libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install noVNC
mkdir -p frontend/lib
git clone https://github.com/novnc/noVNC.git frontend/lib/noVNC
cd frontend/lib/noVNC
git checkout v1.4.0
cd ../../..