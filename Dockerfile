# Multi-stage Dockerfile for Blacklake API
FROM rustlang/rust:nightly as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY migrations/ ./migrations/

# Build dependencies (cached layer)
RUN cargo build --release --workspace --all-features

# Build the application
RUN cargo build --release --workspace --all-features

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r blacklake && useradd -r -g blacklake blacklake

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/blacklake-api /app/blacklake-api

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Change ownership
RUN chown -R blacklake:blacklake /app

# Switch to non-root user
USER blacklake

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/live || exit 1

# Run the application
CMD ["./blacklake-api"]
