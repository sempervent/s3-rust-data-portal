#!/bin/bash
# Wrapper script to run commands with Oh My Zsh properly configured

# Set environment variables to disable interactive features
export ZSH_DISABLE_COMPFIX=true
export ZSH_THEME=robbyrussell
export DISABLE_AUTO_UPDATE=true
export DISABLE_UPDATE_PROMPT=true
export DISABLE_MAGIC_FUNCTIONS=true
export DISABLE_LS_COLORS=true
export DISABLE_AUTO_TITLE=true
export DISABLE_UNTRACKED_FILES_DIRTY=true

# Use Homebrew zsh with proper Oh My Zsh configuration
/opt/homebrew/bin/zsh -l -c "source ~/.zshrc && exec \"$@\""
