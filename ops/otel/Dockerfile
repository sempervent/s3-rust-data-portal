# BlackLake OpenTelemetry Collector Dockerfile
# Week 5: Custom OTEL collector with BlackLake configuration

FROM otel/opentelemetry-collector-contrib:0.88.0

# Copy custom configuration
COPY otel-collector-config.yaml /etc/otel-collector-config.yaml

# Override the default command to use our config
CMD ["--config=/etc/otel-collector-config.yaml"]

# Expose ports
EXPOSE 4317 4318 8889

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:13133/ || exit 1
