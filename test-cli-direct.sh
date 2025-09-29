#!/bin/bash
set -e

echo "🧪 Testing BlackLake CLI functionality (direct build)..."

# Test 1: Build CLI Docker image directly for ARM64
echo "📦 Building CLI Docker image directly for ARM64..."
docker buildx build --platform linux/arm64 -f Dockerfile.cli -t blacklake-cli:arm64 .

# Test 2: Test CLI help command
echo "❓ Testing CLI help command..."
docker run --rm blacklake-cli:arm64 blacklake --help

# Test 3: Test init command help
echo "🔧 Testing init command help..."
docker run --rm blacklake-cli:arm64 blacklake init --help

# Test 4: Test put command help  
echo "📤 Testing put command help..."
docker run --rm blacklake-cli:arm64 blacklake put --help

echo "✅ CLI functionality tests completed!"

