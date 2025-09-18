#!/bin/bash

# OneClick BRM Rust Server - Start Script
# This script provides an easy way to start the Rust server

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${BLUE}🚀 OneClick BRM Rust Server Startup Script${NC}"
echo "=================================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: Rust/Cargo is not installed${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if config file exists
if [ ! -f "config.toml" ]; then
    echo -e "${RED}❌ Error: config.toml not found${NC}"
    echo "Please ensure config.toml exists in the project directory"
    exit 1
fi

# Create logs directory if it doesn't exist
mkdir -p logs

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  dev, development    Start in development mode (with auto-reload)"
    echo "  prod, production    Start in production mode (optimized build)"
    echo "  build              Build the project only"
    echo "  clean              Clean build artifacts"
    echo "  test               Run tests"
    echo "  check              Check code without building"
    echo "  help, -h, --help   Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                 # Start in development mode (default)"
    echo "  $0 dev             # Start in development mode"
    echo "  $0 prod            # Start in production mode"
    echo "  $0 build           # Build only"
}

# Function to check dependencies
check_dependencies() {
    echo -e "${YELLOW}🔍 Checking dependencies...${NC}"
    
    if ! cargo check --quiet > /dev/null 2>&1; then
        echo -e "${YELLOW}📦 Installing/updating dependencies...${NC}"
        cargo build
    else
        echo -e "${GREEN}✅ Dependencies OK${NC}"
    fi
}

# Function to start in development mode
start_dev() {
    echo -e "${YELLOW}🛠️  Starting in DEVELOPMENT mode...${NC}"
    check_dependencies
    
    echo -e "${GREEN}📍 Server will be available at:${NC}"
    echo "   🌐 http://localhost:3000"
    echo "   🌐 http://0.0.0.0:3000"
    echo ""
    echo -e "${YELLOW}📝 Logs will be written to: logs/requests.log${NC}"
    echo -e "${YELLOW}⚡ Press Ctrl+C to stop the server${NC}"
    echo ""
    
    # Run in development mode
    cargo run
}

# Function to start in production mode
start_prod() {
    echo -e "${YELLOW}🚀 Starting in PRODUCTION mode...${NC}"
    
    echo -e "${YELLOW}📦 Building optimized release...${NC}"
    cargo build --release
    
    echo -e "${GREEN}📍 Server will be available at:${NC}"
    echo "   🌐 http://localhost:3000"
    echo "   🌐 http://0.0.0.0:3000"
    echo ""
    echo -e "${YELLOW}📝 Logs will be written to: logs/requests.log${NC}"
    echo -e "${YELLOW}⚡ Press Ctrl+C to stop the server${NC}"
    echo ""
    
    # Run the release binary
    ./target/release/oneclick_brm_rust
}

# Function to build only
build_only() {
    echo -e "${YELLOW}🔨 Building project...${NC}"
    cargo build
    echo -e "${GREEN}✅ Build complete${NC}"
}

# Function to build release
build_release() {
    echo -e "${YELLOW}🔨 Building optimized release...${NC}"
    cargo build --release
    echo -e "${GREEN}✅ Release build complete${NC}"
}

# Function to clean
clean_build() {
    echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✅ Clean complete${NC}"
}

# Function to run tests
run_tests() {
    echo -e "${YELLOW}🧪 Running tests...${NC}"
    cargo test
}

# Function to check code
check_code() {
    echo -e "${YELLOW}🔍 Checking code...${NC}"
    cargo check
    echo -e "${GREEN}✅ Check complete${NC}"
}

# Parse command line arguments
case "${1:-dev}" in
    "dev"|"development"|"")
        start_dev
        ;;
    "prod"|"production")
        start_prod
        ;;
    "build")
        build_only
        ;;
    "release")
        build_release
        ;;
    "clean")
        clean_build
        ;;
    "test")
        run_tests
        ;;
    "check")
        check_code
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown option: $1${NC}"
        echo ""
        show_usage
        exit 1
        ;;
esac
