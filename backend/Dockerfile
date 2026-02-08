FROM rust:1.84-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/
RUN cargo build --release --bin api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/api /app/api
COPY config.toml /app/config.toml
RUN mkdir -p /data
EXPOSE 3000
CMD ["/app/api"]
