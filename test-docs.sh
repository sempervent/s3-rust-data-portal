#!/bin/bash

# Test script for MkDocs documentation setup
echo "🧪 Testing MkDocs Documentation Setup"
echo "======================================"

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 not found. Please install Python 3.8+"
    exit 1
fi

# Check if pip is available
if ! command -v pip &> /dev/null; then
    echo "❌ pip not found. Please install pip"
    exit 1
fi

echo "✅ Python and pip are available"

# Install requirements
echo "📦 Installing documentation requirements..."
pip install -r requirements-docs.txt

if [ $? -eq 0 ]; then
    echo "✅ Requirements installed successfully"
else
    echo "❌ Failed to install requirements"
    exit 1
fi

# Test MkDocs configuration
echo "🔧 Testing MkDocs configuration..."
mkdocs --version

if [ $? -eq 0 ]; then
    echo "✅ MkDocs is working"
else
    echo "❌ MkDocs test failed"
    exit 1
fi

# Test Mermaid plugin
echo "🎨 Testing Mermaid plugin..."
python3 -c "import mkdocs_mermaid2_plugin; print('Mermaid plugin available')"

if [ $? -eq 0 ]; then
    echo "✅ Mermaid plugin is working"
else
    echo "❌ Mermaid plugin test failed"
    exit 1
fi

# Test build (dry run)
echo "🏗️ Testing documentation build..."
mkdocs build --strict

if [ $? -eq 0 ]; then
    echo "✅ Documentation build successful"
    echo "🎉 All tests passed! Documentation is ready."
else
    echo "❌ Documentation build failed"
    exit 1
fi

echo ""
echo "🚀 To serve the documentation locally:"
echo "   mkdocs serve"
echo ""
echo "🚀 To build the documentation:"
echo "   mkdocs build"
