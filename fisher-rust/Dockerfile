# Multi-stage build for optimized production image
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release --bin fisher-relayer

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 fisher && \
    mkdir -p /app /data && \
    chown -R fisher:fisher /app /data

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/fisher-relayer /usr/local/bin/

# Copy configuration template
COPY config.example.json /app/config.json

USER fisher

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/fisher-relayer", "--health-check"]

ENTRYPOINT ["/usr/local/bin/fisher-relayer"]
CMD ["--config", "/data/config.json"]
