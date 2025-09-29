#!/bin/zsh
# Wrapper script to run commands with proper .zshrc sourcing
source ~/.zshrc
exec "$@"
