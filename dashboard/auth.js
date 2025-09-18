// Authentication System
class Auth {
    constructor() {
        this.users = this.loadUsers();
        this.currentUser = this.getCurrentUser();
        this.initializeAuth();
    }

    // Load users from localStorage (in production, this would be from a database)
    loadUsers() {
        const users = localStorage.getItem('brm_users');
        if (!users) {
            // Initialize with default users
            const defaultUsers = [
                {
                    username: 'admin',
                    email: 'admin@brm.com',
                    password: this.hashPassword('admin123'),
                    role: 'admin',
                    status: 'active',
                    lastLogin: null,
                    createdAt: new Date().toISOString()
                },
                {
                    username: 'suresh',
                    email: 'suresh@brm.com',
                    password: this.hashPassword('suresh123'),
                    role: 'user',
                    status: 'active',
                    lastLogin: null,
                    createdAt: new Date().toISOString()
                }
            ];
            this.saveUsers(defaultUsers);
            return defaultUsers;
        }
        return JSON.parse(users);
    }

    // Save users to localStorage
    saveUsers(users) {
        localStorage.setItem('brm_users', JSON.stringify(users));
        this.users = users;
    }

    // Simple password hashing (in production, use proper hashing)
    hashPassword(password) {
        return btoa(password + 'brm_salt').split('').reverse().join('');
    }

    // Verify password
    verifyPassword(password, hash) {
        return this.hashPassword(password) === hash;
    }

    // Get current user from session
    getCurrentUser() {
        const user = sessionStorage.getItem('brm_current_user');
        return user ? JSON.parse(user) : null;
    }

    // Set current user in session
    setCurrentUser(user) {
        sessionStorage.setItem('brm_current_user', JSON.stringify(user));
        this.currentUser = user;
    }

    // Initialize authentication
    initializeAuth() {
        if (this.currentUser) {
            this.showDashboard();
        } else {
            this.showAuthModal();
        }
        this.setupEventListeners();
    }

