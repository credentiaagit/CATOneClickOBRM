#!/bin/bash

# OneClickBRMRust Dashboard Startup Script
# This script starts both the Rust backend and the Flask dashboard

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting Oracle BRM Flask Dashboard...${NC}"

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Please run this script from the OneClickBRMRust root directory${NC}"
    exit 1
fi

# Check if dashboard directory exists
if [ ! -d "dashboard" ]; then
    echo -e "${RED}❌ Error: Dashboard directory not found${NC}"
    exit 1
fi

# Check if Flask app exists
if [ ! -f "dashboard/app.py" ]; then
    echo -e "${RED}❌ Error: Flask app.py not found in dashboard directory${NC}"
    exit 1
fi

# Function to cleanup background processes on exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down services...${NC}"
    if [ ! -z "$RUST_PID" ]; then
        kill $RUST_PID 2>/dev/null || true
    fi
    if [ ! -z "$FLASK_PID" ]; then
        kill $FLASK_PID 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ Services stopped${NC}"
}

# Trap cleanup on script exit
trap cleanup EXIT INT TERM

# Start Rust backend
echo -e "${BLUE}📡 Starting Rust backend server...${NC}"
cargo run &
RUST_PID=$!

# Wait for backend to start
echo -e "${YELLOW}⏳ Waiting for backend to start...${NC}"
sleep 5

# Check if backend is running
if ! curl -s http://147.79.70.86:3000/health > /dev/null 2>&1; then
    echo -e "${RED}❌ Failed to start Rust backend${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust backend started on http://147.79.70.86:3000${NC}"

# Check Python and pip availability
echo -e "${BLUE}🐍 Checking Python environment...${NC}"
if ! command -v python3 > /dev/null 2>&1; then
    echo -e "${RED}❌ Python3 not found. Please install Python 3.7+${NC}"
    exit 1
fi

# Check if pip is available
if ! command -v pip3 > /dev/null 2>&1; then
    echo -e "${RED}❌ pip3 not found. Please install pip3${NC}"
    exit 1
fi

# Install Flask dependencies if needed
echo -e "${BLUE}📦 Checking Flask dependencies...${NC}"
cd dashboard

if [ -f "requirements.txt" ]; then
    echo -e "${BLUE}Installing Python dependencies...${NC}"
    pip3 install -r requirements.txt --quiet
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to install Python dependencies${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  No requirements.txt found, installing Flask manually...${NC}"
    pip3 install Flask requests --quiet
fi

# Start Flask dashboard server
echo -e "${BLUE}🖥️  Starting Flask dashboard server...${NC}"
export FLASK_APP=app.py
export FLASK_ENV=development
export FLASK_DEBUG=1

# Start Flask in background
python3 app.py > flask.log 2>&1 &
FLASK_PID=$!

cd ..

# Wait for Flask server to start
echo -e "${YELLOW}⏳ Waiting for Flask server to start...${NC}"
sleep 5

# Check if Flask server is accessible
if ! curl -s http://147.79.70.86:8008 > /dev/null 2>&1; then
    echo -e "${RED}❌ Failed to start Flask server${NC}"
    echo -e "${YELLOW}📋 Flask log output:${NC}"
    tail -20 dashboard/flask.log
    exit 1
fi

echo -e "${GREEN}✅ Flask dashboard started on http://147.79.70.86:8008${NC}"

# Display startup information
echo -e "\n${GREEN}🎉 Oracle BRM Flask Dashboard is ready!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}📊 Dashboard:${NC} http://147.79.70.86:8008"
echo -e "${GREEN}🔧 Backend API:${NC} http://147.79.70.86:3000"
echo -e "${GREEN}💻 Server:${NC} Flask Development Server"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${YELLOW}📝 Default Login Credentials:${NC}"
echo -e "${GREEN}Username:${NC} admin"
echo -e "${GREEN}Password:${NC} admin123"
echo -e "${RED}⚠️  Please change the default password after first login!${NC}"

echo -e "\n${YELLOW}🔧 Available Endpoints:${NC}"
echo -e "   • GET  /health - Health check"
echo -e "   • POST /register/{id} - Register session"
echo -e "   • GET  /status/{id} - Session status"
echo -e "   • DELETE /unregister/{id} - Unregister session"
echo -e "   • POST /obrm/load_obrm_fields/{id} - Load BRM fields"
echo -e "   • GET  /obrm/get_session_fields/{id} - Get session fields"
echo -e "   • POST /obrm/convert_* - Various conversion endpoints"
echo -e "   • POST /obrm/view_call_stack - Call stack visualization"

echo -e "\n${BLUE}📖 Documentation:${NC} See dashboard/README.md for detailed usage instructions"
echo -e "\n${GREEN}🔍 Open your browser and navigate to http://147.79.70.86:8008 to get started!${NC}"

# Keep the script running
echo -e "\n${YELLOW}Press Ctrl+C to stop all services${NC}\n"

# Wait for user interrupt
wait
