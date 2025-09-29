#!/bin/bash
# Manual test script for BlackLake CLI - run this manually in your terminal

echo "🧪 BlackLake CLI Manual Test Suite"
echo "==================================="

# Change to the project directory
cd /Users/joshuagrant/dev/s3-rust-data-portal

echo "📋 Environment Check:"
echo "  Current directory: $(pwd)"
echo "  Shell: $SHELL"
echo "  PATH: $PATH"
echo "  Rust version: $(rustc --version)"
echo "  Cargo version: $(cargo --version)"
echo ""

# Test 1: Compilation
echo "🔨 Testing CLI compilation..."
if cargo check -p blacklake-cli; then
    echo "✅ CLI compilation successful!"
else
    echo "❌ CLI compilation failed!"
    echo "Let's check the specific errors..."
    cargo check -p blacklake-cli 2>&1 | head -20
fi

echo ""

# Test 2: Test justfile commands
echo "🧪 Testing justfile commands..."

# Test init-dry-run
echo "Testing init-dry-run..."
mkdir -p test-dir
echo "test data" > test-dir/sample.txt

if just init-dry-run test-dir; then
    echo "✅ init-dry-run successful!"
else
    echo "❌ init-dry-run failed!"
fi

# Test init-dir
echo "Testing init-dir..."
if just init-dir test-dir; then
    echo "✅ init-dir successful!"
    
    # Check if .bl directory was created
    if [ -d "test-dir/.bl" ]; then
        echo "✅ .bl directory created!"
        echo "Contents:"
        ls -la test-dir/.bl/
    else
        echo "❌ .bl directory not created!"
    fi
else
    echo "❌ init-dir failed!"
fi

# Test init-file
echo "Testing init-file..."
if just init-file test-dir/sample.txt; then
    echo "✅ init-file successful!"
    
    # Check if metadata file was created
    if [ -f "test-dir/sample.txt.bl.metadata.yaml" ]; then
        echo "✅ Metadata file created!"
        echo "Metadata content:"
        cat test-dir/sample.txt.bl.metadata.yaml
    else
        echo "❌ Metadata file not created!"
    fi
else
    echo "❌ init-file failed!"
fi

# Cleanup
echo ""
echo "🧹 Cleaning up..."
rm -rf test-dir

echo ""
echo "🎉 Manual test completed!"
echo ""
echo "If you see any errors above, please share them so I can help fix them."
