#!/usr/bin/env python3
"""
Oracle BRM Dashboard - Flask Application
A modern web dashboard for managing Oracle BRM operations
"""

import os
import json
import hashlib
import secrets
from datetime import datetime, timedelta
from functools import wraps
from flask import Flask, render_template, request, redirect, url_for, session, flash, jsonify, send_file
from werkzeug.security import generate_password_hash, check_password_hash
import requests
import zipfile
import io
from typing import Dict, List, Optional

app = Flask(
    __name__,
    template_folder=os.path.join(os.path.dirname(__file__), 'templates'),
    static_folder=os.path.join(os.path.dirname(__file__), 'static')
)
app.secret_key = os.environ.get('SECRET_KEY', secrets.token_hex(32))

# Configuration
app.config['BRM_BACKEND_URL'] = os.environ.get('BRM_BACKEND_URL', 'http://localhost:3000')
app.config['SESSION_PERMANENT'] = False
app.config['PERMANENT_SESSION_LIFETIME'] = timedelta(hours=24)

# In-memory storage for demo (in production, use a database)
users_db = {}
sessions_db = {}

# Inject common template variables
@app.context_processor
def inject_globals():
    current_user = users_db.get(session.get('user_id')) if 'user_id' in session else None
    return {
        'user': current_user,
        'current_session_id': session.get('brm_session_id') if current_user else None
    }

def init_default_users():
    """Initialize default users for the system"""
    global users_db
    
    # Default admin user
    users_db['admin'] = {
        'username': 'admin',
        'email': 'admin@brm.com',
        'password': generate_password_hash('admin123'),
        'role': 'admin',
        'status': 'active',
        'last_login': None,
        'created_at': datetime.now().isoformat()
    }
    
    # Default test user
    users_db['suresh'] = {
        'username': 'suresh',
        'email': 'suresh@brm.com',
        'password': generate_password_hash('suresh123'),
        'role': 'user',
        'status': 'active',
        'last_login': None,
        'created_at': datetime.now().isoformat()
    }

def login_required(f):
    """Decorator to require login for protected routes"""
    @wraps(f)
    def decorated_function(*args, **kwargs):
        if 'user_id' not in session:
            flash('Please log in to access this page.', 'error')
            return redirect(url_for('login'))
        return f(*args, **kwargs)
    return decorated_function

def admin_required(f):
    """Decorator to require admin role for protected routes"""
    @wraps(f)
    def decorated_function(*args, **kwargs):
        if 'user_id' not in session:
            flash('Please log in to access this page.', 'error')
            return redirect(url_for('login'))
        
        user = users_db.get(session['user_id'])
        if not user or user['role'] != 'admin':
            flash('Admin access required.', 'error')
            return redirect(url_for('dashboard'))
        return f(*args, **kwargs)
    return decorated_function

def get_current_user():
    """Get current logged-in user"""
    if 'user_id' not in session:
        return None
    return users_db.get(session['user_id'])

def get_current_session_id():
    """Get current user's BRM session ID"""
    user = get_current_user()
    if not user:
        return None
    return session.get('brm_session_id')

def register_brm_session(session_id: str, username: str) -> bool:
    """Register BRM session with backend"""
    try:
        response = requests.post(f"{app.config['BRM_BACKEND_URL']}/register/{session_id}")
        if response.ok:
            sessions_db[session_id] = {
                'username': username,
                'created_at': datetime.now().isoformat(),
                'status': 'active'
            }
            return True
    except requests.RequestException as e:
        print(f"Failed to register BRM session: {e}")
    return False

def make_brm_api_request(endpoint: str, method: str = 'GET', data: str = None, session_id: str = None) -> Dict:
    """Make API request to BRM backend"""
    try:
        url = f"{app.config['BRM_BACKEND_URL']}{endpoint}"
        if session_id:
            url = url.replace('{session_id}', session_id)
        
        headers = {'Content-Type': 'text/plain'} if data else {}
        
        if method == 'GET':
            response = requests.get(url, headers=headers)
        elif method == 'POST':
            response = requests.post(url, data=data, headers=headers)
        elif method == 'DELETE':
            response = requests.delete(url, headers=headers)
        else:
            raise ValueError(f"Unsupported HTTP method: {method}")
        
        if response.ok:
            try:
                return response.json()
            except:
                return {'content': response.text}
        else:
            return {'error': f"HTTP {response.status_code}: {response.text}"}
    
    except requests.RequestException as e:
        return {'error': f"Backend connection failed: {str(e)}"}

