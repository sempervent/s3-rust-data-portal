#!/bin/bash
# Test script to verify Oh My Zsh automation fix

echo "🧪 Testing Oh My Zsh automation configuration"
echo "============================================="

# Test 1: Check if automation config exists
if [ -f ".zshrc.automation" ]; then
    echo "✅ .zshrc.automation found"
else
    echo "❌ .zshrc.automation not found"
    exit 1
fi

# Test 2: Test with automation config
echo ""
echo "🔧 Testing with automation config..."
if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc.automation && echo 'Oh My Zsh automation works!' && which cargo && which rustc"; then
    echo "✅ Oh My Zsh automation config works!"
else
    echo "❌ Oh My Zsh automation config failed!"
fi

# Test 3: Test compilation
echo ""
echo "🔨 Testing compilation with automation config..."
if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc.automation && cargo check -p blacklake-cli"; then
    echo "✅ Compilation successful with automation config!"
else
    echo "❌ Compilation failed with automation config!"
fi

# Test 4: Test justfile commands
echo ""
echo "🧪 Testing justfile commands..."
mkdir -p test-dir
echo "test data" > test-dir/sample.txt

if /opt/homebrew/bin/zsh -l -c "source ~/.zshrc.automation && just init-dry-run test-dir"; then
    echo "✅ just init-dry-run successful with automation config!"
else
    echo "❌ just init-dry-run failed with automation config!"
fi

# Cleanup
rm -rf test-dir

echo ""
echo "🎉 Oh My Zsh automation test completed!"
