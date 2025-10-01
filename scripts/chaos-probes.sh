#!/bin/bash
# Chaos Engineering Probes
# Implements chaos probes (dev profile) to simulate packet loss/latency
# Tests S3/Redis/Solr connectivity issues for 60 seconds
# Asserts that SLOs and circuit breakers engage properly

set -euo pipefail

# Configuration
CHAOS_DURATION=60
PACKET_LOSS_RATE=0.1
LATENCY_MS=1000
SLO_THRESHOLD=0.95
CIRCUIT_BREAKER_THRESHOLD=5

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

# Error handling
error_exit() {
    echo -e "${RED}ERROR: $1${NC}"
    exit 1
}

# Success message
success() {
    echo -e "${GREEN}SUCCESS: $1${NC}"
}

# Warning message
warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

# Check if running in dev profile
check_dev_profile() {
    if [ "${NODE_ENV:-production}" != "development" ]; then
        error_exit "Chaos probes can only run in development profile"
    fi
    log "Running in development profile - chaos probes enabled"
}

# Install chaos engineering tools
install_chaos_tools() {
    log "Installing chaos engineering tools..."
    
    # Install tc (traffic control) for network chaos
    if ! command -v tc >/dev/null 2>&1; then
        sudo apt-get update
        sudo apt-get install -y iproute2
    fi
    
    # Install iptables for packet filtering
    if ! command -v iptables >/dev/null 2>&1; then
        sudo apt-get install -y iptables
    fi
    
    # Install chaos engineering tools
    if ! command -v chaos-monkey >/dev/null 2>&1; then
        # Install chaos-monkey (simplified version)
        sudo apt-get install -y python3-pip
        pip3 install chaos-monkey
    fi
    
    success "Chaos engineering tools installed"
}

# Simulate packet loss
simulate_packet_loss() {
    local target_host="$1"
    local loss_rate="$2"
    
    log "Simulating packet loss ($loss_rate) to $target_host"
    
    # Get network interface
    local interface=$(ip route | grep default | awk '{print $5}' | head -1)
    
    # Add packet loss rule
    sudo tc qdisc add dev "$interface" root netem loss "$loss_rate" 2>/dev/null || true
    
    # Store original state for cleanup
    echo "$interface" > /tmp/chaos_interface
}

# Simulate latency
simulate_latency() {
    local target_host="$1"
    local latency="$2"
    
    log "Simulating latency (${latency}ms) to $target_host"
    
    # Get network interface
    local interface=$(ip route | grep default | awk '{print $5}' | head -1)
    
    # Add latency rule
    sudo tc qdisc add dev "$interface" root netem delay "${latency}ms" 2>/dev/null || true
}

# Simulate network partition
simulate_network_partition() {
    local target_host="$1"
    
    log "Simulating network partition to $target_host"
    
    # Block traffic to target host
    sudo iptables -A OUTPUT -d "$target_host" -j DROP
    sudo iptables -A INPUT -s "$target_host" -j DROP
    
    # Store rules for cleanup
    echo "$target_host" > /tmp/chaos_partition_host
}

# Test S3 connectivity
test_s3_connectivity() {
    local s3_endpoint="${S3_ENDPOINT:-localhost:9000}"
    local s3_bucket="${S3_BUCKET:-blacklake}"
    
    log "Testing S3 connectivity to $s3_endpoint"
    
    # Test S3 connectivity
    if curl -f "http://$s3_endpoint/$s3_bucket" >/dev/null 2>&1; then
        success "S3 connectivity test passed"
        return 0
    else
        warning "S3 connectivity test failed"
        return 1
    fi
}

# Test Redis connectivity
test_redis_connectivity() {
    local redis_host="${REDIS_HOST:-localhost}"
    local redis_port="${REDIS_PORT:-6379}"
    
    log "Testing Redis connectivity to $redis_host:$redis_port"
    
    # Test Redis connectivity
    if redis-cli -h "$redis_host" -p "$redis_port" ping >/dev/null 2>&1; then
        success "Redis connectivity test passed"
        return 0
    else
        warning "Redis connectivity test failed"
        return 1
    fi
}

# Test Solr connectivity
test_solr_connectivity() {
    local solr_host="${SOLR_HOST:-localhost}"
    local solr_port="${SOLR_PORT:-8983}"
    
    log "Testing Solr connectivity to $solr_host:$solr_port"
    
    # Test Solr connectivity
    if curl -f "http://$solr_host:$solr_port/solr/admin/ping" >/dev/null 2>&1; then
        success "Solr connectivity test passed"
        return 0
    else
        warning "Solr connectivity test failed"
        return 1
    fi
}

# Monitor SLO compliance
monitor_slo_compliance() {
    local api_endpoint="${API_ENDPOINT:-http://localhost:8080}"
    local threshold="$1"
    local duration="$2"
    
    log "Monitoring SLO compliance (threshold: $threshold) for ${duration}s"
    
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    local total_requests=0
    local successful_requests=0
    
    while [ $(date +%s) -lt $end_time ]; do
        total_requests=$((total_requests + 1))
        
        if curl -f "$api_endpoint/health" >/dev/null 2>&1; then
            successful_requests=$((successful_requests + 1))
        fi
        
        sleep 1
    done
    
    local success_rate=$(echo "scale=2; $successful_requests * 100 / $total_requests" | bc)
    
    log "SLO monitoring results:"
    log "  Total requests: $total_requests"
    log "  Successful requests: $successful_requests"
    log "  Success rate: $success_rate%"
    
    if (( $(echo "$success_rate >= $threshold" | bc -l) )); then
        success "SLO compliance maintained ($success_rate% >= $threshold%)"
        return 0
    else
        warning "SLO compliance violated ($success_rate% < $threshold%)"
        return 1
    fi
}

