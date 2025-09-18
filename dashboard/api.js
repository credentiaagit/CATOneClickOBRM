// API integration for Oracle BRM Rust backend
class BrmApi {
    constructor() {
        // Always use the backend port 3000, regardless of frontend port
        this.baseUrl = 'http://localhost:3000';
        this.setupRequestInterceptors();
    }

    setupRequestInterceptors() {
        // Add loading indicators and error handling
        this.isLoading = false;
    }

    async makeRequest(url, options = {}) {
        this.setLoading(true);
        
        try {
            const response = await fetch(url, {
                ...options,
                headers: {
                    'Content-Type': 'application/json',
                    ...options.headers
                }
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({ message: 'Network error' }));
                throw new Error(errorData.message || `HTTP ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error('API Error:', error);
            throw error;
        } finally {
            this.setLoading(false);
        }
    }

    async makeFormRequest(url, formData) {
        this.setLoading(true);
        
        try {
            const response = await fetch(url, {
                method: 'POST',
                body: formData // Don't set Content-Type for FormData
            });

            if (!response.ok) {
                const errorData = await response.text();
                throw new Error(errorData || `HTTP ${response.status}`);
            }

            // Handle different response types
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return await response.json();
            } else if (contentType && contentType.includes('application/zip')) {
                return await response.blob();
            } else {
                return await response.text();
            }
        } catch (error) {
            console.error('API Error:', error);
            throw error;
        } finally {
            this.setLoading(false);
        }
    }

    setLoading(loading) {
        this.isLoading = loading;
        // Update UI loading indicators
        const buttons = document.querySelectorAll('button[type="submit"]');
        buttons.forEach(btn => {
            if (loading) {
                btn.disabled = true;
                const originalText = btn.innerHTML;
                btn.dataset.originalText = originalText;
                btn.innerHTML = '<span class="loading-spinner"></span> Loading...';
            } else {
                btn.disabled = false;
                if (btn.dataset.originalText) {
                    btn.innerHTML = btn.dataset.originalText;
                    delete btn.dataset.originalText;
                }
            }
        });
    }

    // Health Check
    async healthCheck() {
        return await this.makeRequest(`${this.baseUrl}/health`);
    }

    // Session Management
    async registerSession(sessionId) {
        return await this.makeRequest(`${this.baseUrl}/register/${sessionId}`, {
            method: 'POST'
        });
    }

    async getSessionStatus(sessionId) {
        return await this.makeRequest(`${this.baseUrl}/status/${sessionId}`);
    }

    async unregisterSession(sessionId) {
        return await this.makeRequest(`${this.baseUrl}/unregister/${sessionId}`, {
            method: 'DELETE'
        });
    }

    // BRM Field Management
    async loadBrmFields(sessionId, fieldsData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/load_obrm_fields/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: fieldsData
        });
    }

    async getSessionFields(sessionId) {
        return await this.makeRequest(`${this.baseUrl}/obrm/get_session_fields/${sessionId}`);
    }

    // PODL Conversion
    async convertFieldSpecToPodl(sessionId, fieldSpecData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/convert_fld_spec_to_podl/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: fieldSpecData
        });
    }

    async convertClassSpecToPodl(sessionId, classSpecData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/convert_class_spec_to_podl/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: classSpecData
        });
    }

    // FLIST Conversion
    async convertFlistToCode(sessionId, flistData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/convert_flist2code/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: flistData
        });
    }

    async convertFlistToXml(sessionId, flistData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/convert_flist2xml/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: flistData
        });
    }

    async convertFlistToJson(sessionId, flistData) {
        return await this.makeRequest(`${this.baseUrl}/obrm/convert_flist2json/${sessionId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            body: flistData
        });
    }

    // Call Stack Visualization
    async visualizeCallStack(file, threshold) {
        const formData = new FormData();
        formData.append('file', file);
        if (threshold) {
            formData.append('threshold', threshold.toString());
        }

        return await this.makeFormRequest(`${this.baseUrl}/obrm/view_call_stack`, formData);
    }
}

// Global API instance
const api = new BrmApi();

// Health Check Function
async function performHealthCheck() {
    try {
        const result = await api.healthCheck();
        
        const outputContainer = document.getElementById('health-result');
        const outputContent = document.getElementById('health-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = JSON.stringify(result, null, 2);
        
        showNotification('Health check completed', 'success');
        
        // Update dashboard stats
        if (window.dashboard) {
            window.dashboard.addActivity('Health check performed');
        }
    } catch (error) {
        showNotification(`Health check failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('health-result');
        const outputContent = document.getElementById('health-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

// Session management is now automatic - sessions are created on login

// Helper function to get current session ID
function getCurrentSessionId() {
    return window.auth && window.auth.currentUser ? window.auth.currentUser.sessionId : null;
}

// BRM Field Management Functions
async function loadBrmFields(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const fieldsData = formData.get('fieldsData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.loadBrmFields(sessionId, fieldsData);
        
        const outputContainer = document.getElementById('load-fields-result');
        const outputContent = document.getElementById('load-fields-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = JSON.stringify(result, null, 2);
        
        showNotification('BRM fields loaded successfully', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Loaded fields for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Failed to load fields: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('load-fields-result');
        const outputContent = document.getElementById('load-fields-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

async function viewSessionFields() {
    const sessionId = getCurrentSessionId();
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.getSessionFields(sessionId);
        
        const outputContainer = document.getElementById('view-fields-result');
        const outputContent = document.getElementById('view-fields-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = JSON.stringify(result, null, 2);
        
        showNotification('Session fields retrieved', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Viewed fields for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Failed to retrieve fields: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('view-fields-result');
        const outputContent = document.getElementById('view-fields-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

// PODL Conversion Functions
async function convertFieldSpecToPodl(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const fieldSpecData = formData.get('fieldSpecData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.convertFieldSpecToPodl(sessionId, fieldSpecData);
        
        const outputContainer = document.getElementById('field-spec-podl-result');
        const outputContent = document.getElementById('field-spec-podl-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = result.podl_content || JSON.stringify(result, null, 2);
        
        showNotification('Field spec converted to PODL', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Converted field spec to PODL for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Conversion failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('field-spec-podl-result');
        const outputContent = document.getElementById('field-spec-podl-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

async function convertClassSpecToPodl(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const classSpecData = formData.get('classSpecData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.convertClassSpecToPodl(sessionId, classSpecData);
        
        const outputContainer = document.getElementById('class-spec-podl-result');
        const outputContent = document.getElementById('class-spec-podl-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = result.podl_content || JSON.stringify(result, null, 2);
        
        showNotification('Class spec converted to PODL', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Converted class spec to PODL for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Conversion failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('class-spec-podl-result');
        const outputContent = document.getElementById('class-spec-podl-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

// FLIST Conversion Functions
async function convertFlistToCode(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const flistData = formData.get('flistData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.convertFlistToCode(sessionId, flistData);
        
        const outputContainer = document.getElementById('flist-to-code-result');
        const outputContent = document.getElementById('flist-to-code-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = result.podl_content || result;
        
        showNotification('FLIST converted to C code', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Converted FLIST to C code for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Conversion failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('flist-to-code-result');
        const outputContent = document.getElementById('flist-to-code-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

async function convertFlistToXml(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const flistData = formData.get('flistData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.convertFlistToXml(sessionId, flistData);
        
        const outputContainer = document.getElementById('flist-to-xml-result');
        const outputContent = document.getElementById('flist-to-xml-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = result.podl_content || result;
        
        showNotification('FLIST converted to XML', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Converted FLIST to XML for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Conversion failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('flist-to-xml-result');
        const outputContent = document.getElementById('flist-to-xml-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

async function convertFlistToJson(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const sessionId = getCurrentSessionId();
    const flistData = formData.get('flistData');
    
    if (!sessionId) {
        showNotification('No active session found. Please log in again.', 'error');
        return;
    }
    
    try {
        const result = await api.convertFlistToJson(sessionId, flistData);
        
        const outputContainer = document.getElementById('flist-to-json-result');
        const outputContent = document.getElementById('flist-to-json-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = result.podl_content || result;
        
        showNotification('FLIST converted to JSON', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Converted FLIST to JSON for session ${sessionId}`);
        }
    } catch (error) {
        showNotification(`Conversion failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('flist-to-json-result');
        const outputContent = document.getElementById('flist-to-json-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

// Call Stack Visualization Function
async function visualizeCallStack(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const file = formData.get('file');
    const threshold = formData.get('threshold');
    
    if (!file) {
        showNotification('Please select a ZIP file', 'error');
        return;
    }
    
    try {
        const result = await api.visualizeCallStack(file, threshold);
        
        const outputContainer = document.getElementById('call-stack-result');
        const outputContent = document.getElementById('call-stack-output');
        
        outputContainer.style.display = 'block';
        
        if (result instanceof Blob) {
            // Handle ZIP file download
            const url = window.URL.createObjectURL(result);
            const a = document.createElement('a');
            a.href = url;
            a.download = `callstack_analysis_${Date.now()}.zip`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
            
            outputContent.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <i class="fas fa-download" style="font-size: 3rem; color: #28a745; margin-bottom: 15px;"></i>
                    <h3>Analysis Complete!</h3>
                    <p>Your call stack analysis has been downloaded as a ZIP file.</p>
                    <p>Extract the ZIP and open the HTML file in your browser to view the interactive visualization.</p>
                </div>
            `;
            
            showNotification('Call stack analysis downloaded successfully', 'success');
        } else {
            outputContent.textContent = typeof result === 'string' ? result : JSON.stringify(result, null, 2);
            showNotification('Call stack processed', 'success');
        }
        
        if (window.dashboard) {
            window.dashboard.addActivity('Generated call stack visualization');
        }
    } catch (error) {
        showNotification(`Visualization failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('call-stack-result');
        const outputContent = document.getElementById('call-stack-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    }
}

// Export API instance for global use
window.brmApi = api;
