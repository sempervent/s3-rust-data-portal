#!/bin/zsh
# Full test script for BlackLake CLI with proper zsh configuration

echo "🧪 BlackLake CLI Full Test Suite"
echo "================================="

# Source .zshrc to get proper PATH and environment
source ~/.zshrc

echo "📋 Environment Check:"
echo "  Shell: $SHELL"
echo "  PATH: $PATH"
echo "  Rust version: $(rustc --version)"
echo "  Cargo version: $(cargo --version)"
echo "  Cargo bin: $(which cargo)"
echo ""

# Test 1: Compilation
echo "🔨 Testing CLI compilation..."
if cargo check -p blacklake-cli; then
    echo "✅ CLI compilation successful!"
else
    echo "❌ CLI compilation failed!"
    exit 1
fi

# Test 2: Test justfile commands
echo ""
echo "🧪 Testing justfile commands..."

# Test init-dry-run
echo "Testing init-dry-run..."
if just init-dry-run ./test-dir; then
    echo "✅ init-dry-run successful!"
else
    echo "❌ init-dry-run failed!"
fi

# Test 3: Test actual init command
echo ""
echo "🧪 Testing actual init command..."
mkdir -p test-dir
echo "test data" > test-dir/sample.txt

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

# Test 4: Test file init
echo ""
echo "🧪 Testing file init..."
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
echo "🎉 All tests completed!"
