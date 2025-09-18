# Oracle BRM Flask Dashboard

A modern, responsive web dashboard for managing Oracle BRM (Billing and Revenue Management) operations built with Python Flask and designed to work with the OneClickBRMRust backend.

## Features

### 🔐 Authentication System
- **Sign Up/Sign In**: Secure user registration and login with Flask sessions
- **User Management**: Admin-only user management panel
- **Role-based Access**: Admin and regular user roles
- **Password Management**: Change passwords with admin oversight
- **Session Management**: Automatic BRM session registration on login

### 📊 Dashboard Overview
- **Real-time Stats**: Monitor active sessions, loaded fields, and server status
- **Recent Activity**: Track recent operations and system events
- **Quick Actions**: Fast access to common BRM operations
- **Responsive Design**: Works seamlessly on desktop and mobile devices

### 🛠 BRM Operations

#### Session Management
- **Automatic Session Registration**: Sessions are created automatically on login
- **Session Status**: Check the current status of existing sessions
- **Session Information**: View detailed session information

#### Field Management
- **Load BRM Fields**: Import Oracle BRM field definitions into sessions
- **View Session Fields**: Display all loaded fields for a specific session

#### PODL Conversion
- **Field Spec to PODL**: Convert field specifications to PODL format
- **Class Spec to PODL**: Convert class specifications to PODL format

#### FLIST Conversion
- **FLIST to C Code**: Generate Oracle BRM C code from FLIST data
- **FLIST to XML**: Convert FLIST format to XML structure
- **FLIST to JSON**: Transform FLIST data to JSON format

#### Analytics
- **Call Stack Visualization**: Upload ZIP files containing call stack traces and generate interactive HTML visualizations with performance analysis

### 👨‍💼 Admin Panel (Admin Users Only)
- **User Management**: View, add, edit, and delete users
- **Password Management**: Reset user passwords
- **Session Monitoring**: View and manage active BRM sessions
- **System Administration**: Monitor system health and performance

## Quick Start

### Prerequisites
- Python 3.7 or higher
- pip3 (Python package installer)
- OneClickBRMRust backend server running on port 3000

### Setup

1. **Start the Rust Backend**:
   ```bash
   cd /opt/apps/OneClickBRMRust
   cargo run
   ```
   The server will start on `http://localhost:3000`

2. **Start the Flask Dashboard**:
   ```bash
   cd /opt/apps/OneClickBRMRust
   ./start-dashboard.sh
   ```
   This will:
   - Start the Rust backend
   - Install Python dependencies
   - Start the Flask dashboard on `http://localhost:8008`

3. **Manual Setup** (if needed):
   ```bash
   cd /opt/apps/OneClickBRMRust/dashboard
   pip3 install -r requirements.txt
   python3 app.py
   ```

4. **Access the Dashboard**:
   Open your web browser and navigate to `http://localhost:8008`

## Default Credentials

For initial setup, the system creates default users:

**Admin User:**
- **Username**: `admin`
- **Password**: `admin123`

**Test User:**
- **Username**: `suresh`
- **Password**: `suresh123`

**⚠️ Important**: Change the default passwords immediately after first login for security.

## Usage Guide

### First Time Setup

1. **Login**: Use the default admin credentials to access the dashboard
2. **Change Password**: Go to Admin Panel → Manage Users → Change admin password
3. **Create Users**: Add additional users as needed through the Admin Panel

### Working with BRM Sessions

1. **Automatic Session Creation**: Sessions are automatically created when you log in
2. **Load BRM Fields**:
   - Go to "BRM Field Management" → "Load BRM Fields"
   - Paste field data in the format: `field_name|data_type|field_number`
   - Click "Load Fields"
3. **Perform Conversions**:
   - Use the various conversion tools under "PODL Conversion" and "FLIST Conversion"
   - Each tool uses your current session automatically
   - Enter your data and get formatted output

### Call Stack Analysis

1. **Prepare Data**: Ensure you have call stack data in a ZIP file
2. **Upload**: Navigate to "Analytics" → "Call Stack Visualization"
3. **Configure**: Set highlight threshold if needed (optional)
4. **Analyze**: Upload the ZIP file and download the generated analysis
5. **View**: Extract the downloaded ZIP and open the HTML file in your browser

## API Integration

The Flask dashboard integrates with the following Rust backend endpoints:

- `GET /health` - Health check
- `POST /register/{session_id}` - Register session
- `GET /status/{session_id}` - Get session status
- `DELETE /unregister/{session_id}` - Unregister session
- `POST /obrm/load_obrm_fields/{session_id}` - Load BRM fields
- `GET /obrm/get_session_fields/{session_id}` - Get session fields
- `POST /obrm/convert_fld_spec_to_podl/{session_id}` - Convert field spec to PODL
- `POST /obrm/convert_class_spec_to_podl/{session_id}` - Convert class spec to PODL
- `POST /obrm/convert_flist2code/{session_id}` - Convert FLIST to C code
- `POST /obrm/convert_flist2xml/{session_id}` - Convert FLIST to XML
- `POST /obrm/convert_flist2json/{session_id}` - Convert FLIST to JSON
- `POST /obrm/view_call_stack` - Visualize call stack from ZIP

