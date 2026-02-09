# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Family Budget Tracker — a web application for household budget management with a Vue 3 frontend and Rust/Axum backend following hexagonal architecture.

**Key characteristics:**
- Backend: Rust workspace with 3 crates (`domain`, `db`, `api`) following ports & adapters pattern
- Frontend: Vue 3 + TypeScript + Vuetify (Material Design)
- Database: SQLite with SQLx (compile-time checked queries)
- Deployment: Docker Compose with nginx reverse proxy

## Development Commands

### Backend (Rust)

```bash
# From backend/ directory
cargo build                          # Build all workspace members
cargo test                           # Run all tests (unit + integration)
cargo run --bin api                  # Run the API server
cargo check                          # Fast check without building

# SQLx database operations
sqlx migrate run                     # Run pending migrations
sqlx migrate revert                  # Revert last migration
sqlx prepare                         # Update sqlx-data.json for offline builds

# Individual crate operations
cargo test -p domain                 # Test domain crate only
cargo build -p api                   # Build API crate only
```

### Frontend (Vue + TypeScript)

```bash
# From frontend/ directory
npm run dev                          # Start Vite dev server (default: port 5173)
npm run build                        # Type-check and build for production
npm run preview                      # Preview production build
npm run type-check                   # Run Vue type checking without build
```

### Docker

```bash
# From root directory
docker compose up --build            # Build and start all services
docker compose up -d                 # Start in background
docker compose down                  # Stop and remove containers
docker compose logs -f backend       # Follow backend logs
docker compose logs -f frontend      # Follow frontend/nginx logs
```

## Architecture

### Backend: Hexagonal Architecture

The backend is organized as a Rust workspace with strict dependency rules:

**Dependency flow:** `api` → `domain` ← `db`

- **`domain` crate**: Pure business logic with NO framework dependencies
  - Domain types (newtypes): `Money`, `BudgetMonth`, `CategoryName`, `DueDay`, `TransactionDate`, `Ulid`
  - All newtypes validate on construction, making invalid states unrepresentable
  - Entities: `Category`, `Month`, `BudgetEntry`, `Transaction`
  - Ports (traits): `CategoryRepository`, `MonthRepository`, `BudgetEntryRepository`, `TransactionRepository`
  - Services: Application logic orchestrating repositories (e.g., `MonthService::create()` copies entries from latest month)
  - Error enums: Each module defines focused error types (`CategoryError`, `MonthError`, etc.)

- **`db` crate**: SQLite adapter implementing domain ports
  - Repository implementations using SQLx
  - Migrations in `crates/db/migrations/`
  - Database-specific type mappings
  - **IMPORTANT**: All connections MUST run `PRAGMA foreign_keys = ON`

- **`api` crate**: HTTP/REST layer
  - Axum handlers in `src/handlers/`
  - Request/response types (separate from domain entities)
  - Error mapping: domain errors → HTTP status codes + structured JSON
  - Middleware: request ID, structured logging, CORS
  - OpenAPI spec generation via `utoipa`
  - Entry point: `src/main.rs` wires everything together

**Key architectural rules:**
1. Domain crate has NO dependencies on frameworks, databases, or HTTP libraries
2. Domain entities ≠ API request/response types (API layer maps between them)
3. All foreign keys use `ON DELETE RESTRICT` (deletion rules enforced in domain AND database)
4. Services depend on traits, never on concrete implementations

### Database Schema

All tables use ULID as TEXT primary key. All timestamps are UTC RFC 3339 with `Z` suffix.

**Tables:**
- `categories`: Global category definitions (hierarchical names like `utils/electricity`)
- `months`: Budget months (ULID + `YYYY-MM` string)
- `budget_entries`: Category assignments per month (links month + category, stores budgeted amount + due day)
- `transactions`: Recorded payments against budget entries

**Important constraints:**
- `UNIQUE(month_id, category_id)` on `budget_entries` — category appears max once per month
- All foreign keys use `ON DELETE RESTRICT`
- `updated_at` managed by SQLite triggers (application sets only `created_at`)

### Frontend: Vue 3 Composition API

**Structure:**
- `src/api/`: Typed fetch wrapper + API client functions
- `src/components/`: Reusable components organized by domain (charts, entries, transactions, layout)
- `src/views/`: Top-level route views
- `src/router/`: Vue Router with human-readable URLs (`/months/2026-02/budget`)
- `src/i18n/`: Internationalization (PL + EN) via vue-i18n
- `src/theme/`: Custom Vuetify theme

