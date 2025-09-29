#!/bin/bash
# Test script for BlackLake CLI with Homebrew zsh

echo "🧪 Testing BlackLake CLI with Homebrew zsh"
echo "=========================================="

# Check zsh versions
echo "📋 Shell Information:"
echo "  Current shell: $SHELL"
echo "  Homebrew zsh: $(/opt/homebrew/bin/zsh --version 2>/dev/null || echo 'Not found')"
echo "  System zsh: $(/bin/zsh --version 2>/dev/null || echo 'Not found')"
echo ""

# Test with Homebrew zsh
echo "🔧 Testing with Homebrew zsh..."
if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc && echo 'Homebrew zsh with .zshrc works!' && which cargo && which rustc"; then
    echo "✅ Homebrew zsh with .zshrc works!"
else
    echo "❌ Homebrew zsh with .zshrc failed!"
fi

echo ""

# Test compilation with Homebrew zsh
echo "🔨 Testing compilation with Homebrew zsh..."
if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc && cargo check -p blacklake-cli"; then
    echo "✅ Compilation successful with Homebrew zsh!"
else
    echo "❌ Compilation failed with Homebrew zsh!"
fi

echo ""

# Test justfile commands
echo "🧪 Testing justfile commands..."
mkdir -p test-dir
echo "test data" > test-dir/sample.txt

if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc && just init-dry-run test-dir"; then
    echo "✅ just init-dry-run successful!"
else
    echo "❌ just init-dry-run failed!"
fi

# Cleanup
rm -rf test-dir

echo ""
echo "🎉 Test completed!"
