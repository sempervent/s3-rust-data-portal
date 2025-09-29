#!/bin/bash
set -e

echo "🧪 Testing BlackLake init command (simple test)..."

# Create test directory
mkdir -p test_simple
echo "test data" > test_simple/data.txt

# Test basic init command
echo "📁 Testing basic directory initialization..."
cargo run -p blacklake-cli -- init test_simple --dry-run

echo "📄 Testing basic file initialization..."
cargo run -p blacklake-cli -- init test_simple/data.txt --dry-run

# Cleanup
rm -rf test_simple

echo "✅ Basic init command test completed!"

