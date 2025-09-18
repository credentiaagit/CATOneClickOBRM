// Dashboard functionality
class Dashboard {
    constructor() {
        this.currentPage = 'overview';
        this.sidebarOpen = false;
        this.sessions = new Map();
        this.activityLog = [];
        this.initialize();
    }

    initialize() {
        this.createPages();
        this.setupEventListeners();
        this.startHealthCheck();
        this.updateStats();
        this.loadRecentActivity();
        
        // Update session displays after a short delay to ensure auth is loaded
        setTimeout(() => {
            this.updateSessionDisplays();
        }, 500);
    }

    // Setup event listeners
    setupEventListeners() {
        // Sidebar toggle for mobile
        window.addEventListener('resize', () => {
            if (window.innerWidth > 1024) {
                this.closeSidebar();
            }
        });

        // User dropdown
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.user-dropdown')) {
                this.closeUserDropdown();
            }
        });
    }

    // Get current session ID from auth
    getCurrentSessionId() {
        const sessionId = window.auth && window.auth.currentUser ? window.auth.currentUser.sessionId : null;
        console.log('getCurrentSessionId called, returning:', sessionId);
        return sessionId;
    }

    // Create all page content
    createPages() {
        const pages = {
            'health': this.createHealthPage(),
            'session-info': this.createSessionInfoPage(),
            'load-fields': this.createLoadFieldsPage(),
            'view-fields': this.createViewFieldsPage(),
            'field-spec-podl': this.createFieldSpecPodlPage(),
            'class-spec-podl': this.createClassSpecPodlPage(),
            'flist-to-code': this.createFlistToCodePage(),
            'flist-to-xml': this.createFlistToXmlPage(),
            'flist-to-json': this.createFlistToJsonPage(),
            'call-stack': this.createCallStackPage()
        };

        const pageContent = document.getElementById('page-content');
        
        Object.entries(pages).forEach(([id, content]) => {
            const pageDiv = document.createElement('div');
            pageDiv.id = `${id}-page`;
            pageDiv.className = 'page';
            pageDiv.innerHTML = content;
            pageContent.appendChild(pageDiv);
        });
    }

    // Health Check Page
    createHealthPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-heartbeat"></i> Health Check</h2>
                <p>Monitor server health and connectivity</p>
            </div>
            
            <div class="section">
                <div class="form-group">
                    <button id="health-check-btn" class="btn btn-primary" onclick="performHealthCheck()">
                        <i class="fas fa-heartbeat"></i> Check Server Health
                    </button>
                </div>
                
                <div id="health-result" class="output-container" style="display: none;">
                    <h3>Health Check Result</h3>
                    <div id="health-output" class="output-content"></div>
                </div>
            </div>
        `;
    }

    // Session Info Page
    createSessionInfoPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-info-circle"></i> Current Session Information</h2>
                <p>View your current BRM session details and status</p>
            </div>
            
            <div class="section">
                <div class="form-row">
                    <div class="form-group">
                        <label>Session ID</label>
                        <input type="text" id="current-session-id" readonly class="session-display">
                    </div>
                    <div class="form-group">
                        <label>Username</label>
                        <input type="text" id="current-username" readonly class="session-display">
                    </div>
                </div>
                
                <div class="form-row">
                    <div class="form-group">
                        <label>Login Time</label>
                        <input type="text" id="current-login-time" readonly class="session-display">
                    </div>
                    <div class="form-group">
                        <label>Session Status</label>
                        <input type="text" id="current-session-status" readonly class="session-display">
                    </div>
                </div>
                
                <div class="form-actions">
                    <button type="button" class="btn btn-primary" onclick="checkCurrentSessionStatus()">
                        <i class="fas fa-sync"></i> Refresh Status
                    </button>
                    <button type="button" class="btn btn-secondary" onclick="copySessionId()">
                        <i class="fas fa-copy"></i> Copy Session ID
                    </button>
                </div>
                
                <div id="session-info-result" class="output-container" style="display: none;">
                    <h3>Session Status Details</h3>
                    <div id="session-info-output" class="output-content"></div>
                </div>
            </div>
        `;
    }

    // Load BRM Fields Page
    createLoadFieldsPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-upload"></i> Load BRM Fields</h2>
                <p>Load Oracle BRM field definitions into your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="load-fields-session-display">-</strong></span>
                </div>
                
                <form id="load-fields-form" onsubmit="loadBrmFields(event)">
                    <div class="form-group">
                        <label for="fields-data">BRM Fields Data</label>
                        <textarea id="fields-data" name="fieldsData" required 
                                  placeholder="Enter field data in format: field_name|data_type|field_number"></textarea>
                        <small>Format: field_name|data_type|field_number (one per line)</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-upload"></i> Load Fields
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleFields()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="load-fields-result" class="output-container" style="display: none;">
                    <h3>Load Result</h3>
                    <div id="load-fields-output" class="output-content"></div>
                </div>
            </div>
        `;
    }

    // View Session Fields Page
    createViewFieldsPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-list"></i> View Session Fields</h2>
                <p>View all loaded BRM fields for your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="view-fields-session-display">-</strong></span>
                </div>
                
                <div class="form-actions">
                    <button type="button" class="btn btn-primary" onclick="viewSessionFields()">
                        <i class="fas fa-search"></i> View Fields
                    </button>
                    <button type="button" class="btn btn-secondary" onclick="refreshSessionFields()">
                        <i class="fas fa-sync"></i> Refresh
                    </button>
                </div>
                
                <div id="view-fields-result" class="output-container" style="display: none;">
                    <h3>Session Fields</h3>
                    <div id="view-fields-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('view-fields-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('view-fields-output', 'session-fields.json')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // Field Spec to PODL Page
    createFieldSpecPodlPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-file-code"></i> Field Spec to PODL</h2>
                <p>Convert field specifications to PODL format using your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="field-spec-session-display">-</strong></span>
                </div>
                
                <form id="field-spec-podl-form" onsubmit="convertFieldSpecToPodl(event)">
                    <div class="form-group">
                        <label for="field-spec-data">Field Specification</label>
                        <textarea id="field-spec-data" name="fieldSpecData" required 
                                  placeholder="field_name|description|data_type|field_id"></textarea>
                        <small>Format: field_name|description|data_type|field_id (one per line)</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-exchange-alt"></i> Convert to PODL
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleFieldSpec()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="field-spec-podl-result" class="output-container" style="display: none;">
                    <h3>PODL Output</h3>
                    <div id="field-spec-podl-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('field-spec-podl-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('field-spec-podl-output', 'field-spec.podl')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // Class Spec to PODL Page
    createClassSpecPodlPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-sitemap"></i> Class Spec to PODL</h2>
                <p>Convert class specifications to PODL format using your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="class-spec-session-display">-</strong></span>
                </div>
                
                <form id="class-spec-podl-form" onsubmit="convertClassSpecToPodl(event)">
                    <div class="form-group">
                        <label for="class-spec-data">Class Specification</label>
                        <textarea id="class-spec-data" name="classSpecData" required 
                                  placeholder="Enter class specification with nested field structure"></textarea>
                        <small>Enter class definition with tab-indented field hierarchy</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-exchange-alt"></i> Convert to PODL
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleClassSpec()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="class-spec-podl-result" class="output-container" style="display: none;">
                    <h3>PODL Class Output</h3>
                    <div id="class-spec-podl-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('class-spec-podl-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('class-spec-podl-output', 'class-spec.podl')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // FLIST to Code Page
    createFlistToCodePage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-code"></i> FLIST to C Code</h2>
                <p>Convert FLIST format to Oracle BRM C code using your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="flist-code-session-display">-</strong></span>
                </div>
                
                <form id="flist-to-code-form" onsubmit="convertFlistToCode(event)">
                    <div class="form-group">
                        <label for="flist-code-data">FLIST Data</label>
                        <textarea id="flist-code-data" name="flistData" required 
                                  placeholder="Enter FLIST format data"></textarea>
                        <small>Enter FLIST in Oracle BRM standard format</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-code"></i> Convert to C Code
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleFlist()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="flist-to-code-result" class="output-container" style="display: none;">
                    <h3>Generated C Code</h3>
                    <div id="flist-to-code-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('flist-to-code-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('flist-to-code-output', 'flist-code.c')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // FLIST to XML Page
    createFlistToXmlPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-file-code"></i> FLIST to XML</h2>
                <p>Convert FLIST format to XML structure using your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="flist-xml-session-display">-</strong></span>
                </div>
                
                <form id="flist-to-xml-form" onsubmit="convertFlistToXml(event)">
                    <div class="form-group">
                        <label for="flist-xml-data">FLIST Data</label>
                        <textarea id="flist-xml-data" name="flistData" required 
                                  placeholder="Enter FLIST format data"></textarea>
                        <small>Enter FLIST in Oracle BRM standard format</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-file-code"></i> Convert to XML
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleFlist()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="flist-to-xml-result" class="output-container" style="display: none;">
                    <h3>Generated XML</h3>
                    <div id="flist-to-xml-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('flist-to-xml-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('flist-to-xml-output', 'flist-data.xml')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // FLIST to JSON Page
    createFlistToJsonPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-file-alt"></i> FLIST to JSON</h2>
                <p>Convert FLIST format to JSON structure using your current session</p>
            </div>
            
            <div class="section">
                <div class="session-info-bar">
                    <i class="fas fa-info-circle"></i>
                    <span>Using Session: <strong id="flist-json-session-display">-</strong></span>
                </div>
                
                <form id="flist-to-json-form" onsubmit="convertFlistToJson(event)">
                    <div class="form-group">
                        <label for="flist-json-data">FLIST Data</label>
                        <textarea id="flist-json-data" name="flistData" required 
                                  placeholder="Enter FLIST format data"></textarea>
                        <small>Enter FLIST in Oracle BRM standard format</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-file-alt"></i> Convert to JSON
                        </button>
                        <button type="button" class="btn btn-secondary" onclick="loadSampleFlist()">
                            <i class="fas fa-file-alt"></i> Load Sample
                        </button>
                    </div>
                </form>
                
                <div id="flist-to-json-result" class="output-container" style="display: none;">
                    <h3>Generated JSON</h3>
                    <div id="flist-to-json-output" class="output-content"></div>
                    <div class="output-actions">
                        <button class="btn btn-secondary" onclick="copyToClipboard('flist-to-json-output')">
                            <i class="fas fa-copy"></i> Copy
                        </button>
                        <button class="btn btn-accent" onclick="downloadContent('flist-to-json-output', 'flist-data.json')">
                            <i class="fas fa-download"></i> Download
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    // Call Stack Visualization Page
    createCallStackPage() {
        return `
            <div class="page-header">
                <h2><i class="fas fa-layer-group"></i> Call Stack Visualization</h2>
                <p>Upload and visualize Oracle BRM call stack traces</p>
            </div>
            
            <div class="section">
                <form id="call-stack-form" onsubmit="visualizeCallStack(event)">
                    <div class="form-group">
                        <label for="call-stack-file">Call Stack ZIP File</label>
                        <div class="file-input-container">
                            <input type="file" id="call-stack-file" name="file" accept=".zip" required class="file-input">
                            <label for="call-stack-file" class="file-input-label">
                                <i class="fas fa-cloud-upload-alt"></i>
                                <span>Choose ZIP file or drag & drop</span>
                            </label>
                        </div>
                        <div id="call-stack-file-info" class="file-info" style="display: none;"></div>
                    </div>
                    
                    <div class="form-group">
                        <label for="highlight-threshold">Highlight Threshold (seconds)</label>
                        <input type="number" id="highlight-threshold" name="threshold" 
                               step="0.001" min="0" placeholder="0.005">
                        <small>Operations exceeding this duration will be highlighted (optional)</small>
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">
                            <i class="fas fa-chart-line"></i> Generate Visualization
                        </button>
                    </div>
                </form>
                
                <div id="call-stack-result" class="output-container" style="display: none;">
                    <h3>Call Stack Analysis</h3>
                    <div id="call-stack-output" class="output-content">
                        <p>Processing call stack data...</p>
                    </div>
                </div>
            </div>
        `;
    }

    // Navigation methods
    showPage(pageId) {
        // Hide all pages
        document.querySelectorAll('.page').forEach(page => {
            page.classList.remove('active');
        });

        // Remove active class from nav links
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });

        // Show selected page
        const targetPage = document.getElementById(`${pageId}-page`);
        if (targetPage) {
            targetPage.classList.add('active');
            this.currentPage = pageId;
        }

        // Add active class to clicked nav link
        const activeLink = document.querySelector(`[onclick="showPage('${pageId}')"]`);
        if (activeLink) {
            activeLink.classList.add('active');
        }

        // Update session displays on the page
        this.updateSessionDisplays();

        // Close sidebar on mobile
        this.closeSidebar();

        // Log activity
        this.addActivity(`Navigated to ${pageId} page`);
    }

    // Update session ID displays on pages
    updateSessionDisplays() {
        console.log('updateSessionDisplays called');
        const sessionId = this.getCurrentSessionId();
        const currentUser = window.auth && window.auth.currentUser ? window.auth.currentUser : null;
        console.log('Current user:', currentUser);
        console.log('Session ID:', sessionId);
        
        const displayElements = [
            'current-session-id',
            'current-username', 
            'current-login-time',
            'current-session-status',
            'load-fields-session-display',
            'view-fields-session-display',
            'field-spec-session-display',
            'class-spec-session-display',
            'flist-code-session-display',
            'flist-xml-session-display',
            'flist-json-session-display'
        ];

        displayElements.forEach(elementId => {
            const element = document.getElementById(elementId);
            console.log(`Updating element: ${elementId}, found: ${!!element}`);
            if (element) {
                if (elementId.includes('session-display')) {
                    element.textContent = sessionId || 'No session';
                } else if (elementId === 'current-session-id') {
                    element.value = sessionId || '';
                } else if (elementId === 'current-username') {
                    element.value = currentUser ? currentUser.username : '';
                } else if (elementId === 'current-login-time') {
                    const loginTime = currentUser ? currentUser.lastLogin : null;
                    element.value = loginTime ? new Date(loginTime).toLocaleString() : '';
                } else if (elementId === 'current-session-status') {
                    element.value = sessionId ? 'Active' : 'Not Active';
                }
            }
        });
    }

    // Sidebar methods
    toggleSidebar() {
        const sidebar = document.getElementById('sidebar');
        this.sidebarOpen = !this.sidebarOpen;
        
        if (this.sidebarOpen) {
            sidebar.classList.add('open');
        } else {
            sidebar.classList.remove('open');
        }
    }

    closeSidebar() {
        const sidebar = document.getElementById('sidebar');
        sidebar.classList.remove('open');
        this.sidebarOpen = false;
    }

    // User dropdown methods
    toggleUserDropdown() {
        const dropdown = document.getElementById('userDropdown');
        dropdown.classList.toggle('show');
    }

    closeUserDropdown() {
        const dropdown = document.getElementById('userDropdown');
        dropdown.classList.remove('show');
    }

    // Health check
    startHealthCheck() {
        // Perform initial health check
        this.performHealthCheck();
        
        // Set up periodic health checks
        setInterval(() => {
            this.performHealthCheck();
        }, 30000); // Every 30 seconds
    }

    async performHealthCheck() {
        try {
            console.log('Performing health check to http://localhost:3000/health');
            const response = await fetch('http://localhost:3000/health');
            if (response.ok) {
                document.getElementById('server-status').textContent = 'Online';
                document.getElementById('server-status').style.color = '#28a745';
                console.log('Health check successful');
            } else {
                document.getElementById('server-status').textContent = 'Error';
                document.getElementById('server-status').style.color = '#dc3545';
                console.log('Health check failed with status:', response.status);
            }
        } catch (error) {
            document.getElementById('server-status').textContent = 'Offline';
            document.getElementById('server-status').style.color = '#dc3545';
            console.log('Health check error:', error);
        }
    }

    // Update dashboard stats
    updateStats() {
        // Update active sessions count
        document.getElementById('active-sessions').textContent = this.sessions.size;
        
        // Update other stats (placeholder values)
        document.getElementById('loaded-fields').textContent = '0';
        document.getElementById('conversions-today').textContent = '0';
    }

    // Activity logging
    addActivity(message) {
        const activity = {
            message,
            timestamp: new Date(),
            icon: 'fas fa-info-circle'
        };
        
        this.activityLog.unshift(activity);
        
        // Keep only last 10 activities
        if (this.activityLog.length > 10) {
            this.activityLog = this.activityLog.slice(0, 10);
        }
        
        this.updateActivityDisplay();
    }

    updateActivityDisplay() {
        const container = document.getElementById('recent-activity');
        if (!container) return;
        
        container.innerHTML = this.activityLog.map(activity => `
            <div class="activity-item">
                <i class="${activity.icon}"></i>
                <span>${activity.message}</span>
                <small>${this.formatTimeAgo(activity.timestamp)}</small>
            </div>
        `).join('');
    }

    loadRecentActivity() {
        this.addActivity('Dashboard initialized');
    }

    formatTimeAgo(date) {
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        return `${diffDays}d ago`;
    }
}

// Session management is now automatic - no manual session ID generation needed

function copyToClipboard(elementId) {
    const element = document.getElementById(elementId);
    const text = element.textContent;
    
    navigator.clipboard.writeText(text).then(() => {
        showNotification('Content copied to clipboard', 'success');
    }).catch(() => {
        showNotification('Failed to copy content', 'error');
    });
}

function downloadContent(elementId, filename) {
    const element = document.getElementById(elementId);
    const content = element.textContent;
    
    const blob = new Blob([content], { type: 'text/plain' });
    const url = window.URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    
    window.URL.revokeObjectURL(url);
    showNotification(`Downloaded ${filename}`, 'success');
}

// Sample data loaders
function loadSampleFields() {
    const sampleData = `PIN_FLD_ACCOUNT_OBJ|PIN_FLDT_POID|1
