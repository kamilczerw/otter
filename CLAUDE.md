# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Task Tracking

Feature specs, implementation plans, and progress tracking live in the `tasks/` directory:

- **`tasks/1.0-budget-bar-expand/`** — Interactive Budget Bar Component: accordion expand/collapse on budget bars with inline transaction list, lazy loading, and drawer integration. Contains:
  - `SPEC.md` — Full feature specification (UX behavior, API requirements, component structure, testing)
  - `PLAN.md` — Implementation plan with phased task breakdown, dependency graph, and file-level change details
  - `STATUS.md` — Current progress tracker and codebase state notes

When working on a feature, check the corresponding `STATUS.md` for current state and `PLAN.md` for next steps.

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

### Nix (Recommended)

The project uses Nix flakes for reproducible builds and testing.

```bash
# From root directory
nix build ./nix#backend              # Build the backend binary
nix build ./nix#backend-tests-cargo  # Build and run all backend tests (REQUIRED after every task)
nix build ./nix#backend-tests        # Build and run tests with cargo-nextest
nix develop                          # Enter development shell with all tools
```

**CRITICAL: After completing ANY backend task, you MUST run:**

```bash
nix build ./nix#backend-tests-cargo
```

This ensures all tests pass and validates that your changes haven't broken existing functionality. No task is complete until tests are green.

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
  - Services: Application logic orchestrating repositories (e.g., `MonthService::create()` supports flexible month creation strategies)
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
1. Domain entities ≠ API request/response types (API layer maps between them)
1. All foreign keys use `ON DELETE RESTRICT` (deletion rules enforced in domain AND database)
1. Services depend on traits, never on concrete implementations

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
- `MonthNavigationBar` provides smart month navigation with conditional icons:
  - Left button: Navigate to previous month (arrow) or create empty month (plus)
  - Right button: Navigate to next month (arrow) or create by copying current month's entries (plus)
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

1. **Month creation strategies:** `POST /months` supports three creation strategies via optional parameters:
   - **Empty month** (`empty: true`): Creates a month with no budget entries. Used when navigating to previous months that don't exist yet.
   - **Copy from specific month** (`copy_from: "{ulid}"`): Copies all budget entries (category assignments, budgeted amounts, due dates) from the specified month. Used by the MonthNavigationBar when creating next months.
   - **Default behavior** (no parameters): Copies entries from the most recently stored month. Maintains backward compatibility with existing clients.

1. **Entry deletion requires no transactions:** You can only remove a budget entry if it has zero transactions. The API returns `409 ENTRY_HAS_TRANSACTIONS` otherwise.

1. **Category rename is retroactive:** Renaming a category updates its `name` globally; all months display the new name (no rename history in Phase 1).

1. **Transaction date is informational:** The `date` field records when payment was made but doesn't determine budget month attribution. Budget month is determined by `entry_id` → `budget_entries.month_id`.

1. **Money amounts:** Stored as integers in minor currency units (e.g., grosz for PLN, cents for USD). Currency is app configuration, not per-transaction.

## UI Style

The frontend follows a styling guide documented in `docs/STYLE_GUIDE.md` which defines the dark-mode theme, color usage, spacing, and component patterns. Key points:

- Dark, low-noise UI with bright accent for paid amounts and CTAs
- Cards with minimal elevation for grouping content
- Clear visual distinction between budgeted, paid, and remaining amounts
- One primary accent color (pink/magenta) for emphasis
- Responsive phone-first layout with bottom navigation and sensible horizontal gutters on desktop

## Coding Standards

### Clean Code Rules

All code contributions **must** follow clean code principles. The most critical rule is the **Single Responsibility Principle** for functions:

- Every function must do exactly **one thing**.
- If a function needs to perform multiple steps, break each step into its own small, focused function.
- Compose these smaller functions inside a higher-level coordinating function that orchestrates the calls.

**Bad (Rust):**

```rust
fn process_order(order: &Order) -> Result<()> {
  if order.items.is_empty() {
    return Err(OrderError::EmptyItems);
  }

  if !order.customer.email.contains('@') {
    return Err(OrderError::InvalidEmail);
  }
    // validates, calculates totals, saves to DB, and sends email
    // ...
}
```

