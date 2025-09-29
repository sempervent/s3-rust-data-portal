#!/bin/bash
# Wrapper script to run commands with Homebrew zsh

# Check if Homebrew zsh exists
if [ -f "/opt/homebrew/bin/zsh" ]; then
    echo "Using Homebrew zsh: /opt/homebrew/bin/zsh"
    /opt/homebrew/bin/zsh -l -c "source ~/.zshrc && exec \"$@\""
else
    echo "Homebrew zsh not found, using system zsh"
    /bin/zsh -l -c "source ~/.zshrc && exec \"$@\""
fi
