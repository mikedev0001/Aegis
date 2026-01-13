FROM rust:1.70-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    qemu-utils \
    libvirt-clients \
    bridge-utils \
    iproute2 \
    iptables \
    websockify \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 vmuser && \
    usermod -aG kvm vmuser

# Create directories
RUN mkdir -p /var/lib/vm-manager/{isos,disks,configs,logs} && \
    chown -R vmuser:vmuser /var/lib/vm-manager

# Copy binary from builder
COPY --from=builder /app/target/release/vm-manager /usr/local/bin/vm-manager

# Copy frontend
COPY frontend /opt/vm-manager/frontend
RUN chown -R vmuser:vmuser /opt/vm-manager

# Copy configuration
COPY config /etc/vm-manager

# Set working directory
WORKDIR /var/lib/vm-manager

# Switch to non-root user
USER vmuser

# Expose ports
EXPOSE 3030 6080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3030/api/health || exit 1

# Entrypoint
ENTRYPOINT ["vm-manager"]