class VMApi {
    constructor(baseUrl = 'http://127.0.0.1:3030/api') {
        this.baseUrl = baseUrl;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const defaultOptions = {
            headers: {
                'Content-Type': 'application/json',
            },
        };

        const response = await fetch(url, { ...defaultOptions, ...options });
        
        if (!response.ok) {
            let errorMsg = `HTTP ${response.status}`;
            try {
                const errorData = await response.json();
                errorMsg = errorData.error || errorMsg;
            } catch (e) {
                // Ignore JSON parsing errors
            }
            throw new Error(errorMsg);
        }

        return response.json();
    }

    async listVMs() {
        return this.request('/vms');
    }

    async createVM(config) {
        return this.request('/vms', {
            method: 'POST',
            body: JSON.stringify(config),
        });
    }

    async startVM(vmId) {
        return this.request(`/vms/${vmId}/start`, {
            method: 'POST',
        });
    }

    async stopVM(vmId) {
        return this.request(`/vms/${vmId}/stop`, {
            method: 'POST',
        });
    }

    async deleteVM(vmId) {
        return this.request(`/vms/${vmId}`, {
            method: 'DELETE',
        });
    }

    async getVNCUrl(vmId) {
        return this.request(`/vms/${vmId}/vnc`);
    }

    async uploadISO(file) {
        const formData = new FormData();
        formData.append('iso', file);
        
        const response = await fetch(`${this.baseUrl}/isos/upload`, {
            method: 'POST',
            body: formData,
        });
        
        if (!response.ok) {
            throw new Error(`Upload failed: ${response.status}`);
        }
        
        return response.json();
    }
}