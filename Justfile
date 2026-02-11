# Default: show help
default:
    @just --list

build-frontend:
    @cd frontend && npm install && npm run build

build-backend:
    @cd backend && cargo build --release

build: build-frontend build-backend

run-dev: build-frontend
    @cd backend && cargo run --bin otter -- --static-dir ../frontend/dist

