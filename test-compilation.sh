#!/bin/bash
set -e

echo "Testing BlackLake CLI compilation..."

# Set environment variables
export DATABASE_URL="postgresql://blacklake:blacklake@localhost:5432/blacklake"
export PATH="$HOME/.cargo/bin:$PATH"

# Test compilation
echo "Running cargo check..."
cargo check -p blacklake-cli

echo "✅ Compilation successful!"