## File Structure

```
dashboard/
├── app.py                    # Main Flask application
├── requirements.txt          # Python dependencies
├── templates/                # Jinja2 templates
│   ├── base.html            # Base template
│   ├── login.html           # Login page
│   ├── register.html        # Registration page
│   ├── dashboard.html       # Main dashboard
│   ├── health.html          # Health check page
│   ├── session_info.html    # Session information
│   ├── load_fields.html     # Load BRM fields
│   ├── view_fields.html     # View session fields
│   ├── field_spec_podl.html # Field spec to PODL
│   ├── class_spec_podl.html # Class spec to PODL
│   ├── flist_to_code.html   # FLIST to C code
│   ├── flist_to_xml.html    # FLIST to XML
│   ├── flist_to_json.html   # FLIST to JSON
│   ├── call_stack.html      # Call stack visualization
│   ├── admin.html           # Admin panel
│   ├── admin_users.html     # User management
│   ├── admin_sessions.html  # Session management
│   ├── dashboard_header.html # Shared header
│   └── error.html           # Error pages
├── static/                   # Static files
│   ├── css/
│   │   └── styles.css       # CSS styles
│   ├── js/
│   │   └── dashboard.js     # JavaScript functionality
│   └── downloads/           # Generated files
└── README.md                # This documentation
```

## Configuration

### Environment Variables

You can configure the Flask application using environment variables:

- `SECRET_KEY`: Flask secret key for sessions (default: auto-generated)
- `BRM_BACKEND_URL`: Backend API URL (default: http://localhost:3000)
- `FLASK_ENV`: Flask environment (development/production)
- `FLASK_DEBUG`: Enable debug mode (true/false)

### Example Configuration

```bash
export SECRET_KEY="your-secret-key-here"
export BRM_BACKEND_URL="http://localhost:3000"
export FLASK_ENV="production"
export FLASK_DEBUG="false"
python3 app.py
```

## Browser Compatibility

- **Chrome**: 90+
- **Firefox**: 88+
- **Safari**: 14+
- **Edge**: 90+

## Security Notes

- User data is stored in memory (for demo purposes)
- In production, implement proper database-backed authentication
- Use HTTPS in production environments
- Regularly update passwords and review user access
- Flask sessions are used for authentication state

## Troubleshooting

### Common Issues

1. **Cannot Connect to Backend**:
   - Ensure the Rust server is running on port 3000
   - Check for CORS issues if serving from different ports
   - Verify firewall settings
   - Check browser console for "CORS" or "network" errors

2. **Python Dependencies Issues**:
   - Ensure Python 3.7+ is installed
   - Install dependencies: `pip3 install -r requirements.txt`
   - Check Python path and virtual environment

3. **Flask Server Won't Start**:
   - Check if port 8008 is available
   - Verify Flask app.py exists and is valid
   - Check Flask log output in `dashboard/flask.log`

4. **Authentication Issues**:
   - Clear browser cookies if experiencing login problems
   - Ensure JavaScript is enabled
   - Check browser console for errors
   - Try refreshing the page

5. **Session ID Not Showing**:
   - Check that you're logged in
   - Verify backend connectivity
   - Check Flask logs for session registration errors

6. **Tools Showing Network Errors**:
   - Verify backend is running: open http://localhost:3000/health in browser
   - Check browser developer tools Network tab for failed requests
   - Make sure you're logged in with a valid session

7. **File Upload Issues**:
   - Ensure ZIP files contain valid call stack data
   - Check file size limits
   - Verify file format matches expected Oracle BRM trace format

8. **Conversion Failures**:
   - Ensure session has loaded BRM fields before attempting conversions
   - Verify input data format matches expected specifications
   - Check session ID validity

### Getting Help

1. Check the Flask log file: `dashboard/flask.log`
2. Check the browser console for JavaScript errors
3. Verify the Rust backend logs for API errors
4. Ensure all required files are properly installed
5. Confirm network connectivity between dashboard and backend

## Development

For development and customization:

1. **Modify Templates**: Edit files in `templates/` directory
2. **Add Features**: Extend functionality in `app.py`
3. **API Changes**: Update API integration in `app.py`
4. **Authentication**: Customize auth logic in `app.py`
5. **Styling**: Modify `static/css/styles.css`

### Development Mode

```bash
export FLASK_ENV=development
export FLASK_DEBUG=1
python3 app.py
```

## Production Deployment

For production use:

1. **Use HTTPS**: Configure SSL/TLS certificates
2. **Database**: Implement proper database-backed user management
3. **Reverse Proxy**: Use nginx or similar for serving static files and API proxying
4. **Monitoring**: Set up logging and monitoring for both frontend and backend
5. **Security**: Implement proper session management and CSRF protection
6. **WSGI Server**: Use Gunicorn or similar for production Flask deployment

### Production Example with Gunicorn

```bash
pip3 install gunicorn
gunicorn -w 4 -b 0.0.0.0:8008 app:app
```

## License

This dashboard is part of the OneClickBRMRust project. Please refer to the main project license.