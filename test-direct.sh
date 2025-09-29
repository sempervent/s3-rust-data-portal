#!/bin/bash
# Direct test without VS Code terminal integration

echo "Testing direct execution..."

# Test basic commands
echo "Current directory: $(pwd)"
echo "Shell: $SHELL"
echo "PATH: $PATH"

# Test if we can run cargo
if command -v cargo >/dev/null 2>&1; then
    echo "✅ Cargo found: $(which cargo)"
    cargo --version
else
    echo "❌ Cargo not found"
fi

# Test if we can run rustc
if command -v rustc >/dev/null 2>&1; then
    echo "✅ Rustc found: $(which rustc)"
    rustc --version
else
    echo "❌ Rustc not found"
fi

echo "Test completed!"
