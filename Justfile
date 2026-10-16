
watch_dirs := "backend/ frontend/"

# Default: show help
default:
    @just --list

build-frontend:
    @cd frontend && npm install && npm run build

build-backend:
    @cd backend && cargo build --release

build: build-frontend build-backend

localConfigFile := "./config/config.local.toml"
config := if path_exists("backend/" + localConfigFile) == "true" { localConfigFile } else { "./config.toml" }

run-dev: build-frontend
    @echo "Config file: {{config}}"
    @cd backend && cargo run --bin otter -- --static-dir ../frontend/dist --config {{config}}

# Import and categorize a bank CSV export, e.g.:
#   just csv-import --input statement.csv --rules-file rules.toml
csv-import *ARGS:
    @cd backend && cargo run -p csv_import -- {{ARGS}}

watch:
    @cd backend && cargo watch -c -q {{ prepend("-w ", watch_dirs) }} -s 'just run-dev'
