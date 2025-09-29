#!/bin/bash
set -e

echo "🔨 Building BlackLake images individually..."

# Build API
echo "📦 Building API..."
docker build -f Dockerfile.api -t blacklake-api:local .

# Build UI
echo "📦 Building UI..."
docker build -f Dockerfile -t blacklake-ui:local ./ui

# Build CLI
echo "📦 Building CLI..."
docker build -f Dockerfile.cli -t blacklake-cli:local .

# Build JobRunner
echo "📦 Building JobRunner..."
docker build -f Dockerfile.jobrunner -t blacklake-jobrunner:local .

# Build Gateway
echo "📦 Building Gateway..."
docker build -f Dockerfile -t blacklake-gateway:local ./ops/nginx

echo "✅ All images built successfully!"