**Good (Rust):**

```rust
fn process_order(order: &Order) -> Result<()> {
    validate_order(order)?;
    let total = calculate_order_total(order);
    save_order(order, total)?;
    send_order_confirmation(order)?;
    Ok(())
}
```

**Bad (TypeScript):**

```typescript
function processOrder(order: Order): void {
  // validates, calculates totals, saves to DB, and sends email
  // ...
}
```

**Good (TypeScript):**

```typescript
function processOrder(order: Order): void {
  validateOrder(order);
  const total = calculateOrderTotal(order);
  saveOrder(order, total);
  sendOrderConfirmation(order);
}
```

Additional clean code expectations:

- Functions should have clear, descriptive names that convey intent.
- Keep functions short. If you need to scroll to read it, it's too long.
- Avoid side effects that aren't obvious from the function name.
- Minimize function arguments — prefer fewer than three.
- Don't repeat yourself. Extract shared logic into reusable functions.

### Testing Requirements

All code **must** be tested after every task. This is non-negotiable.

**CRITICAL: After completing ANY task (adding a feature, fixing a bug, refactoring code), you MUST:**

1. **Run the full test suite immediately** using `nix build ./nix#backend-tests-cargo` (for backend) or `npm run type-check` (for frontend)
2. **Verify all tests pass** before moving to the next task or considering the work complete
3. **Never mark a task as done** until tests are green

Additional testing requirements:

- Write unit tests for every new function.
- Ensure all existing tests still pass after your changes.
- Cover both the happy path and meaningful edge cases.
- If a bug is being fixed, add a regression test that reproduces the bug first, then verify the fix makes it pass.

No task is complete until the test suite passes. No PR will be considered ready for review until tests are green.

### Documentation

All code **must** be documented using doc comments. Undocumented code will not be accepted.

- Every function, method, struct, class, and interface must have a doc comment explaining its purpose.
- Doc comments should describe **what** the function does and **why**, not just restate the function name.
- Document all parameters, return values, and any errors that may be returned or thrown (Rust `# Errors` section, TypeScript `@throws`).
- If a function has non-obvious behavior, edge cases, or assumptions, call them out explicitly.
- Keep doc comments concise but complete — one clear sentence is better than a vague paragraph.

**Bad (Rust):**

```rust
// calculates total
fn calculate_total(items: &[OrderItem]) -> Decimal {
    // ...
}
```

**Good (Rust):**

```rust
/// Calculates the total price for a list of order items, including tax.
///
/// # Arguments
///
/// * `items` - The order items to sum. Must not be empty.
///
/// # Returns
///
/// The total price as a `Decimal`, rounded to two decimal places.
///
/// # Errors
///
/// Returns `OrderError::EmptyItems` if the items slice is empty.
fn calculate_total(items: &[OrderItem]) -> Result<Decimal, OrderError> {
    // ...
}
```

**Bad (TypeScript):**

```typescript
// calculates total
function calculateTotal(items: OrderItem[]): number {
  // ...
}
```

**Good (TypeScript):**

```typescript
/**
 * Calculates the total price for a list of order items, including tax.
 *
 * @param items - The order items to sum. Must not be empty.
 * @returns The total price, rounded to two decimal places.
 * @throws {EmptyItemsError} If the items array is empty.
 */
function calculateTotal(items: OrderItem[]): number {
  // ...
}
```

- For Rust, include module-level `//!` doc comments summarizing the module's responsibility. For TypeScript, include a top-of-file JSDoc comment for the same purpose.
- Update doc comments whenever the function's behavior changes. Stale documentation is worse than no documentation.

## Testing

**Backend (Nix - Recommended):**

- **Run all tests (REQUIRED after every task):** `nix build ./nix#backend-tests-cargo`
- Alternative with nextest: `nix build ./nix#backend-tests`

**Backend (Cargo - Alternative):**

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

**IMPORTANT:** Always use Nix for testing when available. It ensures reproducible builds and catches issues that might not appear in local cargo runs.

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
