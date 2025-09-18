#!/bin/bash

# Quick Start Script - Minimal setup for immediate testing
# This is the simplest way to get the server running

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "🚀 Quick Starting OneClick BRM Server..."
echo "📍 Server will be at: http://localhost:3000"
echo "⚡ Press Ctrl+C to stop"
echo ""

# Ensure logs directory exists
mkdir -p logs

# Run directly with cargo
cargo run
