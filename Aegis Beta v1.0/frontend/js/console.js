class VMConsole {
    constructor(vmId, vncUrl) {
        this.vmId = vmId;
        this.vncUrl = vncUrl;
        this.rfb = null;
        this.container = document.getElementById('noVncContainer');
        this.statusElement = document.getElementById('consoleStatus');
    }

    connect() {
        try {
            this.statusElement.style.display = 'flex';
            this.container.innerHTML = '';
            
            // Create canvas for VNC
            const canvas = document.createElement('canvas');
            canvas.id = 'noVnc-canvas';
            this.container.appendChild(canvas);
            
            // Initialize RFB
            this.rfb = new RFB(canvas, this.vncUrl, {
                credentials: {
                    password: ''
                }
            });
            
            this.rfb.addEventListener("connect", () => {
                console.log('VNC connected');
                this.statusElement.style.display = 'none';
            });
            
            this.rfb.addEventListener("disconnect", (e) => {
                console.log('VNC disconnected:', e.detail);
                if (!e.detail.clean) {
                    this.statusElement.innerHTML = `
                        <div>
                            <i class="fas fa-exclamation-triangle"></i>
                            <div>Console disconnected</div>
                            <button id="reconnectBtn" class="btn btn-primary" style="margin-top: 10px;">
                                Reconnect
                            </button>
                        </div>
                    `;
                    this.statusElement.style.display = 'flex';
                    
                    document.getElementById('reconnectBtn').addEventListener('click', () => {
                        this.connect();
                    });
                }
            });
            
            this.rfb.addEventListener("credentialsrequired", () => {
                const password = prompt("Enter VNC password (if any):");
                if (password !== null) {
                    this.rfb.sendCredentials({ password: password });
                }
            });
            
            // Bind Ctrl+Alt+Del
            document.getElementById('sendCtrlAltDel').addEventListener('click', () => {
                this.sendCtrlAltDel();
            });
            
        } catch (error) {
            console.error('Failed to initialize VNC:', error);
            this.statusElement.innerHTML = `
                <div>
                    <i class="fas fa-exclamation-triangle"></i>
                    <div>Failed to connect: ${error.message}</div>
                </div>
            `;
        }
    }

    sendCtrlAltDel() {
        if (this.rfb) {
            // RFB doesn't have a direct method for Ctrl+Alt+Del
            // We need to send the key combination
            const keys = [
                { code: 'ControlLeft', down: true },
                { code: 'AltLeft', down: true },
                { code: 'Delete', down: true },
                { code: 'Delete', down: false },
                { code: 'AltLeft', down: false },
                { code: 'ControlLeft', down: false }
            ];
            
            keys.forEach(key => {
                const event = {
                    type: key.down ? 'keydown' : 'keyup',
                    code: key.code
                };
                this.rfb.handleKeyEvent(event);
            });
        }
    }

    disconnect() {
        if (this.rfb) {
            this.rfb.disconnect();
            this.rfb = null;
        }
        this.container.innerHTML = '';
        this.statusElement.style.display = 'none';
    }
}