# Routes
@app.route('/')
def index():
    """Redirect to dashboard or login"""
    if 'user_id' in session:
        return redirect(url_for('dashboard'))
    return redirect(url_for('login'))

@app.route('/login', methods=['GET', 'POST'])
def login():
    """User login"""
    if request.method == 'POST':
        username = request.form.get('username')
        password = request.form.get('password')
        
        if not username or not password:
            flash('Please enter both username and password.', 'error')
            return render_template('login.html')
        
        user = users_db.get(username)
        if not user or user['status'] != 'active':
            flash('Invalid username or account inactive.', 'error')
            return render_template('login.html')
        
        if not check_password_hash(user['password'], password):
            flash('Invalid password.', 'error')
            return render_template('login.html')
        
        # Update last login
        user['last_login'] = datetime.now().isoformat()
        
        # Create session
        session['user_id'] = username
        session.permanent = True
        
        # Generate BRM session ID
        brm_session_id = f"brm_{username}_{int(datetime.now().timestamp())}"
        session['brm_session_id'] = brm_session_id
        
        # Register BRM session
        if register_brm_session(brm_session_id, username):
            flash(f'Welcome back, {username}!', 'success')
        else:
            flash(f'Welcome back, {username}! (Note: BRM session registration failed)', 'warning')
        
        return redirect(url_for('dashboard'))
    
    return render_template('login.html')

@app.route('/register', methods=['GET', 'POST'])
def register():
    """User registration"""
    if request.method == 'POST':
        username = request.form.get('username')
        email = request.form.get('email')
        password = request.form.get('password')
        confirm_password = request.form.get('confirm_password')
        
        # Validation
        if not all([username, email, password, confirm_password]):
            flash('Please fill in all fields.', 'error')
            return render_template('register.html')
        
        if password != confirm_password:
            flash('Passwords do not match.', 'error')
            return render_template('register.html')
        
        if len(password) < 6:
            flash('Password must be at least 6 characters.', 'error')
            return render_template('register.html')
        
        # Check if user already exists
        if username in users_db:
            flash('Username already exists.', 'error')
            return render_template('register.html')
        
        if any(user['email'] == email for user in users_db.values()):
            flash('Email already registered.', 'error')
            return render_template('register.html')
        
        # Create new user
        users_db[username] = {
            'username': username,
            'email': email,
            'password': generate_password_hash(password),
            'role': 'user',
            'status': 'active',
            'last_login': None,
            'created_at': datetime.now().isoformat()
        }
        
        flash('Account created successfully! You are now logged in.', 'success')
        
        # Auto-login the new user
        session['user_id'] = username
        session.permanent = True
        
        # Generate BRM session ID
        brm_session_id = f"brm_{username}_{int(datetime.now().timestamp())}"
        session['brm_session_id'] = brm_session_id
        
        # Register BRM session
        register_brm_session(brm_session_id, username)
        
        return redirect(url_for('dashboard'))
    
    return render_template('register.html')

@app.route('/logout')
def logout():
    """User logout"""
    # Unregister BRM session if exists
    brm_session_id = session.get('brm_session_id')
    if brm_session_id:
        make_brm_api_request(f'/unregister/{brm_session_id}', 'DELETE')
        if brm_session_id in sessions_db:
            del sessions_db[brm_session_id]
    
    session.clear()
    flash('Logged out successfully.', 'success')
    return redirect(url_for('login'))

@app.route('/dashboard')
@login_required
def dashboard():
    """Main dashboard"""
    user = get_current_user()
    brm_session_id = get_current_session_id()
    
    # Get dashboard stats
    stats = {
        'active_sessions': len(sessions_db),
        'loaded_fields': 0,  # This would come from backend
        'conversions_today': 0,  # This would be tracked
        'server_status': 'Online'  # This would be checked
    }
    
    return render_template('dashboard.html', user=user, session_id=brm_session_id, stats=stats)

@app.route('/health')
@login_required
def health_check():
    """Health check page"""
    return render_template('health.html')

@app.route('/api/health')
@login_required
def api_health():
    """API endpoint for health check"""
    result = make_brm_api_request('/health')
    return jsonify(result)

@app.route('/session-info')
@login_required
def session_info():
    """Session information page"""
    user = get_current_user()
    brm_session_id = get_current_session_id()
    
    session_data = {
        'session_id': brm_session_id,
        'username': user['username'],
        'login_time': user['last_login'],
        'status': 'Active' if brm_session_id else 'Not Active'
    }
    
    return render_template('session_info.html', user=user, session_data=session_data)

