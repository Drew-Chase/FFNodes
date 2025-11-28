# syntax=docker/dockerfile:1

# =============================================================================
# Builder Stage - Compile Rust binary (ffnodes-server ONLY)
# =============================================================================
FROM --platform=$TARGETPLATFORM rust:1.91-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Skip SQLx compile-time verification (database not available during build)
ENV SQLX_OFFLINE=true

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY ffmpeg/Cargo.toml ./ffmpeg/
COPY ffnodes-server/Cargo.toml ./ffnodes-server/

# Remove client from workspace members to avoid building it
RUN sed -i '/"ffnodes-client\/src-tauri"/d' Cargo.toml

# Copy ONLY server and library source (NOT client)
COPY ffmpeg/ ./ffmpeg/
COPY ffnodes-server/ ./ffnodes-server/

# Build the actual server binary - native build for target platform
RUN cargo build --release --bin ffnodes_server && \
    cp target/release/ffnodes_server /build/ffnodes_server

# Strip binary to reduce size
RUN strip /build/ffnodes_server

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
COPY --from=builder /build/ffnodes_server /app/ffnodes_server
RUN chmod +x /app/ffnodes_server && \
    chown ffnodes:ffnodes /app/ffnodes_server

# Switch to non-root user
USER ffnodes
WORKDIR /app


# Define volumes for persistent data
VOLUME ["/app/data", "/app/logs"]

# Environment variables for runtime configuration
ENV RUST_LOG=info
ENV FFNODE_PORT=7456
ENV FFNODE_OUTPUT_CONTAINER=mp4
ENV FFNODE_WATCH_DIRECTORIES="[]"
ENV FFNODE_FFMPEG_TEMPLATE="-i {INPUT} -map 0 -c:v h264{HWACCEL_CODE} -b:v 5M -maxrate 8M -bufsize 8M -profile:v high -vf \"scale='min(1920,iw)':-2\" -c:a aac -b:a 320k {OUTPUT}"
ENV FFNODE_CLIENT_TIMEOUT_SECONDS=300
ENV FFNODE_NOTIFY_BATCH_INTERVAL_SECONDS=30

# Expose default port
EXPOSE 7456

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["wget", "--quiet", "--tries=1", "--spider", "http://localhost:${FFNODE_PORT}/api/status"]

# Entry point - run server only
ENTRYPOINT ["/app/ffnodes_server"]
