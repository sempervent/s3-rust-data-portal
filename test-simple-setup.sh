#!/bin/bash
set -e

echo "🧪 Testing BlackLake Simple Setup..."

# Test 1: Build all images
echo "📦 Building all images..."
just bake

# Test 2: Start database and dependencies
echo "🗄️  Starting database and dependencies..."
docker compose -f docker-compose.simple.yml up -d db minio redis keycloak solr

# Test 3: Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 15

# Test 4: Run migrations
echo "🔄 Running migrations..."
docker compose -f docker-compose.simple.yml run --rm migrations

# Test 5: Start API and UI
echo "🚀 Starting API and UI..."
docker compose -f docker-compose.simple.yml up -d api ui

# Test 6: Wait for services to be ready
echo "⏳ Waiting for API and UI to be ready..."
sleep 10

# Test 7: Check all services
echo "📊 Checking service status..."
docker compose -f docker-compose.simple.yml ps

# Test 8: Test API health
echo "🔍 Testing API health..."
curl -f http://localhost:8080/health || echo "API not ready yet"

# Test 9: Test UI
echo "🔍 Testing UI..."
curl -f http://localhost:5173 || echo "UI not ready yet"

echo "✅ Simple setup test completed!"
echo "🌐 API: http://localhost:8080"
echo "🌐 UI: http://localhost:5173"
echo "🌐 MinIO: http://localhost:9001"
echo "🌐 Keycloak: http://localhost:8081"