@app.route('/api/session-status')
@login_required
def api_session_status():
    """API endpoint for session status"""
    brm_session_id = get_current_session_id()
    if not brm_session_id:
        return jsonify({'error': 'No active session'})
    
    result = make_brm_api_request(f'/status/{brm_session_id}')
    return jsonify(result)

@app.route('/load-fields', methods=['GET', 'POST'])
@login_required
def load_fields():
    """Load BRM fields page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        fields_data = request.form.get('fields_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('load_fields'))
        
        result = make_brm_api_request(f'/obrm/load_obrm_fields/{brm_session_id}', 'POST', fields_data)
        
        if 'error' in result:
            flash(f'Failed to load fields: {result["error"]}', 'error')
        else:
            flash('BRM fields loaded successfully!', 'success')
        
        return render_template('load_fields.html', result=result)
    
    return render_template('load_fields.html')

@app.route('/view-fields')
@login_required
def view_fields():
    """View session fields page"""
    brm_session_id = get_current_session_id()
    
    if not brm_session_id:
        flash('No active session found.', 'error')
        return redirect(url_for('dashboard'))
    
    result = make_brm_api_request(f'/obrm/get_session_fields/{brm_session_id}')
    
    return render_template('view_fields.html', result=result)

@app.route('/field-spec-podl', methods=['GET', 'POST'])
@login_required
def field_spec_podl():
    """Field spec to PODL conversion page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        field_spec_data = request.form.get('field_spec_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('field_spec_podl'))
        
        result = make_brm_api_request(f'/obrm/convert_fld_spec_to_podl/{brm_session_id}', 'POST', field_spec_data)
        
        return render_template('field_spec_podl.html', result=result)
    
    return render_template('field_spec_podl.html')

@app.route('/class-spec-podl', methods=['GET', 'POST'])
@login_required
def class_spec_podl():
    """Class spec to PODL conversion page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        class_spec_data = request.form.get('class_spec_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('class_spec_podl'))
        
        result = make_brm_api_request(f'/obrm/convert_class_spec_to_podl/{brm_session_id}', 'POST', class_spec_data)
        
        return render_template('class_spec_podl.html', result=result)
    
    return render_template('class_spec_podl.html')

@app.route('/flist-to-code', methods=['GET', 'POST'])
@login_required
def flist_to_code():
    """FLIST to C code conversion page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        flist_data = request.form.get('flist_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('flist_to_code'))
        
        result = make_brm_api_request(f'/obrm/convert_flist2code/{brm_session_id}', 'POST', flist_data)
        
        return render_template('flist_to_code.html', result=result)
    
    return render_template('flist_to_code.html')

@app.route('/flist-to-xml', methods=['GET', 'POST'])
@login_required
def flist_to_xml():
    """FLIST to XML conversion page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        flist_data = request.form.get('flist_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('flist_to_xml'))
        
        result = make_brm_api_request(f'/obrm/convert_flist2xml/{brm_session_id}', 'POST', flist_data)
        
        return render_template('flist_to_xml.html', result=result)
    
    return render_template('flist_to_xml.html')

@app.route('/flist-to-json', methods=['GET', 'POST'])
@login_required
def flist_to_json():
    """FLIST to JSON conversion page"""
    if request.method == 'POST':
        brm_session_id = get_current_session_id()
        flist_data = request.form.get('flist_data')
        
        if not brm_session_id:
            flash('No active session found.', 'error')
            return redirect(url_for('flist_to_json'))
        
        result = make_brm_api_request(f'/obrm/convert_flist2json/{brm_session_id}', 'POST', flist_data)
        
        return render_template('flist_to_json.html', result=result)
    
    return render_template('flist_to_json.html')

@app.route('/call-stack', methods=['GET', 'POST'])
@login_required
def call_stack():
    """Call stack visualization page"""
    if request.method == 'POST':
        if 'file' not in request.files:
            flash('No file selected.', 'error')
            return redirect(url_for('call_stack'))
        
        file = request.files['file']
        if file.filename == '':
            flash('No file selected.', 'error')
            return redirect(url_for('call_stack'))
        
        if not file.filename.endswith('.zip'):
            flash('Please select a ZIP file.', 'error')
            return redirect(url_for('call_stack'))
        
        threshold = request.form.get('threshold', '0.005')
        
        try:
            # Send file to backend
            files = {'file': (file.filename, file.stream, 'application/zip')}
            data = {'threshold': threshold}
            
            response = requests.post(
                f"{app.config['BRM_BACKEND_URL']}/obrm/view_call_stack",
                files=files,
                data=data
            )
            
            if response.ok:
                content_type = response.headers.get('content-type', '')
                ts = int(datetime.now().timestamp())
                downloads_dir = os.path.join(app.static_folder, 'downloads')
                os.makedirs(downloads_dir, exist_ok=True)

                # If backend returns HTML content directly, embed it and save for download
                if 'text/html' in content_type.lower():
                    html_content = response.text
                    html_filename = f"callstack_analysis_{ts}.html"
                    html_path = os.path.join(downloads_dir, html_filename)
                    with open(html_path, 'w', encoding='utf-8') as f:
                        f.write(html_content)

                    flash('Call stack analysis completed!', 'success')
                    return render_template(
                        'call_stack.html',
                        html_content=html_content,
                        download_file=html_filename,
                        uploaded_filename=file.filename,
                        used_threshold=threshold
                    )

                # If backend returns a ZIP, save it, extract it, find an HTML entry and embed via iframe
                if 'application/zip' in content_type.lower() or response.content[:2] == b'PK':
                    zip_filename = f"callstack_analysis_{ts}.zip"
                    zip_path = os.path.join(downloads_dir, zip_filename)
                    with open(zip_path, 'wb') as f:
                        f.write(response.content)

                    # Extract zip to a subdirectory under static/downloads
                    extract_dir_name = f"callstack_analysis_{ts}"
                    extract_dir = os.path.join(downloads_dir, extract_dir_name)
                    os.makedirs(extract_dir, exist_ok=True)

                    with zipfile.ZipFile(io.BytesIO(response.content)) as zf:
                        zf.extractall(extract_dir)
                        # Try to find an index.html or any .html to embed
                        html_candidates = [n for n in zf.namelist() if n.lower().endswith('.html')]
                        selected_html = None
                        if html_candidates:
                            # Prefer index.html
                            preferred = [n for n in html_candidates if os.path.basename(n).lower() == 'index.html']
                            selected_html = preferred[0] if preferred else html_candidates[0]

                        iframe_src = None
                        if selected_html:
                            # Build static URL for iframe
                            # Normalize path to use '/'
                            selected_html = selected_html.replace('\\', '/')
                            static_rel_path = f"downloads/{extract_dir_name}/{selected_html}"
                            iframe_src = url_for('static', filename=static_rel_path)

                    flash('Call stack analysis completed!', 'success')
                    return render_template(
                        'call_stack.html',
                        iframe_src=iframe_src,
                        download_file=zip_filename,
                        uploaded_filename=file.filename,
                        used_threshold=threshold
                    )

                # Fallback: treat as text
                text_content = response.text
                txt_filename = f"callstack_analysis_{ts}.txt"
                txt_path = os.path.join(downloads_dir, txt_filename)
                with open(txt_path, 'w', encoding='utf-8') as f:
                    f.write(text_content)

                flash('Call stack analysis completed!', 'success')
                return render_template(
                    'call_stack.html',
                    html_content=text_content,
                    download_file=txt_filename,
                    uploaded_filename=file.filename,
                    used_threshold=threshold
                )
            else:
                flash(f'Analysis failed: {response.text}', 'error')
        
        except requests.RequestException as e:
            flash(f'Analysis failed: {str(e)}', 'error')
    
    return render_template('call_stack.html')

@app.route('/download/<filename>')
@login_required
def download_file(filename):
    """Download generated files"""
    filepath = os.path.join(app.static_folder, 'downloads', filename)
    if os.path.exists(filepath):
        return send_file(filepath, as_attachment=True)
    else:
        flash('File not found.', 'error')
        return redirect(url_for('dashboard'))

# Admin routes
@app.route('/admin')
@admin_required
def admin_panel():
    """Admin panel"""
    return render_template('admin.html', users=users_db, sessions=sessions_db)

@app.route('/admin/users')
@admin_required
def admin_users():
    """Admin user management"""
    return render_template('admin_users.html', users=users_db)

@app.route('/admin/sessions')
@admin_required
def admin_sessions():
    """Admin session management"""
    return render_template('admin_sessions.html', sessions=sessions_db)

@app.route('/admin/add-user', methods=['POST'])
@admin_required
def admin_add_user():
    """Add new user (admin only)"""
    username = request.form.get('username')
    email = request.form.get('email')
    password = request.form.get('password')
    role = request.form.get('role', 'user')
    
    if not all([username, email, password]):
        flash('Please fill in all fields.', 'error')
        return redirect(url_for('admin_users'))
    
    if username in users_db:
        flash('Username already exists.', 'error')
        return redirect(url_for('admin_users'))
    
    if any(user['email'] == email for user in users_db.values()):
        flash('Email already registered.', 'error')
        return redirect(url_for('admin_users'))
    
    users_db[username] = {
        'username': username,
        'email': email,
        'password': generate_password_hash(password),
        'role': role,
        'status': 'active',
        'last_login': None,
        'created_at': datetime.now().isoformat()
    }
    
    flash(f'User {username} created successfully.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/edit-user/<username>', methods=['POST'])
@admin_required
def admin_edit_user(username):
    """Edit user (admin only)"""
    if username not in users_db:
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    email = request.form.get('email')
    role = request.form.get('role')
    
    if not email or not role:
        flash('Please fill in all fields.', 'error')
        return redirect(url_for('admin_users'))
    
    users_db[username]['email'] = email
    users_db[username]['role'] = role
    
    flash(f'User {username} updated successfully.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/change-password/<username>', methods=['POST'])
@admin_required
def admin_change_password(username):
    """Change user password (admin only)"""
    if username not in users_db:
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    new_password = request.form.get('new_password')
    confirm_password = request.form.get('confirm_password')
    
    if not new_password or not confirm_password:
        flash('Please fill in all fields.', 'error')
        return redirect(url_for('admin_users'))
    
    if new_password != confirm_password:
        flash('Passwords do not match.', 'error')
        return redirect(url_for('admin_users'))
    
    if len(new_password) < 6:
        flash('Password must be at least 6 characters.', 'error')
        return redirect(url_for('admin_users'))
    
    users_db[username]['password'] = generate_password_hash(new_password)
    flash(f'Password updated for user {username}.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/toggle-user-status/<username>')
@admin_required
def admin_toggle_user_status(username):
    """Toggle user status (admin only)"""
    if username not in users_db:
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    if username == 'admin':
        flash('Cannot modify admin user status.', 'error')
        return redirect(url_for('admin_users'))
    
    current_status = users_db[username]['status']
    new_status = 'inactive' if current_status == 'active' else 'active'
    users_db[username]['status'] = new_status
    
    flash(f'User {username} {new_status}.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/delete-user/<username>')
@admin_required
def admin_delete_user(username):
    """Delete user (admin only)"""
    if username not in users_db:
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    if username == 'admin':
        flash('Cannot delete admin user.', 'error')
        return redirect(url_for('admin_users'))
    
    del users_db[username]
    flash(f'User {username} deleted successfully.', 'success')
    return redirect(url_for('admin_users'))

# Error handlers
@app.errorhandler(404)
def not_found(error):
    return render_template('error.html', error_code=404, error_message='Page not found'), 404

@app.errorhandler(500)
def internal_error(error):
    return render_template('error.html', error_code=500, error_message='Internal server error'), 500

# Template filters
@app.template_filter('datetime')
def datetime_filter(value):
    """Format datetime for display"""
    if isinstance(value, str):
        try:
            dt = datetime.fromisoformat(value.replace('Z', '+00:00'))
            return dt.strftime('%Y-%m-%d %H:%M:%S')
        except:
            return value
    return value

@app.template_filter('timeago')
def timeago_filter(value):
    """Format time ago for display"""
    if isinstance(value, str):
        try:
            dt = datetime.fromisoformat(value.replace('Z', '+00:00'))
            now = datetime.now()
            diff = now - dt
            
            if diff.days > 0:
                return f"{diff.days}d ago"
            elif diff.seconds > 3600:
                hours = diff.seconds // 3600
                return f"{hours}h ago"
            elif diff.seconds > 60:
                minutes = diff.seconds // 60
                return f"{minutes}m ago"
            else:
                return "Just now"
        except:
            return value
    return value

if __name__ == '__main__':
    # Initialize default users
    init_default_users()
    
    # Create necessary directories
    os.makedirs(os.path.join(app.static_folder, 'downloads'), exist_ok=True)
    
    # Run the application
    app.run(debug=True, host='0.0.0.0', port=8008)

