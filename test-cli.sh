#!/bin/bash
set -e

echo "🧪 Testing BlackLake CLI functionality..."

# Test 1: Build CLI Docker image
echo "📦 Building CLI Docker image..."
docker buildx bake cli-local

# Test 2: Test CLI help command
echo "❓ Testing CLI help command..."
docker run --rm blacklake-cli:local blacklake --help

# Test 3: Test init command help
echo "🔧 Testing init command help..."
docker run --rm blacklake-cli:local blacklake init --help

# Test 4: Test put command help  
echo "📤 Testing put command help..."
docker run --rm blacklake-cli:local blacklake put --help

echo "✅ CLI functionality tests completed!"

