# syntax=docker/dockerfile:1

# =============================================================================
# Builder Stage - Compile Rust binary (ffnodes-server ONLY)
# =============================================================================
FROM --platform=$BUILDPLATFORM rust:1.91-bookworm AS builder

ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG TARGETARCH

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set up cross-compilation based on target architecture
RUN case "$TARGETARCH" in \
    "amd64") \
        rustup target add x86_64-unknown-linux-gnu \
        ;; \
    "arm64") \
        apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
        rustup target add aarch64-unknown-linux-gnu && \
        rm -rf /var/lib/apt/lists/* \
        ;; \
    *) \
        echo "Unsupported architecture: $TARGETARCH" && exit 1 \
        ;; \
    esac

WORKDIR /build

# Skip SQLx compile-time verification (database not available during build)
ENV SQLX_OFFLINE=true

# Copy workspace manifests first (for better layer caching)
COPY Cargo.toml Cargo.lock ./
COPY ffmpeg/Cargo.toml ./ffmpeg/
COPY ffnodes-server/Cargo.toml ./ffnodes-server/

# Create dummy source files to build dependencies (cache optimization)
RUN mkdir -p ffmpeg/src && echo "fn main() {}" > ffmpeg/src/lib.rs && \
    mkdir -p ffnodes-server/src && echo "fn main() {}" > ffnodes-server/src/main.rs && \
    echo "fn main() {}" > ffnodes-server/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN case "$TARGETARCH" in \
    "amd64") \
        cargo build --release --target x86_64-unknown-linux-gnu --bin ffnodes-server || true \
        ;; \
    "arm64") \
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
        cargo build --release --target aarch64-unknown-linux-gnu --bin ffnodes-server || true \
        ;; \
    esac

# Remove dummy files
RUN rm -rf ffmpeg/src ffnodes-server/src

# Copy ONLY server and library source (NOT client)
COPY ffmpeg/ ./ffmpeg/
COPY ffnodes-server/ ./ffnodes-server/

# Build the actual server binary
RUN case "$TARGETARCH" in \
    "amd64") \
        cargo build --release --target x86_64-unknown-linux-gnu --bin ffnodes-server && \
        cp target/x86_64-unknown-linux-gnu/release/ffnodes-server /build/ffnodes-server \
        ;; \
    "arm64") \
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
        cargo build --release --target aarch64-unknown-linux-gnu --bin ffnodes-server && \
        cp target/aarch64-unknown-linux-gnu/release/ffnodes-server /build/ffnodes-server \
        ;; \
    esac

# Strip binary to reduce size
RUN strip /build/ffnodes-server

# =============================================================================
# Runtime Stage - Minimal image with FFmpeg and server binary ONLY
# =============================================================================
FROM debian:bookworm-slim

# Install FFmpeg and CA certificates (required for HTTPS)
RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Verify FFmpeg installation
RUN ffmpeg -version && ffprobe -version

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash ffnodes && \
    mkdir -p /app/data /app/logs && \
    chown -R ffnodes:ffnodes /app

# Copy ONLY the server binary from builder (no client, no library)
COPY --from=builder /build/ffnodes-server /app/ffnodes-server
RUN chmod +x /app/ffnodes-server && \
    chown ffnodes:ffnodes /app/ffnodes-server

# Switch to non-root user
USER ffnodes
WORKDIR /app

# Expose default port
EXPOSE 8080

# Define volumes for persistent data
VOLUME ["/app/data", "/app/logs"]

# Environment variables for runtime configuration
ENV RUST_LOG=info
ENV FFNODES_PORT=8080

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["wget", "--quiet", "--tries=1", "--spider", "http://localhost:8080/api/status"]

# Entry point - run server only
ENTRYPOINT ["/app/ffnodes-server"]
