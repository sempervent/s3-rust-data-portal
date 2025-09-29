#!/bin/bash
# BlackLake CLI Helper Script
# This script makes it easy to use the BlackLake CLI

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show help
show_help() {
    echo "BlackLake CLI Helper"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  start     Start the CLI service"
    echo "  run       Run a one-off CLI command"
    echo "  shell     Open an interactive shell"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start                    # Start CLI service"
    echo "  $0 run repos list          # List repositories"
    echo "  $0 run search --query test  # Search for 'test'"
    echo "  $0 shell                    # Open interactive shell"
}

# Function to start CLI service
start_cli() {
    print_status "Starting BlackLake CLI service..."
    docker-compose up cli
}

# Function to run CLI command
run_cli() {
    if [ $# -eq 0 ]; then
        print_error "No command provided"
        show_help
        exit 1
    fi
    
    print_status "Running BlackLake CLI command: $*"
    docker-compose run --rm cli blacklake-cli "$@"
}

# Function to open interactive shell
open_shell() {
    print_status "Opening interactive shell..."
    docker-compose run --rm cli bash
}

# Main script logic
case "${1:-help}" in
    start)
        start_cli
        ;;
    run)
        shift
        run_cli "$@"
        ;;
    shell)
        open_shell
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