    // Setup event listeners
    setupEventListeners() {
        // Sign in form
        const signinForm = document.getElementById('signinForm');
        if (signinForm) {
            signinForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.handleSignIn(e.target);
            });
        }

        // Sign up form
        const signupForm = document.getElementById('signupForm');
        if (signupForm) {
            signupForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.handleSignUp(e.target);
            });
        }

        // Change password form
        const changePasswordForm = document.getElementById('changePasswordForm');
        if (changePasswordForm) {
            changePasswordForm.addEventListener('submit', (e) => {
                e.preventDefault();
                // Check if this is from admin panel or regular user
                if (window.adminPanel) {
                    window.adminPanel.handleAdminPasswordChange(e.target);
                } else {
                    this.handleChangePassword(e.target);
                }
            });
        }
    }

    // Handle sign in
    handleSignIn(form) {
        const formData = new FormData(form);
        const username = formData.get('username');
        const password = formData.get('password');

        if (!username || !password) {
            showNotification('Please enter both username and password', 'error');
            return;
        }

        const user = this.users.find(u => u.username === username && u.status === 'active');
        if (!user) {
            showNotification('User not found or account inactive', 'error');
            return;
        }

        if (!this.verifyPassword(password, user.password)) {
            showNotification('Invalid password', 'error');
            return;
        }

        // Update last login
        user.lastLogin = new Date().toISOString();
        this.saveUsers(this.users);

        // Generate automatic session ID for BRM operations
        const sessionId = 'brm_' + user.username + '_' + Date.now().toString(36);
        
        // Set current user with session ID
        this.setCurrentUser({
            username: user.username,
            email: user.email,
            role: user.role,
            lastLogin: user.lastLogin,
            sessionId: sessionId
        });

        // Automatically register BRM session
        this.registerBrmSession(sessionId, user.username);

        showNotification(`Welcome back, ${user.username}!`, 'success');
        this.showDashboard();
    }

    // Handle sign up
    handleSignUp(form) {
        const formData = new FormData(form);
        const username = formData.get('username');
        const email = formData.get('email');
        const password = formData.get('password');
        const confirmPassword = formData.get('confirmPassword');

        // Validation
        if (!username || !email || !password || !confirmPassword) {
            showNotification('Please fill in all fields', 'error');
            return;
        }

        if (password !== confirmPassword) {
            showNotification('Passwords do not match', 'error');
            return;
        }

        if (password.length < 6) {
            showNotification('Password must be at least 6 characters', 'error');
            return;
        }

        // Check if user already exists
        if (this.users.find(u => u.username === username)) {
            showNotification('Username already exists', 'error');
            return;
        }

        if (this.users.find(u => u.email === email)) {
            showNotification('Email already registered', 'error');
            return;
        }

        // Create new user
        const newUser = {
            username,
            email,
            password: this.hashPassword(password),
            role: 'user', // Default role
            status: 'active',
            lastLogin: null,
            createdAt: new Date().toISOString()
        };

        this.users.push(newUser);
        this.saveUsers(this.users);

        showNotification('Account created successfully! You are now logged in.', 'success');
        
        // Auto-login the new user
        const sessionId = 'brm_' + newUser.username + '_' + Date.now().toString(36);
        
        this.setCurrentUser({
            username: newUser.username,
            email: newUser.email,
            role: newUser.role,
            lastLogin: newUser.createdAt,
            sessionId: sessionId
        });

        // Register BRM session
        this.registerBrmSession(sessionId, newUser.username);
        
        // Clear form and show dashboard
        form.reset();
        this.showDashboard();
    }

    // Handle password change
    handleChangePassword(form) {
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

        const user = this.users.find(u => u.username === username);
        if (!user) {
            showNotification('User not found', 'error');
            return;
        }

        user.password = this.hashPassword(newPassword);
        this.saveUsers(this.users);

        showNotification('Password updated successfully', 'success');
        closePasswordModal();
    }

    // Show authentication modal
    showAuthModal() {
        document.getElementById('authModal').style.display = 'block';
        document.getElementById('dashboard').style.display = 'none';
    }

    // Show dashboard
    showDashboard() {
        document.getElementById('authModal').style.display = 'none';
        document.getElementById('dashboard').style.display = 'grid';
        
        if (this.currentUser) {
            document.getElementById('user-name').textContent = this.currentUser.username;
            
            // Show admin menu item if user is admin
            const adminMenuItem = document.getElementById('adminMenuItem');
            if (this.currentUser.role === 'admin') {
                adminMenuItem.style.display = 'block';
            } else {
                adminMenuItem.style.display = 'none';
            }
            
            // Update session displays after dashboard is shown
            setTimeout(() => {
                if (window.dashboard) {
                    window.dashboard.updateSessionDisplays();
                }
            }, 200);
        }
    }

    // Logout
    logout() {
        sessionStorage.removeItem('brm_current_user');
        this.currentUser = null;
        showNotification('Logged out successfully', 'success');
        this.showAuthModal();
        
        // Reset forms
        document.getElementById('signinForm').reset();
        document.getElementById('signupForm').reset();
        showAuthTab('signin');
    }

    // Automatically register BRM session
    async registerBrmSession(sessionId, username) {
        try {
            const response = await fetch(`http://localhost:3000/register/${sessionId}`, {
                method: 'POST'
            });
            
            if (response.ok) {
                console.log(`BRM session ${sessionId} registered for user ${username}`);
                if (window.dashboard) {
                    window.dashboard.addActivity(`Auto-registered BRM session: ${sessionId}`);
                }
            } else {
                console.warn(`Failed to register BRM session: ${response.status}`);
            }
        } catch (error) {
            console.warn(`Error registering BRM session: ${error.message}`);
        }
    }

    // Check if user is admin
    isAdmin() {
        return this.currentUser && this.currentUser.role === 'admin';
    }

    // Get all users (admin only)
    getAllUsers() {
        if (!this.isAdmin()) {
            throw new Error('Access denied. Admin role required.');
        }
        return this.users.map(user => ({
            username: user.username,
            email: user.email,
            role: user.role,
            status: user.status,
            lastLogin: user.lastLogin,
            createdAt: user.createdAt
        }));
    }

    // Update user status (admin only)
    updateUserStatus(username, status) {
        if (!this.isAdmin()) {
            throw new Error('Access denied. Admin role required.');
        }

        const user = this.users.find(u => u.username === username);
        if (!user) {
            throw new Error('User not found');
        }

        user.status = status;
        this.saveUsers(this.users);
        return true;
    }

    // Delete user (admin only)
    deleteUser(username) {
        if (!this.isAdmin()) {
            throw new Error('Access denied. Admin role required.');
        }

        if (username === 'admin') {
            throw new Error('Cannot delete admin user');
        }

        const userIndex = this.users.findIndex(u => u.username === username);
        if (userIndex === -1) {
            throw new Error('User not found');
        }

        this.users.splice(userIndex, 1);
        this.saveUsers(this.users);
        return true;
    }

    // Add user (admin only)
    addUser(userData) {
        if (!this.isAdmin()) {
            throw new Error('Access denied. Admin role required.');
        }

        // Check if user already exists
        if (this.users.find(u => u.username === userData.username)) {
            throw new Error('Username already exists');
        }

        if (this.users.find(u => u.email === userData.email)) {
            throw new Error('Email already registered');
        }

        const newUser = {
            username: userData.username,
            email: userData.email,
            password: this.hashPassword(userData.password),
            role: userData.role || 'user',
            status: 'active',
            lastLogin: null,
            createdAt: new Date().toISOString()
        };

        this.users.push(newUser);
        this.saveUsers(this.users);
        return true;
    }
}

// Auth tab switching
function showAuthTab(tab) {
    const tabs = document.querySelectorAll('.tab-btn');
    const forms = document.querySelectorAll('.auth-form');
    
    tabs.forEach(t => t.classList.remove('active'));
    forms.forEach(f => f.classList.remove('active'));
    
    document.querySelector(`.tab-btn[onclick="showAuthTab('${tab}')"]`).classList.add('active');
    document.getElementById(`${tab}-form`).classList.add('active');
}

// Password modal functions
function showChangePasswordModal(username) {
    document.getElementById('change-password-username').value = username;
    document.getElementById('passwordModal').style.display = 'block';
}

function closePasswordModal() {
    document.getElementById('passwordModal').style.display = 'none';
    document.getElementById('changePasswordForm').reset();
}

// Logout function
function logout() {
    if (window.auth) {
        window.auth.logout();
    }
}

// Initialize authentication when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    window.auth = new Auth();
});

// Close modals when clicking outside
window.onclick = function(event) {
    const authModal = document.getElementById('authModal');
    const passwordModal = document.getElementById('passwordModal');
    
    if (event.target === authModal) {
        // Don't close auth modal by clicking outside (user must sign in)
    }
    
    if (event.target === passwordModal) {
        closePasswordModal();
    }
};