PIN_FLD_ACCOUNT_NO|PIN_FLDT_STR|2
PIN_FLD_ACCOUNT_TYPE|PIN_FLDT_ENUM|3
PIN_FLD_CREATED_T|PIN_FLDT_TSTAMP|4
PIN_FLD_STATUS|PIN_FLDT_ENUM|5`;
    document.getElementById('fields-data').value = sampleData;
}

function loadSampleFieldSpec() {
    const sampleData = `PIN_FLD_ACCOUNT_OBJ|Account Object Reference|PIN_FLDT_POID|1
PIN_FLD_ACCOUNT_NO|Account Number|PIN_FLDT_STR|2
PIN_FLD_ACCOUNT_TYPE|Account Type|PIN_FLDT_ENUM|3`;
    document.getElementById('field-spec-data').value = sampleData;
}

function loadSampleClassSpec() {
    const sampleData = `/account/customer|Customer Account Class
	PIN_FLD_ACCOUNT_OBJ|Account Object Reference
	PIN_FLD_ACCOUNT_NO|Account Number
	PIN_FLD_CREATED_T|Creation Timestamp`;
    document.getElementById('class-spec-data').value = sampleData;
}

function loadSampleFlist() {
    const sampleData = `0 PIN_FLD_POID POID [0] 0.0.0.1 /account/customer 1
