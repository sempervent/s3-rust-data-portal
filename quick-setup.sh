#!/bin/bash
# BlackLake Quick Setup Script
# One command to build everything and start the stack

set -e

echo "🚀 BlackLake Quick Setup"
echo "========================"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if just is available
if command -v just > /dev/null 2>&1; then
    echo "⚡ Using just for optimized setup..."
    just setup-all
else
    echo "🔨 Using direct Docker commands..."
    
    # Build all images locally
    echo "Building all images..."
    docker buildx bake -f docker-bake-simple.hcl local
    
    # Start development stack
    echo "Starting development stack..."
    docker compose --profile dev up -d --wait
    
    echo "✅ Setup complete!"
    echo "🌐 API available at http://localhost:8080"
    echo "📊 Grafana available at http://localhost:3000 (admin/admin)"
    echo "🔍 Solr available at http://localhost:8983"
    echo "🗄️  MinIO available at http://localhost:9001 (minioadmin/minioadmin)"
fi

echo ""
echo "🎯 Next steps:"
echo "  • Check logs: docker compose logs -f api"
echo "  • View all services: docker compose ps"
echo "  • Stop stack: docker compose down"
echo ""
echo "📚 For more commands, see FAST_SETUP.md"
