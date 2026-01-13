class VMManagerApp {
    constructor() {
        this.api = new VMApi();
        this.currentVms = [];
        this.init();
    }

    init() {
        this.bindEvents();
        this.loadVMs();
        this.setupAutoRefresh();
    }

    bindEvents() {
        // Create VM modal
        document.getElementById('createVmBtn').addEventListener('click', () => {
            this.showCreateVmModal();
        });

        // Close modal buttons
        document.querySelectorAll('.close-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const modal = e.target.closest('.modal');
                if (modal) {
                    this.hideModal(modal.id);
                }
            });
        });

        // Browse ISO button
        document.getElementById('browseIsoBtn').addEventListener('click', () => {
            document.getElementById('isoUpload').click();
        });

        // ISO file upload
        document.getElementById('isoUpload').addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                // For now, just show the filename
                // In a real app, you would upload this to the server
                document.getElementById('isoPath').value = file.name;
            }
        });

        // VM form submission
        document.getElementById('vmForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.createVM();
        });

        // Refresh button
        document.getElementById('refreshBtn').addEventListener('click', () => {
            this.loadVMs();
        });

        // Close modals on escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                this.hideAllModals();
            }
        });

        // Close modals on background click
        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', (e) => {
                if (e.target === modal) {
                    this.hideModal(modal.id);
                }
            });
        });
    }

    async loadVMs() {
        try {
            const vms = await this.api.listVMs();
            this.currentVms = vms;
            this.renderVMList(vms);
        } catch (error) {
            this.showError('Failed to load VMs: ' + error.message);
        }
    }

    renderVMList(vms) {
        const container = document.getElementById('vmList');
        
        if (vms.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-server fa-3x"></i>
                    <h3>No Virtual Machines</h3>
                    <p>Create your first VM to get started</p>
                    <button id="createFirstVmBtn" class="btn btn-primary">
                        <i class="fas fa-plus"></i> Create VM
                    </button>
                </div>
            `;
            
            document.getElementById('createFirstVmBtn')?.addEventListener('click', () => {
                this.showCreateVmModal();
            });
            return;
        }

        container.innerHTML = vms.map(vm => this.createVMCard(vm)).join('');
        
        // Bind action buttons
        vms.forEach(vm => {
            const startBtn = document.getElementById(`start-${vm.id}`);
            const stopBtn = document.getElementById(`stop-${vm.id}`);
            const deleteBtn = document.getElementById(`delete-${vm.id}`);
            const consoleBtn = document.getElementById(`console-${vm.id}`);

            if (startBtn) {
                startBtn.addEventListener('click', () => this.startVM(vm.id));
            }
            if (stopBtn) {
                stopBtn.addEventListener('click', () => this.stopVM(vm.id));
            }
            if (deleteBtn) {
                deleteBtn.addEventListener('click', () => this.deleteVM(vm.id));
            }
            if (consoleBtn) {
                consoleBtn.addEventListener('click', () => this.openConsole(vm));
            }
        });
    }

    createVMCard(vm) {
        const statusClass = this.getStatusClass(vm.state);
        const statusText = this.getStatusText(vm.state);
        const actions = this.getVMActions(vm);
        
        return `
            <div class="vm-card" id="vm-${vm.id}">
                <div class="vm-card-header">
                    <div class="vm-name">${vm.name}</div>
                    <div class="vm-status ${statusClass}">${statusText}</div>
                </div>
                
                <div class="vm-details">
                    <div class="vm-detail">
                        <i class="fas fa-microchip"></i>
                        <span>${vm.cpu_cores || 2} vCPU</span>
                    </div>
                    <div class="vm-detail">
                        <i class="fas fa-memory"></i>
                        <span>${vm.memory_mb || 0} MB RAM</span>
                    </div>
                    <div class="vm-detail">
                        <i class="fas fa-hdd"></i>
                        <span>${vm.disk_size_gb || 0} GB Disk</span>
                    </div>
                    <div class="vm-detail">
                        <i class="fas fa-network-wired"></i>
                        <span>VNC: ${vm.vnc_port || 'N/A'}</span>
                    </div>
                    ${vm.pid ? `
                    <div class="vm-detail">
                        <i class="fas fa-clock"></i>
                        <span>Uptime: ${this.formatUptime(vm.uptime_seconds)}</span>
                    </div>
                    ` : ''}
                </div>
                
                <div class="vm-actions">
                    ${actions}
                </div>
            </div>
        `;
    }

    getStatusClass(state) {
        if (typeof state === 'string') {
            state = state.toLowerCase();
        }
        
        if (state === 'running' || state === 'r') {
            return 'status-running';
        } else if (state === 'stopped' || state === 's') {
            return 'status-stopped';
        } else if (state === 'starting') {
            return 'status-starting';
        }
        return 'status-stopped';
    }

    getStatusText(state) {
        if (typeof state === 'string') {
            return state;
        } else if (typeof state === 'object' && state.state) {
            return state.state;
        }
        return 'unknown';
    }

    getVMActions(vm) {
        const state = this.getStatusText(vm.state).toLowerCase();
        let actions = '';
        
        if (state === 'stopped' || state === 'error') {
            actions += `
                <button id="start-${vm.id}" class="btn btn-success btn-small">
                    <i class="fas fa-play"></i> Start
                </button>
            `;
        } else if (state === 'running') {
            actions += `
                <button id="stop-${vm.id}" class="btn btn-warning btn-small">
                    <i class="fas fa-stop"></i> Stop
                </button>
                <button id="console-${vm.id}" class="btn btn-primary btn-small">
                    <i class="fas fa-terminal"></i> Console
                </button>
            `;
        } else if (state === 'starting') {
            actions += `
                <button class="btn btn-secondary btn-small" disabled>
                    <i class="fas fa-spinner fa-spin"></i> Starting
                </button>
            `;
        }
        
        // Always show delete button
        actions += `
            <button id="delete-${vm.id}" class="btn btn-danger btn-small">
                <i class="fas fa-trash"></i> Delete
            </button>
        `;
        
        return actions;
    }

    formatUptime(seconds) {
        if (!seconds) return '0s';
        
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        } else if (minutes > 0) {
            return `${minutes}m ${secs}s`;
        } else {
            return `${secs}s`;
        }
    }

    async createVM() {
        const form = document.getElementById('vmForm');
        const formData = {
            name: document.getElementById('vmName').value,
            iso_path: document.getElementById('isoPath').value,
            memory_mb: parseInt(document.getElementById('memory').value),
            cpu_cores: parseInt(document.getElementById('cpu').value),
            disk_size_gb: parseInt(document.getElementById('disk').value)
        };

        try {
            const result = await this.api.createVM(formData);
            this.hideModal('createVmModal');
            this.showSuccess('VM created successfully');
            this.loadVMs();
            
            // Auto-start the VM
            if (result.id) {
                setTimeout(() => this.startVM(result.id), 1000);
            }
        } catch (error) {
            this.showError('Failed to create VM: ' + error.message);
        }
    }

    async startVM(vmId) {
        try {
            await this.api.startVM(vmId);
            this.showSuccess('VM starting...');
            
            // Update UI immediately
            const vm = this.currentVms.find(v => v.id === vmId);
            if (vm) {
                vm.state = 'Starting';
                this.renderVMList(this.currentVms);
            }
            
            // Refresh after delay
            setTimeout(() => this.loadVMs(), 3000);
        } catch (error) {
            this.showError('Failed to start VM: ' + error.message);
        }
    }

    async stopVM(vmId) {
        if (!confirm('Are you sure you want to stop this VM?')) {
            return;
        }

        try {
            await this.api.stopVM(vmId);
            this.showSuccess('VM stopping...');
            
            // Update UI
            const vm = this.currentVms.find(v => v.id === vmId);
            if (vm) {
                vm.state = 'Stopping';
                this.renderVMList(this.currentVms);
            }
            
            setTimeout(() => this.loadVMs(), 2000);
        } catch (error) {
            this.showError('Failed to stop VM: ' + error.message);
        }
    }

    async deleteVM(vmId) {
        if (!confirm('Are you sure you want to delete this VM? This cannot be undone.')) {
            return;
        }

        try {
            await this.api.deleteVM(vmId);
            this.showSuccess('VM deleted');
            this.loadVMs();
        } catch (error) {
            this.showError('Failed to delete VM: ' + error.message);
        }
    }

    async openConsole(vm) {
        if (vm.state !== 'running' && vm.state !== 'Running') {
            this.showError('VM must be running to open console');
            return;
        }

        try {
            const consoleUrl = await this.api.getVNCUrl(vm.id);
            document.getElementById('consoleTitle').textContent = `Console - ${vm.name}`;
            this.showModal('consoleModal');
            
            // Initialize VNC connection
            window.vmConsole = new VMConsole(vm.id, consoleUrl.url);
            window.vmConsole.connect();
        } catch (error) {
            this.showError('Failed to open console: ' + error.message);
        }
    }

    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.add('active');
            document.body.style.overflow = 'hidden';
        }
    }

    hideModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.remove('active');
            document.body.style.overflow = '';
            
            if (modalId === 'consoleModal' && window.vmConsole) {
                window.vmConsole.disconnect();
            }
        }
    }

    hideAllModals() {
        document.querySelectorAll('.modal.active').forEach(modal => {
            modal.classList.remove('active');
        });
        document.body.style.overflow = '';
        
        if (window.vmConsole) {
            window.vmConsole.disconnect();
        }
    }

    showCreateVmModal() {
        document.getElementById('vmForm').reset();
        this.showModal('createVmModal');
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showNotification(message, type = 'info') {
        // Remove existing notifications
        const existing = document.querySelector('.notification');
        if (existing) {
            existing.remove();
        }

        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <i class="fas fa-${type === 'success' ? 'check-circle' : 'exclamation-circle'}"></i>
                <span>${message}</span>
            </div>
            <button class="notification-close">&times;</button>
        `;

        document.body.appendChild(notification);

        // Add styles if not present
        if (!document.querySelector('#notification-styles')) {
            const style = document.createElement('style');
            style.id = 'notification-styles';
            style.textContent = `
                .notification {
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    padding: 15px 20px;
                    border-radius: 6px;
                    background: white;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    min-width: 300px;
                    max-width: 400px;
                    z-index: 9999;
                    animation: slideIn 0.3s ease;
                }
                .notification-success {
                    border-left: 4px solid var(--success-color);
                }
                .notification-error {
                    border-left: 4px solid var(--danger-color);
                }
                .notification-content {
                    display: flex;
                    align-items: center;
                    gap: 10px;
                }
                .notification-close {
                    background: none;
                    border: none;
                    font-size: 20px;
                    cursor: pointer;
                    color: var(--text-secondary);
                }
                @keyframes slideIn {
                    from { transform: translateX(100%); opacity: 0; }
                    to { transform: translateX(0); opacity: 1; }
                }
            `;
            document.head.appendChild(style);
        }

        // Auto-remove after 5 seconds
        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, 5000);

        // Close button
        notification.querySelector('.notification-close').addEventListener('click', () => {
            notification.remove();
        });
    }

    setupAutoRefresh() {
        // Refresh every 10 seconds
        setInterval(() => {
            this.loadVMs();
        }, 10000);
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new VMManagerApp();
});