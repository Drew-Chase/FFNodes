# syntax=docker/dockerfile:1

# =============================================================================
# Builder Stage - Compile Rust binary (ffnodes-server ONLY)
# =============================================================================
FROM rust:1.91-alpine AS builder
#FROM --platform=$TARGETPLATFORM rust:1.91-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    curl \
    nodejs \
    npm

# Install pnpm
RUN npm install -g pnpm@8.13.1

WORKDIR /build

# Skip SQLx compile-time verification (database not available during build)
ENV SQLX_OFFLINE=true

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY ffmpeg/Cargo.toml ./ffmpeg/
COPY ffnodes-server/Cargo.toml ./ffnodes-server/

# Remove client and tools from workspace members to avoid building it
RUN sed -i '/"ffnodes-client\/src-tauri"/d' Cargo.toml
RUN sed -i '/"tools\/update_version"/d' Cargo.toml
RUN sed -i '/"tools\/publish_docker"/d' Cargo.toml

# Copy ONLY server and library source (NOT client)
# This includes all frontend source files (src/, index.html, vite.config.ts, etc.)
COPY ffmpeg/ ./ffmpeg/
COPY ffnodes-server/ ./ffnodes-server/

# Install frontend dependencies
WORKDIR /build/ffnodes-server

# Remove workspace config and node_modules to avoid pnpm workspace issues in Docker
RUN rm -f pnpm-workspace.yaml && rm -rf node_modules

# Install dependencies (force clean install without prompts)
RUN pnpm install --no-frozen-lockfile --force

# Build frontend (outputs to ../target/wwwroot)
RUN pnpm run build:frontend

# Build backend (embeds target/wwwroot via include_dir!)
WORKDIR /build
RUN cargo build --release --bin ffnodes_server && \
    cp target/release/ffnodes_server /build/ffnodes_server

# Strip binary to reduce size
RUN strip /build/ffnodes_server

# =============================================================================
# Runtime Stage - Minimal image with FFmpeg and server binary ONLY
# =============================================================================
FROM alpine:latest

# Install FFmpeg and CA certificates (required for HTTPS)
RUN apk add --no-cache \
    ffmpeg \
    ca-certificates \
    wget

# Verify FFmpeg installation
RUN ffmpeg -version && ffprobe -version

# Create non-root user for security and create directory structure
RUN adduser -D -h /home/ffnodes -s /bin/sh ffnodes && \
    mkdir -p /app /config /data && \
    chown -R ffnodes:ffnodes /app /config /data

# Copy ONLY the server binary from builder (no client, no library)
COPY --from=builder /build/ffnodes_server /app/ffnodes_server
RUN chmod +x /app/ffnodes_server && \
    chown ffnodes:ffnodes /app/ffnodes_server

# Switch to non-root user
USER ffnodes
# Set working directory to /config so config.json and app.db save here
WORKDIR /app/config

# Define volumes for persistent data
# /config - stores config.json, app.db, and logs/
# /data - default directory for media files to encode
VOLUME ["/app/config", "/app/data"]

# Expose default port
EXPOSE 7456

# Environment variables for runtime configuration
ENV RUST_LOG=info
ENV FFNODE_PORT=7456
ENV FFNODE_OUTPUT_CONTAINER=mkv
ENV FFNODE_WATCH_DIRECTORIES='["/app/data"]'
ENV FFNODE_FFMPEG_TEMPLATE="-i {INPUT} -map 0 -c:v h264{HWACCEL_CODE} -b:v 5M -maxrate 8M -bufsize 8M -profile:v high -vf \"scale='min(1920,iw)':-2\" -c:a aac -b:a 320k {OUTPUT}"
ENV FFNODE_CLIENT_TIMEOUT_SECONDS=300
ENV FFNODE_NOTIFY_BATCH_INTERVAL_SECONDS=30
#
## Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["wget", "--quiet", "--tries=1", "--spider", "http://localhost:${FFNODE_PORT}/api/status"]

# Entry point - run server only
ENTRYPOINT ["/app/ffnodes_server"]
