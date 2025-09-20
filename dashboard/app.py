#!/usr/bin/env python3
"""
Oracle BRM Dashboard - Flask Application
A modern web dashboard for managing Oracle BRM operations
"""

import os
import json
import hashlib
import secrets
import sqlite3
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
app.config['SESSION_PERMANENT'] = True
app.config['PERMANENT_SESSION_LIFETIME'] = timedelta(days=30)  # 30 days instead of 24 hours

# SQLite database for users (sessions remain in memory only)
DATABASE = 'users.db'

def get_db_connection():
    """Get database connection"""
    conn = sqlite3.connect(DATABASE)
    conn.row_factory = sqlite3.Row
    return conn

def init_database():
    """Initialize the database with users table"""
    conn = get_db_connection()
    conn.execute('''
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            status TEXT NOT NULL DEFAULT 'active',
            last_login TEXT,
            created_at TEXT NOT NULL
        )
    ''')
    conn.commit()
    conn.close()

def get_user(username):
    """Get user by username"""
    conn = get_db_connection()
    user = conn.execute('SELECT * FROM users WHERE username = ?', (username,)).fetchone()
    conn.close()
    return dict(user) if user else None

def get_all_users():
    """Get all users"""
    conn = get_db_connection()
    users = conn.execute('SELECT * FROM users ORDER BY created_at DESC').fetchall()
    conn.close()
    return {user['username']: dict(user) for user in users}

def create_user(username, email, password, role='user'):
    """Create a new user"""
    conn = get_db_connection()
    try:
        conn.execute('''
            INSERT INTO users (username, email, password, role, status, created_at)
            VALUES (?, ?, ?, ?, 'active', ?)
        ''', (username, email, generate_password_hash(password), role, datetime.now().isoformat()))
        conn.commit()
        return True
    except sqlite3.IntegrityError:
        return False
    finally:
        conn.close()

def update_user(username, email=None, role=None, status=None, last_login=None):
    """Update user information"""
    conn = get_db_connection()
    updates = []
    params = []
    
    if email is not None:
        updates.append('email = ?')
        params.append(email)
    if role is not None:
        updates.append('role = ?')
        params.append(role)
    if status is not None:
        updates.append('status = ?')
        params.append(status)
    if last_login is not None:
        updates.append('last_login = ?')
        params.append(last_login)
    
    if updates:
        params.append(username)
        conn.execute(f'UPDATE users SET {", ".join(updates)} WHERE username = ?', params)
        conn.commit()
    conn.close()

def update_user_password(username, password):
    """Update user password"""
    conn = get_db_connection()
    conn.execute('UPDATE users SET password = ? WHERE username = ?', 
                (generate_password_hash(password), username))
    conn.commit()
    conn.close()

def delete_user(username):
    """Delete user"""
    conn = get_db_connection()
    conn.execute('DELETE FROM users WHERE username = ?', (username,))
    conn.commit()
    conn.close()

def user_exists(username):
    """Check if user exists"""
    conn = get_db_connection()
    user = conn.execute('SELECT username FROM users WHERE username = ?', (username,)).fetchone()
    conn.close()
    return user is not None

def email_exists(email):
    """Check if email exists"""
    conn = get_db_connection()
    user = conn.execute('SELECT email FROM users WHERE email = ?', (email,)).fetchone()
    conn.close()
    return user is not None

# Initialize database and storage
init_database()
sessions_db = {}  # Sessions remain in memory only (temporary)

# Global counters for dashboard stats
conversions_today = 0
loaded_fields_count = 0
conversion_date = datetime.now().date()

# Track files created during sessions for cleanup
session_files = {}  # session_id -> list of file paths

# Session management
@app.before_request
def before_request():
    """Handle session management before each request"""
    # Make session permanent if user is logged in
    if 'user_id' in session:
        session.permanent = True
        # Refresh session activity
        session.modified = True

def cleanup_session_files(session_id):
    """Clean up files created during a session"""
    if session_id in session_files:
        for file_path in session_files[session_id]:
            try:
                if os.path.exists(file_path):
                    if os.path.isdir(file_path):
                        # Remove directory and all contents
                        import shutil
                        shutil.rmtree(file_path)
                        print(f"Cleaned up directory: {file_path}")
                    else:
                        # Remove file
                        os.remove(file_path)
                        print(f"Cleaned up file: {file_path}")
            except Exception as e:
                print(f"Error cleaning up {file_path}: {e}")
        del session_files[session_id]

def track_session_file(session_id, file_path):
    """Track a file created during a session for later cleanup"""
    if session_id not in session_files:
        session_files[session_id] = []
    session_files[session_id].append(file_path)