**Key patterns:**
- Routes use `YYYY-MM` month strings; frontend resolves to backend ULIDs via cached month list
- Bottom navigation as primary navigation (mobile-first)
- `TransactionDrawer` uses `v-bottom-sheet` for in-context editing
- `CategoryAutocomplete` allows inline creation of new categories
- Charts use Chart.js via vue-chartjs, data from `/months/{id}/summary` endpoint

## API Conventions

**Base URL:** `/api/v1` (proxied via nginx in production)

**Resource identification:** All endpoints use ULID in URL paths (not month strings or category names)

**Error responses:** Structured JSON with machine-readable codes
```json
{
  "error": {
    "code": "ENTRY_HAS_TRANSACTIONS",
    "details": { "transaction_count": 3 }
  }
}
```

Frontend maps error codes to translated messages via vue-i18n.

**Default sort orders:**
- Categories: `name` ascending
- Months: `month` descending (most recent first)
- Entries: `due_day` ascending (nulls last), then `category.name` ascending
- Transactions: `date` descending, then `created_at` descending

## Important Domain Rules

1. **Month creation copies entries:** `POST /months` creates a new month and copies all budget entries (category assignments, budgeted amounts, due dates) from the most recently stored month.

2. **Entry deletion requires no transactions:** You can only remove a budget entry if it has zero transactions. The API returns `409 ENTRY_HAS_TRANSACTIONS` otherwise.

3. **Category rename is retroactive:** Renaming a category updates its `name` globally; all months display the new name (no rename history in Phase 1).

4. **Transaction date is informational:** The `date` field records when payment was made but doesn't determine budget month attribution. Budget month is determined by `entry_id` → `budget_entries.month_id`.

5. **Money amounts:** Stored as integers in minor currency units (e.g., grosz for PLN, cents for USD). Currency is app configuration, not per-transaction.

## Testing

**Backend:**
- Unit tests: `cargo test -p domain` (fast, no dependencies)
- Integration tests: `cargo test -p api` (full HTTP + SQLite test database)
- Run all: `cargo test`

**Frontend:**
- Type checking: `npm run type-check`
- E2E tests use Playwright (not yet implemented in Phase 1)

**Test strategy:**
- Domain tests focus on newtype validation, service logic, and error cases
- API integration tests verify full HTTP request/response cycles against real database
- E2E tests verify user flows through browser

## Configuration

Backend uses TOML config file (`backend/config.toml`) with environment variable overrides.

**Override pattern:** `APP__SECTION__KEY` (double underscore)

Example: `APP__DATABASE__URL=sqlite:///data/budget.db`

**Key settings:**
- `server.host` / `server.port`: API server binding
- `database.url`: SQLite connection string
- `currency.code` / `currency.decimal_places`: Currency display settings
- `cors.allowed_origins`: CORS configuration (minimal in production with nginx)

## Production Deployment

**Default model:** nginx reverse proxy
- nginx serves frontend static files
- nginx proxies `/api/*` to backend
- Same-origin setup eliminates CORS complexity
- Single exposed port

**Docker:**
- Backend container: Multi-stage Rust build → minimal Debian slim
- Frontend container: Multi-stage Node build → nginx serving static files + proxy config
- SQLite database mounted as volume for persistence

**Data backup:** Copy the SQLite database file (`data/budget.db` in Docker volume)

## File Naming Conventions

**Backend:**
- Services: `{entity}_service.rs` (e.g., `month_service.rs`)
- Repositories: `{entity}_repo.rs` (e.g., `category_repo.rs`)
- Entities: `{entity}.rs` (e.g., `budget_entry.rs`)
- Types: Descriptive names (e.g., `money.rs`, `budget_month.rs`)

**Frontend:**
- Views: `{Entity}{Action}View.vue` (e.g., `MonthListView.vue`)
- Components: `{Entity}{Purpose}.vue` (e.g., `TransactionDrawer.vue`)
- API modules: `{entity}.ts` (e.g., `transactions.ts`)

## Code Style

**Rust:**
- Use `thiserror` for error enum derivation
- Use `async-trait` for async repository traits
- Prefer compile-time checked SQLx queries (`query_as!`)
- Use `tracing` for structured logging with request IDs

**TypeScript/Vue:**
- Composition API (not Options API)
- Typed API client functions (types generated from OpenAPI spec)
- All user-facing strings via `$t()` i18n function
- Vuetify components for UI consistency
