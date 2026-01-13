#!/bin/bash

set -e

echo "=== Lightweight VM Manager Setup ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root"
    exit 1
fi

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "Cannot detect OS"
    exit 1
fi

echo "Detected OS: $OS"

# Install dependencies
echo "Installing dependencies..."
case $OS in
    ubuntu|debian)
        apt-get update
        apt-get install -y \
            qemu-kvm \
            qemu-utils \
            libvirt-daemon-system \
            libvirt-clients \
            bridge-utils \
            virt-manager \
            websockify \
            build-essential \
            pkg-config \
            libssl-dev
        ;;
    fedora|centos|rhel)
        dnf install -y \
            qemu-kvm \
            qemu-img \
            libvirt \
            virt-install \
            virt-manager \
            bridge-utils \
            websockify \
            gcc \
            make \
            openssl-devel
        ;;
    arch)
        pacman -Syu --noconfirm \
            qemu \
            libvirt \
            virt-manager \
            bridge-utils \
            websockify \
            base-devel \
            openssl
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Enable services
echo "Enabling services..."
systemctl enable libvirtd
systemctl start libvirtd

# Add user to kvm group
echo "Adding user to kvm group..."
usermod -aG kvm $SUDO_USER
usermod -aG libvirt $SUDO_USER

# Create data directories
echo "Creating data directories..."
mkdir -p /var/lib/vm-manager/{isos,disks,configs,logs}
chown -R $SUDO_USER:$SUDO_USER /var/lib/vm-manager
chmod 755 /var/lib/vm-manager

# Install Rust if not present
echo "Checking for Rust..."
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Build backend
echo "Building backend..."
cd backend
cargo build --release
cp target/release/vm-manager /usr/local/bin/
cd ..

# Install frontend
echo "Installing frontend..."
mkdir -p /opt/vm-manager/frontend
cp -r frontend/* /opt/vm-manager/frontend/

# Install systemd service
echo "Installing systemd service..."
cat > /etc/systemd/system/vm-manager.service << EOF
[Unit]
Description=Lightweight VM Manager
After=network.target libvirtd.service

[Service]
Type=simple
User=$SUDO_USER
Group=$SUDO_USER
WorkingDirectory=/opt/vm-manager
ExecStart=/usr/local/bin/vm-manager
Restart=on-failure
RestartSec=5

# Security
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/vm-manager

[Install]
WantedBy=multi-user.target
EOF

# Install websockify service for VNC
cat > /etc/systemd/system/vm-websockify.service << EOF
[Unit]
Description=Websockify VNC Proxy
After=network.target

[Service]
Type=simple
User=$SUDO_USER
Group=$SUDO_USER
ExecStart=/usr/bin/websockify --web /opt/vm-manager/frontend/lib/noVNC 6080 localhost:5900-5999
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start services
systemctl daemon-reload
systemctl enable vm-manager vm-websockify
systemctl start vm-manager vm-websockify

# Configure firewall
echo "Configuring firewall..."
if command -v ufw &> /dev/null; then
    ufw allow 3030/tcp  # Web UI
    ufw allow 6080/tcp  # VNC websocket
    ufw reload
elif command -v firewall-cmd &> /dev/null; then
    firewall-cmd --permanent --add-port=3030/tcp
    firewall-cmd --permanent --add-port=6080/tcp
    firewall-cmd --reload
fi

echo "=== Setup Complete ==="
echo ""
echo "VM Manager is now running!"
echo "Access the web interface at: http://$(hostname -I | awk '{print $1}'):3030"
echo ""
echo "To stop the service: systemctl stop vm-manager"
echo "To start the service: systemctl start vm-manager"
echo "To view logs: journalctl -u vm-manager -f"