# Inject common template variables
@app.context_processor
def inject_globals():
    current_user = get_user(session.get('user_id')) if 'user_id' in session else None
    return {
        'user': current_user,
        'current_session_id': session.get('brm_session_id') if current_user else None
    }

def migrate_users_from_json():
    """Migrate users from JSON file to SQLite database"""
    json_file = 'users.json'
    if os.path.exists(json_file):
        try:
            with open(json_file, 'r') as f:
                users_data = json.load(f)
            
            for username, user_data in users_data.items():
                if not user_exists(username):
                    # Create user with existing data
                    conn = get_db_connection()
                    conn.execute('''
                        INSERT INTO users (username, email, password, role, status, last_login, created_at)
                        VALUES (?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        username,
                        user_data['email'],
                        user_data['password'],  # Keep existing hashed password
                        user_data['role'],
                        user_data['status'],
                        user_data.get('last_login'),
                        user_data['created_at']
                    ))
                    conn.commit()
                    conn.close()
                    print(f"Migrated user: {username}")
            
            # Backup the JSON file
            os.rename(json_file, f"{json_file}.backup")
            print("User migration completed. JSON file backed up.")
            
        except Exception as e:
            print(f"Error migrating users: {e}")

def init_default_users():
    """Initialize default users for the system"""
    # Migrate existing users from JSON first
    migrate_users_from_json()
    
    # Only add default users if they don't exist
    if not user_exists('admin'):
        create_user('admin', 'admin@brm.com', 'admin123', 'admin')
    
    if not user_exists('suresh'):
        create_user('suresh', 'suresh@brm.com', 'suresh123', 'user')

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
        
        user = get_user(session['user_id'])
        if not user or user['role'] != 'admin':
            flash('Admin access required.', 'error')
            return redirect(url_for('dashboard'))
        return f(*args, **kwargs)
    return decorated_function

def get_current_user():
    """Get current logged-in user"""
    if 'user_id' not in session:
        return None
    return get_user(session['user_id'])

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
            # Sessions remain in memory only (temporary)
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
        
        user = get_user(username)
        if not user or user['status'] != 'active':
            flash('Invalid username or account inactive.', 'error')
            return render_template('login.html')
        
        if not check_password_hash(user['password'], password):
            flash('Invalid password.', 'error')
            return render_template('login.html')
        
        # Update last login
        update_user(username, last_login=datetime.now().isoformat())
        
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
        if user_exists(username):
            flash('Username already exists.', 'error')
            return render_template('register.html')
        
        if email_exists(email):
            flash('Email already registered.', 'error')
            return render_template('register.html')
        
        # Create new user
        if not create_user(username, email, password, 'user'):
            flash('Username already exists.', 'error')
            return render_template('register.html')
        
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
            # Sessions remain in memory only (temporary)
        
        # Clean up files created during this session
        cleanup_session_files(brm_session_id)
    
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
    global conversions_today, loaded_fields_count, conversion_date
    
    # Reset conversions counter if it's a new day
    current_date = datetime.now().date()
    if current_date != conversion_date:
        conversions_today = 0
        conversion_date = current_date
    
    stats = {
        'active_sessions': len(sessions_db),
        'loaded_fields': loaded_fields_count,
        'conversions_today': conversions_today,
        'server_status': 'Online'
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

@app.route('/api/session-ping', methods=['POST'])
def api_session_ping():
    """API endpoint to keep session alive"""
    if 'user_id' in session:
        # User is logged in, refresh session
        session.permanent = True
        session.modified = True
        return jsonify({'status': 'active', 'user': session['user_id']})
    else:
        # User not logged in
        return jsonify({'status': 'inactive'}), 401

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
            global loaded_fields_count
            # Count the number of fields loaded (assuming each line is a field)
            field_count = len([line for line in fields_data.strip().split('\n') if line.strip()])
            loaded_fields_count += field_count
            flash(f'BRM fields loaded successfully! {field_count} fields added.', 'success')
        
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
        
        # Increment conversions counter if successful
        if 'error' not in result:
            global conversions_today
            conversions_today += 1
        
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
        
        # Increment conversions counter if successful
        if 'error' not in result:
            global conversions_today
            conversions_today += 1
        
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
        
        # Increment conversions counter if successful
        if 'error' not in result:
            global conversions_today
            conversions_today += 1
        
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
        
        # Increment conversions counter if successful
        if 'error' not in result:
            global conversions_today
            conversions_today += 1
        
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
        
        # Increment conversions counter if successful
        if 'error' not in result:
            global conversions_today
            conversions_today += 1
        
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
                    
                    # Track file for cleanup
                    brm_session_id = get_current_session_id()
                    if brm_session_id:
                        track_session_file(brm_session_id, html_path)

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
                    
                    # Track ZIP file for cleanup
                    brm_session_id = get_current_session_id()
                    if brm_session_id:
                        track_session_file(brm_session_id, zip_path)

                    # Extract zip to a subdirectory under static/downloads
                    extract_dir_name = f"callstack_analysis_{ts}"
                    extract_dir = os.path.join(downloads_dir, extract_dir_name)
                    os.makedirs(extract_dir, exist_ok=True)
                    
                    # Track extracted directory for cleanup
                    if brm_session_id:
                        track_session_file(brm_session_id, extract_dir)

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
                
                # Track text file for cleanup
                brm_session_id = get_current_session_id()
                if brm_session_id:
                    track_session_file(brm_session_id, txt_path)

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
    return render_template('admin.html', users=get_all_users(), sessions=sessions_db)

@app.route('/admin/users')
@admin_required
def admin_users():
    """Admin user management"""
    return render_template('admin_users.html', users=get_all_users())

@app.route('/admin/sessions')
@admin_required
def admin_sessions():
    """Admin session management"""
    return render_template('admin_sessions.html', sessions=sessions_db)

@app.route('/api/sessions')
@admin_required
def api_sessions():
    """API endpoint for getting sessions data"""
    return jsonify(sessions_db)

@app.route('/admin/delete-session/<session_id>', methods=['DELETE'])
@admin_required
def admin_delete_session(session_id):
    """Delete a specific session"""
    if session_id in sessions_db:
        del sessions_db[session_id]
        # Sessions remain in memory only (temporary)
        
        # Clean up files created during this session
        cleanup_session_files(session_id)
        
        return jsonify({'success': True, 'message': f'Session {session_id} deleted successfully'})
    else:
        return jsonify({'success': False, 'error': 'Session not found'}), 404

@app.route('/admin/clear-all-sessions', methods=['DELETE'])
@admin_required
def admin_clear_all_sessions():
    """Clear all sessions"""
    global sessions_db, session_files
    
    # Clean up files for all sessions before clearing
    for session_id in list(sessions_db.keys()):
        cleanup_session_files(session_id)
    
    sessions_db.clear()
    # Sessions remain in memory only (temporary)
    return jsonify({'success': True, 'message': 'All sessions cleared successfully'})

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
    
    if user_exists(username):
        flash('Username already exists.', 'error')
        return redirect(url_for('admin_users'))
    
    if email_exists(email):
        flash('Email already registered.', 'error')
        return redirect(url_for('admin_users'))
    
    if create_user(username, email, password, role):
        flash(f'User {username} created successfully.', 'success')
    else:
        flash('Failed to create user.', 'error')
    
    return redirect(url_for('admin_users'))

@app.route('/admin/edit-user/<username>', methods=['POST'])
@admin_required
def admin_edit_user(username):
    """Edit user (admin only)"""
    if not user_exists(username):
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    email = request.form.get('email')
    role = request.form.get('role')
    
    if not email or not role:
        flash('Please fill in all fields.', 'error')
        return redirect(url_for('admin_users'))
    
    update_user(username, email=email, role=role)
    flash(f'User {username} updated successfully.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/change-password/<username>', methods=['POST'])
@admin_required
def admin_change_password(username):
    """Change user password (admin only)"""
    if not user_exists(username):
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
    
    update_user_password(username, new_password)
    flash(f'Password updated for user {username}.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/toggle-user-status/<username>')
@admin_required
def admin_toggle_user_status(username):
    """Toggle user status (admin only)"""
    if not user_exists(username):
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    if username == 'admin':
        flash('Cannot modify admin user status.', 'error')
        return redirect(url_for('admin_users'))
    
    user = get_user(username)
    current_status = user['status']
    new_status = 'inactive' if current_status == 'active' else 'active'
    
    update_user(username, status=new_status)
    flash(f'User {username} {new_status}.', 'success')
    return redirect(url_for('admin_users'))

@app.route('/admin/delete-user/<username>')
@admin_required
def admin_delete_user(username):
    """Delete user (admin only)"""
    if not user_exists(username):
        flash('User not found.', 'error')
        return redirect(url_for('admin_users'))
    
    if username == 'admin':
        flash('Cannot delete admin user.', 'error')
        return redirect(url_for('admin_users'))
    
    delete_user(username)
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

def cleanup_all_session_files():
    """Clean up all session files on app shutdown"""
    global session_files
    for session_id in list(session_files.keys()):
        cleanup_session_files(session_id)

if __name__ == '__main__':
    # Initialize default users
    init_default_users()
    
    # Create necessary directories
    os.makedirs(os.path.join(app.static_folder, 'downloads'), exist_ok=True)
    
    # Register cleanup function for app shutdown
    import atexit
    atexit.register(cleanup_all_session_files)
    
    # Run the application
    app.run(debug=True, host='0.0.0.0', port=8008)

