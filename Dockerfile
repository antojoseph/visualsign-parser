# Dockerfile at repository root for easy building
# Multi-stage build for minimal image size

# Stage 1: Build
FROM rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for caching
COPY src/Cargo.toml src/Cargo.lock* ./src/
COPY api-server/Cargo.toml ./api-server/

# Copy source code
COPY src/ ./src/
COPY api-server/src/ ./api-server/src/

# Build the API server in release mode
WORKDIR /app/api-server
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies including curl for healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 apiuser

# Copy the binary from builder
COPY --from=builder /app/api-server/target/release/visualsign-api /usr/local/bin/visualsign-api

# Set ownership
RUN chown apiuser:apiuser /usr/local/bin/visualsign-api

# Switch to non-root user
USER apiuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["visualsign-api"]