# Monitor circuit breaker engagement
monitor_circuit_breaker() {
    local api_endpoint="${API_ENDPOINT:-http://localhost:8080}"
    local threshold="$1"
    local duration="$2"
    
    log "Monitoring circuit breaker engagement (threshold: $threshold) for ${duration}s"
    
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    local failure_count=0
    local circuit_breaker_engaged=false
    
    while [ $(date +%s) -lt $end_time ]; do
        if ! curl -f "$api_endpoint/health" >/dev/null 2>&1; then
            failure_count=$((failure_count + 1))
            
            if [ $failure_count -ge $threshold ]; then
                circuit_breaker_engaged=true
                log "Circuit breaker engaged after $failure_count failures"
                break
            fi
        else
            failure_count=0
        fi
        
        sleep 1
    done
    
    if [ "$circuit_breaker_engaged" = true ]; then
        success "Circuit breaker engaged as expected"
        return 0
    else
        warning "Circuit breaker did not engage (failure count: $failure_count)"
        return 1
    fi
}

# Cleanup chaos effects
cleanup_chaos() {
    log "Cleaning up chaos effects..."
    
    # Restore network interface
    if [ -f /tmp/chaos_interface ]; then
        local interface=$(cat /tmp/chaos_interface)
        sudo tc qdisc del dev "$interface" root 2>/dev/null || true
        rm -f /tmp/chaos_interface
    fi
    
    # Remove iptables rules
    if [ -f /tmp/chaos_partition_host ]; then
        local target_host=$(cat /tmp/chaos_partition_host)
        sudo iptables -D OUTPUT -d "$target_host" -j DROP 2>/dev/null || true
        sudo iptables -D INPUT -s "$target_host" -j DROP 2>/dev/null || true
        rm -f /tmp/chaos_partition_host
    fi
    
    success "Chaos effects cleaned up"
}

# Run chaos experiment
run_chaos_experiment() {
    local experiment_name="$1"
    local target_host="$2"
    
    log "Starting chaos experiment: $experiment_name"
    
    # Baseline connectivity tests
    log "Running baseline connectivity tests..."
    test_s3_connectivity
    test_redis_connectivity
    test_solr_connectivity
    
    # Apply chaos
    case "$experiment_name" in
        "packet_loss")
            simulate_packet_loss "$target_host" "$PACKET_LOSS_RATE"
            ;;
        "latency")
            simulate_latency "$target_host" "$LATENCY_MS"
            ;;
        "network_partition")
            simulate_network_partition "$target_host"
            ;;
        *)
            error_exit "Unknown experiment: $experiment_name"
            ;;
    esac
    
    # Monitor system behavior
    log "Monitoring system behavior during chaos..."
    monitor_slo_compliance "$SLO_THRESHOLD" "$CHAOS_DURATION"
    monitor_circuit_breaker "$CIRCUIT_BREAKER_THRESHOLD" "$CHAOS_DURATION"
    
    # Cleanup
    cleanup_chaos
    
    success "Chaos experiment completed: $experiment_name"
}

# Main execution
main() {
    log "Starting chaos engineering probes..."
    
    # Check prerequisites
    check_dev_profile
    install_chaos_tools
    
    # Set up cleanup trap
    trap cleanup_chaos EXIT
    
    # Run chaos experiments
    local experiments=("packet_loss" "latency" "network_partition")
    local target_hosts=("localhost" "redis" "solr" "s3")
    
    for experiment in "${experiments[@]}"; do
        for target_host in "${target_hosts[@]}"; do
            log "Running experiment: $experiment on $target_host"
            run_chaos_experiment "$experiment" "$target_host"
            sleep 10  # Wait between experiments
        done
    done
    
    success "All chaos engineering probes completed"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duration)
            CHAOS_DURATION="$2"
            shift 2
            ;;
        --packet-loss)
            PACKET_LOSS_RATE="$2"
            shift 2
            ;;
        --latency)
            LATENCY_MS="$2"
            shift 2
            ;;
        --slo-threshold)
            SLO_THRESHOLD="$2"
            shift 2
            ;;
        --circuit-breaker-threshold)
            CIRCUIT_BREAKER_THRESHOLD="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --duration SECONDS          Chaos duration (default: 60)"
            echo "  --packet-loss RATE          Packet loss rate (default: 0.1)"
            echo "  --latency MS                Latency in milliseconds (default: 1000)"
            echo "  --slo-threshold RATE        SLO threshold (default: 0.95)"
            echo "  --circuit-breaker-threshold COUNT  Circuit breaker threshold (default: 5)"
            echo "  --help                      Show this help message"
            exit 0
            ;;
        *)
            error_exit "Unknown option: $1"
            ;;
    esac
done

# Run main function
main "$@"
