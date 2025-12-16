# AOF Daemon Dockerfile
# Multi-stage build for optimized production image

# Stage 1: Build
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/aof-core/Cargo.toml crates/aof-core/
COPY crates/aof-llm/Cargo.toml crates/aof-llm/
COPY crates/aof-mcp/Cargo.toml crates/aof-mcp/
COPY crates/aof-memory/Cargo.toml crates/aof-memory/
COPY crates/aof-runtime/Cargo.toml crates/aof-runtime/
COPY crates/aof-triggers/Cargo.toml crates/aof-triggers/
COPY crates/aofctl/Cargo.toml crates/aofctl/
COPY crates/smoke-test-mcp/Cargo.toml crates/smoke-test-mcp/
COPY crates/test-trigger-server/Cargo.toml crates/test-trigger-server/

# Create empty source files for dependency caching
RUN mkdir -p crates/aof-core/src && echo "// placeholder" > crates/aof-core/src/lib.rs
RUN mkdir -p crates/aof-llm/src && echo "// placeholder" > crates/aof-llm/src/lib.rs
RUN mkdir -p crates/aof-mcp/src && echo "// placeholder" > crates/aof-mcp/src/lib.rs
RUN mkdir -p crates/aof-memory/src && echo "// placeholder" > crates/aof-memory/src/lib.rs
RUN mkdir -p crates/aof-runtime/src && echo "// placeholder" > crates/aof-runtime/src/lib.rs
RUN mkdir -p crates/aof-triggers/src && echo "// placeholder" > crates/aof-triggers/src/lib.rs
RUN mkdir -p crates/aofctl/src && echo "fn main() {}" > crates/aofctl/src/main.rs
RUN mkdir -p crates/smoke-test-mcp/src && echo "fn main() {}" > crates/smoke-test-mcp/src/main.rs
RUN mkdir -p crates/test-trigger-server/src && echo "fn main() {}" > crates/test-trigger-server/src/main.rs

# Build dependencies only (this layer gets cached)
RUN cargo build --release -p aofctl 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/
COPY examples/ examples/

# Build the actual binary
RUN cargo build --release -p aofctl

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 aof

# Create directories
RUN mkdir -p /app/agents /app/config /app/checkpoints
RUN chown -R aof:aof /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/aofctl /usr/local/bin/aofctl

# Copy example configurations
COPY --from=builder /app/examples/ /app/examples/

# Switch to non-root user
USER aof

# Expose webhook server port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default environment variables
ENV RUST_LOG=info,aofctl=info,aof_runtime=info
ENV AOF_AGENTS_DIR=/app/agents
ENV AOF_CONFIG_DIR=/app/config

# Default command: start the daemon server
CMD ["aofctl", "serve", "--port", "8080", "--agents-dir", "/app/agents"]
