#!/bin/bash
set -e

echo "🧪 Testing BlackLake CLI Init Command"
echo "======================================"

# Create test directory structure
TEST_DIR="test-artifacts"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Create sample files
echo "Sample CSV data" > "$TEST_DIR/data.csv"
echo "Sample JSON data" > "$TEST_DIR/config.json"
echo "Sample text file" > "$TEST_DIR/README.txt"
echo "Sample Python script" > "$TEST_DIR/script.py"

echo "📁 Created test directory with sample files:"
ls -la "$TEST_DIR"

echo ""
echo "🚀 Testing file initialization..."
echo "blacklake-cli init $TEST_DIR/data.csv --dry-run"
echo ""

echo "🚀 Testing directory initialization..."
echo "blacklake-cli init $TEST_DIR --dry-run"
echo ""

echo "🚀 Testing with custom metadata..."
echo "blacklake-cli init $TEST_DIR/data.csv \\"
echo "  --set 'provenance.author.name=Test User' \\"
echo "  --set 'provenance.author.email=test@example.com' \\"
echo "  --set 'authorization.access_level=public' \\"
echo "  --dry-run"
echo ""

echo "✅ Test examples completed!"
echo ""
echo "To run the actual init commands, use:"
echo "  blacklake-cli init $TEST_DIR/data.csv"
echo "  blacklake-cli init $TEST_DIR"
echo "  blacklake-cli init $TEST_DIR/data.csv --set 'provenance.author.name=Your Name'"