0 PIN_FLD_ACCOUNT_NO STR [0] "123456789"
0 PIN_FLD_ACCOUNT_TYPE ENUM [0] 1
0 PIN_FLD_CREATED_T TSTAMP [0] (1234567890)`;
    
    // Set the same sample for all FLIST conversion forms
    const forms = ['flist-code-data', 'flist-xml-data', 'flist-json-data'];
    forms.forEach(formId => {
        const element = document.getElementById(formId);
        if (element) {
            element.value = sampleData;
        }
    });
}

// Global dashboard instance
let dashboardInstance;

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Wait a bit for auth to initialize
    setTimeout(() => {
        dashboardInstance = new Dashboard();
        window.dashboard = dashboardInstance;
        
        // Test backend connectivity
        setTimeout(() => {
            console.log('Testing backend connectivity...');
            fetch('http://localhost:3000/health')
                .then(response => {
                    if (response.ok) {
                        console.log('✅ Backend connectivity test passed');
                        showNotification('Backend connection established', 'success');
                    } else {
                        console.log('❌ Backend returned error:', response.status);
                        showNotification('Backend connection error', 'warning');
                    }
                })
                .catch(error => {
                    console.log('❌ Backend connectivity test failed:', error);
                    showNotification('Cannot connect to backend server. Please ensure Rust server is running on port 3000.', 'error');
                });
        }, 1000);
    }, 100);
});

// Session Info Page Functions
function checkCurrentSessionStatus() {
    const sessionId = window.dashboard.getCurrentSessionId();
    if (!sessionId) {
        showNotification('No active session found', 'error');
        return;
    }
    
    // Use the existing API function but pass the current session ID
    window.brmApi.getSessionStatus(sessionId).then(result => {
        const outputContainer = document.getElementById('session-info-result');
        const outputContent = document.getElementById('session-info-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = JSON.stringify(result, null, 2);
        
        showNotification('Session status retrieved', 'success');
        
        if (window.dashboard) {
            window.dashboard.addActivity(`Checked session status: ${sessionId}`);
        }
    }).catch(error => {
        showNotification(`Status check failed: ${error.message}`, 'error');
        
        const outputContainer = document.getElementById('session-info-result');
        const outputContent = document.getElementById('session-info-output');
        
        outputContainer.style.display = 'block';
        outputContent.textContent = `Error: ${error.message}`;
    });
}

function copySessionId() {
    const sessionId = window.dashboard.getCurrentSessionId();
    if (!sessionId) {
        showNotification('No session ID to copy', 'error');
        return;
    }
    
    navigator.clipboard.writeText(sessionId).then(() => {
        showNotification('Session ID copied to clipboard', 'success');
    }).catch(() => {
        showNotification('Failed to copy session ID', 'error');
    });
}

function refreshSessionFields() {
    viewSessionFields();
}

// Debug function to check auth state
function debugAuth() {
    console.log('=== AUTH DEBUG INFO ===');
    console.log('window.auth exists:', !!window.auth);
    if (window.auth) {
        console.log('Current user:', window.auth.currentUser);
        console.log('All users:', window.auth.users);
        console.log('Session ID:', window.auth.currentUser ? window.auth.currentUser.sessionId : 'No session');
    }
    console.log('window.dashboard exists:', !!window.dashboard);
    if (window.dashboard) {
        console.log('Dashboard session ID:', window.dashboard.getCurrentSessionId());
    }
    console.log('=======================');
    return 'Debug info logged to console';
}

// Make debug function globally available
window.debugAuth = debugAuth;

// Global navigation function
function showPage(pageId) {
    if (window.dashboard) {
        window.dashboard.showPage(pageId);
    }
}

// Global sidebar toggle
function toggleSidebar() {
    if (window.dashboard) {
        window.dashboard.toggleSidebar();
    }
}

// Global user dropdown toggle
function toggleUserDropdown() {
    if (window.dashboard) {
        window.dashboard.toggleUserDropdown();
    }
}

// File input handler for call stack
document.addEventListener('change', function(e) {
    if (e.target.id === 'call-stack-file') {
        const file = e.target.files[0];
        const fileInfo = document.getElementById('call-stack-file-info');
        
        if (file) {
            fileInfo.style.display = 'block';
            fileInfo.innerHTML = `
                <i class="fas fa-file-archive"></i>
                <strong>${file.name}</strong> (${(file.size / 1024).toFixed(1)} KB)
            `;
        } else {
            fileInfo.style.display = 'none';
        }
    }
});

// Notification system
function showNotification(message, type = 'info', duration = 5000) {
    const container = document.getElementById('notification-container');
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    
    const icons = {
        success: 'fas fa-check-circle',
        error: 'fas fa-exclamation-circle',
        warning: 'fas fa-exclamation-triangle',
        info: 'fas fa-info-circle'
    };
    
    notification.innerHTML = `
        <i class="${icons[type]}"></i>
        <span>${message}</span>
        <button class="notification-close" onclick="this.parentElement.remove()">
            <i class="fas fa-times"></i>
        </button>
    `;
    
    container.appendChild(notification);
    
    // Auto remove after duration
    setTimeout(() => {
        if (notification.parentElement) {
            notification.remove();
        }
    }, duration);
}
