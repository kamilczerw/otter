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

# Import and categorize a bank CSV export, e.g.:
#   just csv-import --input statement.csv --rules-file rules.yaml
csv-import *ARGS:
    @cd backend && cargo run -p csv_import -- {{ARGS}}

