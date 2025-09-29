#!/bin/bash
# Test script to check Homebrew zsh configuration

echo "Testing Homebrew zsh setup..."

# Check if Homebrew zsh exists
if [ -f "/opt/homebrew/bin/zsh" ]; then
    echo "✅ Homebrew zsh found at /opt/homebrew/bin/zsh"
    /opt/homebrew/bin/zsh --version
else
    echo "❌ Homebrew zsh not found at /opt/homebrew/bin/zsh"
fi

# Check system zsh
if [ -f "/bin/zsh" ]; then
    echo "✅ System zsh found at /bin/zsh"
    /bin/zsh --version
else
    echo "❌ System zsh not found at /bin/zsh"
fi

# Check current shell
echo "Current shell: $SHELL"

# Test PATH
echo "PATH: $PATH"
