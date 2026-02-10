# ==============================================================================
# Home Assistant Add-on Dockerfile for Otter Budget Tracker
# Uses s6-overlay as the init system (HA add-on standard).
# Supports multi-architecture builds (amd64 + aarch64).
# ==============================================================================

# --- Stage 1: Build frontend ---
FROM node:22-slim AS frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# --- Stage 2: Build backend ---
FROM rust:1.84-slim AS backend-builder

ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/crates/ crates/

RUN case "${TARGETPLATFORM}" in \
      "linux/arm64") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
        && cargo build --release --bin otter --target aarch64-unknown-linux-musl \
        && cp target/aarch64-unknown-linux-musl/release/otter /app/otter ;; \
      *) \
        cargo build --release --bin otter --target x86_64-unknown-linux-musl \
        && cp target/x86_64-unknown-linux-musl/release/otter /app/otter ;; \
    esac

# --- Stage 3: HA add-on image ---
ARG BUILD_FROM=ghcr.io/home-assistant/amd64-base:3.20
FROM ${BUILD_FROM}

RUN apk add --no-cache sqlite-libs ca-certificates

COPY --from=backend-builder /app/otter /usr/bin/otter
COPY --from=frontend-builder /app/dist /usr/share/otter/static
COPY otter/rootfs/ /

RUN mkdir -p /data

# Default database URL for HA add-on (persistent storage at /data)
ENV APP__DATABASE__URL=sqlite:///data/budget.db
ENV APP__SERVER__HOST=0.0.0.0
ENV APP__SERVER__PORT=3000
