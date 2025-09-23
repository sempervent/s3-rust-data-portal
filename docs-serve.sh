#!/bin/bash
# BlackLake Documentation Server
# Serves MkDocs documentation locally for development

set -euo pipefail

echo "🚀 Starting BlackLake Documentation Server"
echo "=========================================="

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    echo "Please install Python 3 and try again."
    exit 1
fi

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    echo "❌ pip3 is required but not installed."
    echo "Please install pip3 and try again."
    exit 1
fi

# Install dependencies if requirements-docs.txt exists
if [ -f "requirements-docs.txt" ]; then
    echo "📦 Installing documentation dependencies..."
    pip3 install -r requirements-docs.txt
else
    echo "⚠️  requirements-docs.txt not found, installing basic MkDocs..."
    pip3 install mkdocs mkdocs-material
fi

# Check if mkdocs.yml exists
if [ ! -f "mkdocs.yml" ]; then
    echo "❌ mkdocs.yml not found in current directory."
    echo "Please run this script from the project root directory."
    exit 1
fi

# Start the development server
echo "🌐 Starting MkDocs development server..."
echo "📖 Documentation will be available at: http://localhost:8000"
echo "🔄 Auto-reload is enabled - changes will be reflected automatically"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

mkdocs serve --dev-addr=0.0.0.0:8000
