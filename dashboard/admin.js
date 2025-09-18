// Admin Panel functionality
class AdminPanel {
    constructor(auth) {
        this.auth = auth;
        this.sessions = new Map();
        this.initialize();
    }

    initialize() {
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Close admin modal when clicking outside
        window.addEventListener('click', (e) => {
            const adminModal = document.getElementById('adminModal');
            if (e.target === adminModal) {
                this.closeAdminPanel();
            }
        });
    }

    // Show admin panel
    showAdminPanel() {
        if (!this.auth.isAdmin()) {
            showNotification('Access denied. Admin role required.', 'error');
            return;
        }

        document.getElementById('adminModal').style.display = 'block';
        this.showAdminTab('users');
        this.refreshUserList();
    }

    // Close admin panel
    closeAdminPanel() {
        document.getElementById('adminModal').style.display = 'none';
    }

    // Show admin tab
    showAdminTab(tab) {
        // Update tab buttons
        document.querySelectorAll('.admin-tabs .tab-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`.admin-tabs .tab-btn[onclick="showAdminTab('${tab}')"]`).classList.add('active');

        // Update tab content
        document.querySelectorAll('.admin-section').forEach(section => {
            section.classList.remove('active');
        });
        document.getElementById(`admin-${tab}`).classList.add('active');

        // Load appropriate data
        if (tab === 'users') {
            this.refreshUserList();
        } else if (tab === 'sessions') {
            this.refreshSessionList();
        }
    }

    // User Management Methods
    refreshUserList() {
        try {
            const users = this.auth.getAllUsers();
            this.displayUsers(users);
        } catch (error) {
            showNotification(error.message, 'error');
        }
    }

    displayUsers(users) {
        const tbody = document.getElementById('user-table-body');
        
        if (users.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="6" class="text-center">No users found</td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = users.map(user => `
            <tr>
                <td><strong>${user.username}</strong></td>
                <td>${user.email}</td>
                <td>
                    <span class="status-badge ${user.role === 'admin' ? 'active' : 'inactive'}">
                        ${user.role.toUpperCase()}
                    </span>
                </td>
                <td>${user.lastLogin ? this.formatDateTime(user.lastLogin) : 'Never'}</td>
                <td>
                    <span class="status-badge ${user.status === 'active' ? 'active' : 'inactive'}">
                        ${user.status.toUpperCase()}
                    </span>
                </td>
                <td>
                    <div class="flex gap-10">
                        <button class="btn btn-sm btn-secondary" onclick="adminPanel.editUser('${user.username}')" title="Edit User">
                            <i class="fas fa-edit"></i>
                        </button>
                        <button class="btn btn-sm btn-primary" onclick="adminPanel.changeUserPassword('${user.username}')" title="Change Password">
                            <i class="fas fa-key"></i>
                        </button>
                        <button class="btn btn-sm ${user.status === 'active' ? 'btn-warning' : 'btn-accent'}" 
                                onclick="adminPanel.toggleUserStatus('${user.username}', '${user.status}')" 
                                title="${user.status === 'active' ? 'Deactivate' : 'Activate'} User">
                            <i class="fas fa-${user.status === 'active' ? 'ban' : 'check'}"></i>
                        </button>
                        ${user.username !== 'admin' ? `
                            <button class="btn btn-sm btn-danger" onclick="adminPanel.deleteUser('${user.username}')" title="Delete User">
                                <i class="fas fa-trash"></i>
                            </button>
                        ` : ''}
                    </div>
                </td>
            </tr>
        `).join('');
    }

    showAddUserForm() {
        const formHTML = `
            <div class="modal" id="addUserModal" style="display: block;">
                <div class="modal-content">
                    <div class="modal-header">
                        <h2><i class="fas fa-user-plus"></i> Add New User</h2>
                        <button class="close-btn" onclick="adminPanel.closeAddUserForm()">
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                    <div class="modal-body">
                        <form id="addUserForm" onsubmit="adminPanel.handleAddUser(event)">
                            <div class="form-group">
                                <label for="add-username">Username</label>
                                <input type="text" id="add-username" name="username" required>
                            </div>
                            <div class="form-group">
                                <label for="add-email">Email</label>
                                <input type="email" id="add-email" name="email" required>
                            </div>
                            <div class="form-group">
                                <label for="add-password">Password</label>
                                <input type="password" id="add-password" name="password" required>
                            </div>
                            <div class="form-group">
                                <label for="add-role">Role</label>
                                <select id="add-role" name="role" required>
                                    <option value="user">User</option>
                                    <option value="admin">Admin</option>
                                </select>
                            </div>
                            <div class="form-actions">
                                <button type="button" class="btn btn-secondary" onclick="adminPanel.closeAddUserForm()">Cancel</button>
                                <button type="submit" class="btn btn-primary">
                                    <i class="fas fa-plus"></i> Add User
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        `;
        
        document.body.insertAdjacentHTML('beforeend', formHTML);
    }

    closeAddUserForm() {
        const modal = document.getElementById('addUserModal');
        if (modal) {
            modal.remove();
        }
    }

    handleAddUser(event) {
        event.preventDefault();
        const formData = new FormData(event.target);
        
        const userData = {
            username: formData.get('username'),
            email: formData.get('email'),
            password: formData.get('password'),
            role: formData.get('role')
        };

        try {
            this.auth.addUser(userData);
            showNotification('User added successfully', 'success');
            this.closeAddUserForm();
            this.refreshUserList();
        } catch (error) {
            showNotification(error.message, 'error');
        }
    }

    editUser(username) {
        const users = this.auth.getAllUsers();
        const user = users.find(u => u.username === username);
        
        if (!user) {
            showNotification('User not found', 'error');
            return;
        }

        const formHTML = `
            <div class="modal" id="editUserModal" style="display: block;">
                <div class="modal-content">
                    <div class="modal-header">
                        <h2><i class="fas fa-user-edit"></i> Edit User: ${user.username}</h2>
                        <button class="close-btn" onclick="adminPanel.closeEditUserForm()">
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                    <div class="modal-body">
                        <form id="editUserForm" onsubmit="adminPanel.handleEditUser(event)">
                            <input type="hidden" name="originalUsername" value="${user.username}">
                            <div class="form-group">
                                <label for="edit-username">Username</label>
                                <input type="text" id="edit-username" name="username" value="${user.username}" required>
                            </div>
                            <div class="form-group">
                                <label for="edit-email">Email</label>
                                <input type="email" id="edit-email" name="email" value="${user.email}" required>
                            </div>
                            <div class="form-group">
                                <label for="edit-role">Role</label>
                                <select id="edit-role" name="role" required>
                                    <option value="user" ${user.role === 'user' ? 'selected' : ''}>User</option>
                                    <option value="admin" ${user.role === 'admin' ? 'selected' : ''}>Admin</option>
                                </select>
                            </div>
                            <div class="form-actions">
                                <button type="button" class="btn btn-secondary" onclick="adminPanel.closeEditUserForm()">Cancel</button>
                                <button type="submit" class="btn btn-primary">
                                    <i class="fas fa-save"></i> Update User
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        `;
        
        document.body.insertAdjacentHTML('beforeend', formHTML);
    }

    closeEditUserForm() {
        const modal = document.getElementById('editUserModal');
        if (modal) {
            modal.remove();
        }
    }

    handleEditUser(event) {
        event.preventDefault();
        const formData = new FormData(event.target);
        
        const originalUsername = formData.get('originalUsername');
        const newData = {
            username: formData.get('username'),
            email: formData.get('email'),
            role: formData.get('role')
        };

        try {
            // Update user data (simplified - in real app, would have proper API)
            const users = this.auth.users;
            const userIndex = users.findIndex(u => u.username === originalUsername);
            
            if (userIndex !== -1) {
                users[userIndex] = { ...users[userIndex], ...newData };
                this.auth.saveUsers(users);
                showNotification('User updated successfully', 'success');
                this.closeEditUserForm();
                this.refreshUserList();
            } else {
                throw new Error('User not found');
            }
        } catch (error) {
            showNotification(error.message, 'error');
        }
    }

    changeUserPassword(username) {
        showChangePasswordModal(username);
    }
    
    // Handle password change from admin panel
    handleAdminPasswordChange(form) {
        const formData = new FormData(form);
        const username = formData.get('username');
        const newPassword = formData.get('newPassword');
        const confirmNewPassword = formData.get('confirmNewPassword');

        if (newPassword !== confirmNewPassword) {
            showNotification('Passwords do not match', 'error');
            return;
        }

        if (newPassword.length < 6) {
            showNotification('Password must be at least 6 characters', 'error');
            return;
        }

        const user = this.auth.users.find(u => u.username === username);
        if (!user) {
            showNotification('User not found', 'error');
            return;
        }

        user.password = this.auth.hashPassword(newPassword);
        this.auth.saveUsers(this.auth.users);

        showNotification('Password updated successfully', 'success');
        closePasswordModal();
        this.refreshUserList();
    }

    toggleUserStatus(username, currentStatus) {
        const newStatus = currentStatus === 'active' ? 'inactive' : 'active';
        
        try {
            this.auth.updateUserStatus(username, newStatus);
            showNotification(`User ${newStatus === 'active' ? 'activated' : 'deactivated'} successfully`, 'success');
            this.refreshUserList();
        } catch (error) {
            showNotification(error.message, 'error');
        }
    }

    deleteUser(username) {
        if (!confirm(`Are you sure you want to delete user "${username}"? This action cannot be undone.`)) {
            return;
        }

        try {
            this.auth.deleteUser(username);
            showNotification('User deleted successfully', 'success');
            this.refreshUserList();
        } catch (error) {
            showNotification(error.message, 'error');
        }
    }

    // Session Management Methods
    refreshSessionList() {
        // Simulate session data (in real app, would fetch from API)
        const mockSessions = [
            {
                sessionId: 'brm_sample_001',
                createdAt: new Date(Date.now() - 3600000).toISOString(),
                status: 'active',
                fieldsCount: 25
            },
            {
                sessionId: 'brm_test_002',
                createdAt: new Date(Date.now() - 7200000).toISOString(),
                status: 'active',
                fieldsCount: 12
            }
        ];
        
        this.displaySessions(mockSessions);
    }

    displaySessions(sessions) {
        const tbody = document.getElementById('session-table-body');
        
        if (sessions.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="5" class="text-center">No active sessions found</td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = sessions.map(session => `
            <tr>
                <td><code>${session.sessionId}</code></td>
                <td>${this.formatDateTime(session.createdAt)}</td>
                <td>
                    <span class="status-badge ${session.status === 'active' ? 'active' : 'inactive'}">
                        ${session.status.toUpperCase()}
                    </span>
                </td>
                <td>${session.fieldsCount}</td>
                <td>
                    <div class="flex gap-10">
                        <button class="btn btn-sm btn-secondary" onclick="adminPanel.viewSessionDetails('${session.sessionId}')" title="View Details">
                            <i class="fas fa-eye"></i>
                        </button>
                        <button class="btn btn-sm btn-warning" onclick="adminPanel.clearSessionFields('${session.sessionId}')" title="Clear Fields">
                            <i class="fas fa-broom"></i>
                        </button>
                        <button class="btn btn-sm btn-danger" onclick="adminPanel.deleteSession('${session.sessionId}')" title="Delete Session">
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                </td>
            </tr>
        `).join('');
    }

    viewSessionDetails(sessionId) {
        // Simulate session details view
        const detailsHTML = `
            <div class="modal" id="sessionDetailsModal" style="display: block;">
                <div class="modal-content large">
                    <div class="modal-header">
                        <h2><i class="fas fa-info-circle"></i> Session Details: ${sessionId}</h2>
                        <button class="close-btn" onclick="adminPanel.closeSessionDetails()">
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                    <div class="modal-body">
                        <div class="form-row">
                            <div class="form-group">
                                <label>Session ID</label>
                                <input type="text" value="${sessionId}" readonly>
                            </div>
                            <div class="form-group">
                                <label>Status</label>
                                <input type="text" value="Active" readonly>
                            </div>
                        </div>
                        
                        <div class="form-row">
                            <div class="form-group">
                                <label>Created At</label>
                                <input type="text" value="${this.formatDateTime(new Date().toISOString())}" readonly>
                            </div>
                            <div class="form-group">
                                <label>Fields Loaded</label>
                                <input type="text" value="25" readonly>
                            </div>
                        </div>
                        
                        <div class="form-group">
                            <label>Recent Operations</label>
                            <div class="output-content">
                                • Fields loaded: 25 fields (2 hours ago)
                                • FLIST to XML conversion (1 hour ago)
                                • Field spec to PODL conversion (30 minutes ago)
                            </div>
                        </div>
                        
                        <div class="form-actions">
                            <button class="btn btn-secondary" onclick="adminPanel.closeSessionDetails()">Close</button>
                            <button class="btn btn-primary" onclick="adminPanel.exportSessionData('${sessionId}')">
                                <i class="fas fa-download"></i> Export Data
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        document.body.insertAdjacentHTML('beforeend', detailsHTML);
    }

    closeSessionDetails() {
        const modal = document.getElementById('sessionDetailsModal');
        if (modal) {
            modal.remove();
        }
    }

    clearSessionFields(sessionId) {
        if (!confirm(`Are you sure you want to clear all fields for session "${sessionId}"?`)) {
            return;
        }

        // Simulate clearing session fields
        showNotification(`Fields cleared for session ${sessionId}`, 'success');
        this.refreshSessionList();
    }

    deleteSession(sessionId) {
        if (!confirm(`Are you sure you want to delete session "${sessionId}"? This action cannot be undone.`)) {
            return;
        }

        // Simulate session deletion
        showNotification(`Session ${sessionId} deleted successfully`, 'success');
        this.refreshSessionList();
    }

    clearAllSessions() {
        if (!confirm('Are you sure you want to clear ALL sessions? This action cannot be undone.')) {
            return;
        }

        // Simulate clearing all sessions
        showNotification('All sessions cleared successfully', 'success');
        this.refreshSessionList();
    }

    exportSessionData(sessionId) {
        // Simulate session data export
        const sessionData = {
            sessionId: sessionId,
            status: 'active',
            createdAt: new Date().toISOString(),
            fields: [
                { name: 'PIN_FLD_ACCOUNT_OBJ', type: 'PIN_FLDT_POID', number: '1' },
                { name: 'PIN_FLD_ACCOUNT_NO', type: 'PIN_FLDT_STR', number: '2' }
            ],
            operations: [
                { type: 'load_fields', timestamp: new Date().toISOString() },
                { type: 'flist_to_xml', timestamp: new Date().toISOString() }
            ]
        };

        const blob = new Blob([JSON.stringify(sessionData, null, 2)], { type: 'application/json' });
        const url = window.URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = `session_${sessionId}_export.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        
        window.URL.revokeObjectURL(url);
        showNotification(`Session data exported: session_${sessionId}_export.json`, 'success');
    }

    // Utility Methods
    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
    }
}

// Global admin panel instance
let adminPanel;

// Global functions for admin panel
function showAdminPanel() {
    if (window.auth && adminPanel) {
        adminPanel.showAdminPanel();
    }
}

function closeAdminPanel() {
    if (adminPanel) {
        adminPanel.closeAdminPanel();
    }
}

function showAdminTab(tab) {
    if (adminPanel) {
        adminPanel.showAdminTab(tab);
    }
}

function refreshUserList() {
    if (adminPanel) {
        adminPanel.refreshUserList();
    }
}

function refreshSessionList() {
    if (adminPanel) {
        adminPanel.refreshSessionList();
    }
}

function showAddUserForm() {
    if (adminPanel) {
        adminPanel.showAddUserForm();
    }
}

function clearAllSessions() {
    if (adminPanel) {
        adminPanel.clearAllSessions();
    }
}

// Initialize admin panel when auth is ready
document.addEventListener('DOMContentLoaded', function() {
    // Wait for auth to initialize
    setTimeout(() => {
        if (window.auth) {
            adminPanel = new AdminPanel(window.auth);
            window.adminPanel = adminPanel;
        }
    }, 200);
});

// Add CSS for admin-specific elements
const adminStyles = `
<style>
.btn-sm {
    padding: 6px 12px;
    font-size: 0.8rem;
}

select {
    width: 100%;
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 1rem;
    background: white;
}

select:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

code {
    background: #f8f9fa;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
}

.status-badge {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.status-badge.active {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
}

.status-badge.inactive {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
}

.admin-actions {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
    flex-wrap: wrap;
}

@media (max-width: 768px) {
    .admin-actions {
        flex-direction: column;
    }
    
    .data-table {
        font-size: 0.9rem;
    }
    
    .btn-sm {
        padding: 4px 8px;
        font-size: 0.7rem;
    }
}
</style>
`;

// Inject admin styles
document.head.insertAdjacentHTML('beforeend', adminStyles);
