# ==============================================================================
# Multi-stage Dockerfile for Otter Budget Tracker
# Builds both the frontend (Vue/Vite) and backend (Rust/Axum) into a single
# container that serves the API and frontend static files.
#
# Supports multi-architecture builds (amd64 + aarch64) via TARGETPLATFORM.
# ==============================================================================

# --- Stage 1: Build frontend ---
FROM node:22-slim AS frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# --- Stage 2: Build backend ---
FROM rust:1.85-slim AS backend-builder

ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Add Rust targets for cross-compilation
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/crates/ crates/

# Set linker and target based on platform
RUN case "${TARGETPLATFORM}" in \
      "linux/arm64") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
        && cargo build --release --bin otter --target aarch64-unknown-linux-musl \
        && cp target/aarch64-unknown-linux-musl/release/otter /app/otter ;; \
      *) \
        cargo build --release --bin otter --target x86_64-unknown-linux-musl \
        && cp target/x86_64-unknown-linux-musl/release/otter /app/otter ;; \
    esac

# --- Stage 3: Final minimal image ---
FROM alpine:3.20

RUN apk add --no-cache sqlite-libs ca-certificates

WORKDIR /app

COPY --from=backend-builder /app/otter /usr/bin/otter
COPY --from=frontend-builder /app/dist /usr/share/otter/static
COPY backend/config.toml /app/config.toml

RUN mkdir -p /data

EXPOSE 3000

CMD ["otter", "--config", "/app/config.toml", "--static-dir", "/usr/share/otter/static"]
