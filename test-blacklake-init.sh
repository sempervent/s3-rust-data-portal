#!/bin/bash
set -e

echo "🧪 Testing BlackLake CLI Init Command"
echo "======================================"

# Set environment variables
export DATABASE_URL="postgresql://blacklake:blacklake@localhost:5432/blacklake"
export PATH="$HOME/.cargo/bin:$PATH"

echo "📋 Environment:"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  PATH: $PATH"
echo "  Rust version: $(rustc --version)"
echo "  Cargo version: $(cargo --version)"
echo ""

# Test 1: Compilation
echo "🔨 Testing compilation..."
if cargo check -p blacklake-cli; then
    echo "✅ Compilation successful!"
else
    echo "❌ Compilation failed!"
    exit 1
fi

# Test 2: Create test directory
echo ""
echo "📁 Creating test directory..."
mkdir -p test_init_dir
echo "test data" > test_init_dir/sample.txt

# Test 3: Test init command (dry run)
echo ""
echo "🧪 Testing init command (dry run)..."
if cargo run -p blacklake-cli -- init test_init_dir --dry-run; then
    echo "✅ Init command (dry run) successful!"
else
    echo "❌ Init command (dry run) failed!"
fi

# Test 4: Test init command (actual)
echo ""
echo "🧪 Testing init command (actual)..."
if cargo run -p blacklake-cli -- init test_init_dir --namespace test --label domain=demo; then
    echo "✅ Init command successful!"
    
    # Check if files were created
    if [ -d "test_init_dir/.bl" ]; then
        echo "✅ .bl directory created!"
        ls -la test_init_dir/.bl/
    else
        echo "❌ .bl directory not created!"
    fi
else
    echo "❌ Init command failed!"
fi

# Test 5: Test file init
echo ""
echo "🧪 Testing file init..."
if cargo run -p blacklake-cli -- init test_init_dir/sample.txt --class restricted; then
    echo "✅ File init successful!"
    
    # Check if metadata file was created
    if [ -f "test_init_dir/sample.txt.bl.metadata.yaml" ]; then
        echo "✅ Metadata file created!"
        cat test_init_dir/sample.txt.bl.metadata.yaml
    else
        echo "❌ Metadata file not created!"
    fi
else
    echo "❌ File init failed!"
fi

# Cleanup
echo ""
echo "🧹 Cleaning up..."
rm -rf test_init_dir

echo ""
echo "🎉 All tests completed!"
