// Dashboard JavaScript functionality for Flask application

// Global dashboard functions
function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        sidebar.classList.toggle('open');
    }
}

function toggleUserDropdown() {
    const dropdown = document.getElementById('userDropdown');
    if (dropdown) {
        dropdown.classList.toggle('show');
    }
}

// Close dropdown when clicking outside
document.addEventListener('click', function(e) {
    if (!e.target.closest('.user-dropdown')) {
        const dropdown = document.getElementById('userDropdown');
        if (dropdown) {
            dropdown.classList.remove('show');
        }
    }
});

// Auto-hide flash messages
document.addEventListener('DOMContentLoaded', function() {
    setTimeout(function() {
        const flashMessages = document.getElementById('flash-messages');
        if (flashMessages) {
            flashMessages.style.opacity = '0';
            setTimeout(() => {
                if (flashMessages.parentElement) {
                    flashMessages.remove();
                }
            }, 300);
        }
    }, 5000);
});

// Utility functions
function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `flash-message ${type}`;
    notification.innerHTML = `
        <i class="fas fa-${type === 'success' ? 'check-circle' : type === 'error' ? 'exclamation-circle' : type === 'warning' ? 'exclamation-triangle' : 'info-circle'}"></i>
        <span>${message}</span>
        <button class="flash-close" onclick="this.parentElement.remove()">
            <i class="fas fa-times"></i>
        </button>
    `;
    
    let container = document.getElementById('flash-messages');
    if (!container) {
        container = document.createElement('div');
        container.id = 'flash-messages';
        document.body.insertBefore(container, document.body.firstChild);
    }
    container.appendChild(notification);
    
    setTimeout(() => {
        if (notification.parentElement) {
            notification.remove();
        }
    }, 5000);
}

function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showNotification('Content copied to clipboard', 'success');
    }).catch(() => {
        showNotification('Failed to copy content', 'error');
    });
}

function downloadContent(content, filename) {
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

// File input handlers
document.addEventListener('change', function(e) {
    if (e.target.type === 'file') {
        const file = e.target.files[0];
        const fileInfo = document.getElementById('file-info');
        
        if (file && fileInfo) {
            fileInfo.style.display = 'block';
            fileInfo.innerHTML = `
                <i class="fas fa-file-archive"></i>
                <strong>${file.name}</strong> (${(file.size / 1024).toFixed(1)} KB)
            `;
        }
    }
});

// Drag and drop functionality for file inputs
document.addEventListener('DOMContentLoaded', function() {
    const fileInputs = document.querySelectorAll('input[type="file"]');
    
    fileInputs.forEach(input => {
        const label = input.nextElementSibling;
        if (label && label.classList.contains('file-input-label')) {
            label.addEventListener('dragover', function(e) {
                e.preventDefault();
                label.classList.add('drag-over');
            });
            
            label.addEventListener('dragleave', function(e) {
                e.preventDefault();
                label.classList.remove('drag-over');
            });
            
            label.addEventListener('drop', function(e) {
                e.preventDefault();
                label.classList.remove('drag-over');
                
                const files = e.dataTransfer.files;
                if (files.length > 0) {
                    input.files = files;
                    input.dispatchEvent(new Event('change'));
                }
            });
        }
    });
});

// Sample data loaders
function loadSampleFields() {
    const sampleData = `PIN_FLD_ACCOUNT_OBJ|PIN_FLDT_POID|1
PIN_FLD_ACCOUNT_NO|PIN_FLDT_STR|2
PIN_FLD_ACCOUNT_TYPE|PIN_FLDT_ENUM|3
PIN_FLD_CREATED_T|PIN_FLDT_TSTAMP|4
PIN_FLD_STATUS|PIN_FLDT_ENUM|5`;
    
    const textarea = document.getElementById('fields_data');
    if (textarea) {
        textarea.value = sampleData;
    }
}

function loadSampleFieldSpec() {
    const sampleData = `PIN_FLD_ACCOUNT_OBJ|Account Object Reference|PIN_FLDT_POID|1
PIN_FLD_ACCOUNT_NO|Account Number|PIN_FLDT_STR|2
PIN_FLD_ACCOUNT_TYPE|Account Type|PIN_FLDT_ENUM|3`;
    
    const textarea = document.getElementById('field_spec_data');
    if (textarea) {
        textarea.value = sampleData;
    }
}

function loadSampleClassSpec() {
    const sampleData = `/account/customer|Customer Account Class
	PIN_FLD_ACCOUNT_OBJ|Account Object Reference
	PIN_FLD_ACCOUNT_NO|Account Number
	PIN_FLD_CREATED_T|Creation Timestamp`;
    
    const textarea = document.getElementById('class_spec_data');
    if (textarea) {
        textarea.value = sampleData;
    }
}

function loadSampleFlist() {
    const sampleData = `0 PIN_FLD_POID POID [0] 0.0.0.1 /account/customer 1
0 PIN_FLD_ACCOUNT_NO STR [0] "123456789"
0 PIN_FLD_ACCOUNT_TYPE ENUM [0] 1
0 PIN_FLD_CREATED_T TSTAMP [0] (1234567890)`;
    
    // Set the same sample for all FLIST conversion forms
    const forms = ['flist_data'];
    forms.forEach(formId => {
        const element = document.getElementById(formId);
        if (element) {
            element.value = sampleData;
        }
    });
}

// Form validation
function validateForm(form) {
    const requiredFields = form.querySelectorAll('[required]');
    let isValid = true;
    
    requiredFields.forEach(field => {
        if (!field.value.trim()) {
            field.classList.add('error');
            isValid = false;
        } else {
            field.classList.remove('error');
        }
    });
    
    return isValid;
}

// Add form validation to all forms
document.addEventListener('DOMContentLoaded', function() {
    const forms = document.querySelectorAll('form');
    
    forms.forEach(form => {
        form.addEventListener('submit', function(e) {
            if (!validateForm(form)) {
                e.preventDefault();
                showNotification('Please fill in all required fields', 'error');
            }
        });
    });
});

// Loading states for buttons
function setButtonLoading(button, loading = true) {
    if (loading) {
        button.disabled = true;
        button.dataset.originalText = button.innerHTML;
        button.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Loading...';
    } else {
        button.disabled = false;
        if (button.dataset.originalText) {
            button.innerHTML = button.dataset.originalText;
            delete button.dataset.originalText;
        }
    }
}

// API helper functions
async function makeApiRequest(url, options = {}) {
    try {
        const response = await fetch(url, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        });
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        return await response.json();
    } catch (error) {
        console.error('API Error:', error);
        throw error;
    }
}

// Export functions for global use
window.toggleSidebar = toggleSidebar;
window.toggleUserDropdown = toggleUserDropdown;
window.showNotification = showNotification;
window.copyToClipboard = copyToClipboard;
window.downloadContent = downloadContent;
window.loadSampleFields = loadSampleFields;
window.loadSampleFieldSpec = loadSampleFieldSpec;
window.loadSampleClassSpec = loadSampleClassSpec;
window.loadSampleFlist = loadSampleFlist;
window.setButtonLoading = setButtonLoading;
window.makeApiRequest = makeApiRequest;

