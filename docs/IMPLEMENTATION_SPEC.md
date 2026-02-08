# Family Budget Tracker — Implementation Specification

**Version:** 1.1 (Phase 1)
**Date:** February 2026
**Based on:** AppSpec v1.1

-----

## 1. Overview

This document defines the technical implementation plan for the Family Budget Tracker application as described in the Functional Specification (AppSpec v1.1). It covers the technology stack, architecture, data model, API contracts, frontend structure, error handling, testing strategy, configuration, and deployment.

The guiding principle throughout is **simplicity that scales** — every decision favors the simplest viable approach while leaving room for future expansion without rewrites.

### 1.1 Terminology Note

The AppSpec uses the term "payment" for recorded actual spending. This implementation renames it to **"transaction"** throughout the codebase, API, and UI. The domain meaning is identical — a recorded actual payment against a budget entry. The rename better reflects future extensibility (e.g., refunds, transfers).

-----

## 2. Technology Stack

### 2.1 Backend

The backend is a Rust application built on the Axum web framework. Rust was chosen for its type safety, performance, and the team's existing expertise.

**Core dependencies:**

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Language | Rust (latest stable) | Type safety, performance, team expertise |
| Web framework | Axum | Lightweight, async-native, composable |
| Database | SQLite | Zero-process deployment, sufficient for household scale, swappable to Postgres later |
| SQL toolkit | SQLx | Compile-time query checking, async, supports both SQLite and Postgres |
| Migrations | sqlx migrate | Built into SQLx, no additional tooling |
| Configuration | `config` crate | TOML file with environment variable overrides |
| ID generation | ULID | Sortable, unique, URL-safe, used as primary keys across all tables |
| OpenAPI | `utoipa` crate | OpenAPI spec generated from Rust types, used as source for frontend TypeScript types |
| Logging | `tracing` + `tracing-subscriber` | Structured logging with request ID propagation |

### 2.2 Frontend

The frontend is a separate Vue 3 single-page application written in TypeScript.

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Framework | Vue 3 (Composition API) | Team familiarity, gentle learning curve |
| Language | TypeScript | Non-negotiable type safety across the stack |
| UI framework | Vuetify 3 | Material Design component library, mobile-first, highly themeable |
| Charts | Chart.js (via vue-chartjs) | Lightweight, covers required bar and donut charts |
| i18n | vue-i18n | Standard Vue internationalization, supports PL and EN |
| Routing | Vue Router | Standard client-side routing |
| API client | Thin typed fetch wrapper | No external HTTP library dependency, portable |
| API types | Generated from OpenAPI spec | Ensures frontend types always match backend contract |

### 2.3 Testing

| Layer | Tool | Strategy |
|-------|------|----------|
| Backend domain | Rust built-in (`#[cfg(test)]`) | Unit tests for domain newtypes, validation, and service logic |
| Backend API | Rust integration tests | Full HTTP request/response tests against a test SQLite database |
| Frontend E2E | Playwright | End-to-end user flow tests running against the real API |

### 2.4 Deployment

The application is self-hosted on a home lab and can be deployed as either a Docker container or directly on a NixOS VM. The default deployment model uses **nginx as a reverse proxy** serving the frontend and proxying API requests to the backend, enabling same-origin requests and simplifying CORS (see Section 10).

-----

## 3. Architecture

### 3.1 High-Level Architecture

The system follows a **backend/frontend split** architecture. The Vue frontend communicates with the Axum backend exclusively through a REST API. In production, both are served through a single nginx reverse proxy: static frontend assets are served directly, and `/api/*` requests are proxied to the backend. This same-origin setup eliminates most CORS concerns.

### 3.2 Backend Architecture — Hexagonal / Ports & Adapters

The backend follows **hexagonal architecture** (ports and adapters) with clean code principles. The codebase is organized as a Rust workspace with three crates, each with a clearly defined responsibility and dependency direction.

**Dependency direction:** `api` → `domain` ← `db`

The `domain` crate has **no framework, database, or HTTP dependencies**. Small utility crates are allowed where required by Rust language constraints or maintainability (e.g. `async-trait` for async port definitions, `thiserror` for ergonomic error enums). Both `api` and `db` depend on `domain`, but never on each other.

#### 3.2.1 `domain` Crate

This is the heart of the application. It contains:

**Domain types (newtypes):** Every domain primitive is a newtype that validates on construction and returns a `DomainError` on failure. This makes invalid states unrepresentable at compile time and eliminates the "two arguments of the same type" class of bugs.

| Newtype | Inner Representation | Validation |
|---------|---------------------|------------|
| `Money` | `i64` | No constraints in Phase 1. Represents amount in the minor currency unit (e.g. grosz for PLN, cents for USD). Currency is determined by app configuration, not by this type. Negative values are allowed at the domain/DB level; the UI restricts entry to non-negative in Phase 1. |
| `BudgetMonth` | `{ year: i32, month: u8 }` | Year must be reasonable (e.g. 2000–2100). Month must be 1–12. Serialized to/from `YYYY-MM` string at the API boundary. |
| `DueDay` | `u8` | Range 1–31. |
| `CategoryName` | `String` | Non-empty. Uses `/` as hierarchy separator. No leading or trailing `/`. No empty segments (e.g. `a//b` is invalid). Allowed characters: alphanumeric, hyphens, underscores, and `/` as separator. Must have at least one segment. |
| `TransactionDate` | Date type (e.g. `chrono::NaiveDate` or equivalent) | Must be a valid calendar date. Serialized to/from `YYYY-MM-DD` string at the API boundary. |
| `Ulid` | ULID type | Valid ULID format. |

**Domain entities:** Structs representing categories, months, budget entries, and transactions. These use the newtypes above for their fields.

**Traits (ports):** Repository traits that define the interface for data persistence. These use `async-trait` (or an equivalent pattern) to define async methods. For example:

```rust
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<Category>, CategoryError>;
    async fn find_by_id(&self, id: &Ulid) -> Result<Option<Category>, CategoryError>;
    async fn create(&self, category: NewCategory) -> Result<Category, CategoryError>;
    async fn update_name(&self, id: &Ulid, name: CategoryName) -> Result<Category, CategoryError>;
}
```

Similar traits exist for `MonthRepository`, `BudgetEntryRepository`, and `TransactionRepository`.

**Services (use cases):** Application logic that orchestrates operations using the repository traits. Services live in the `domain` crate and depend only on traits, never on concrete implementations. Examples of service operations include creating a new month (which involves copying entries from the latest existing month), computing a month summary, and validating that a budget entry can be removed (no linked transactions).

**Error enums:** Each domain module defines its own error enum (using `thiserror` for ergonomic derivation). This keeps error types focused and makes them portable to other applications in the future.

| Error Enum | Examples of Variants |
|------------|---------------------|
| `CategoryError` | `NameAlreadyExists`, `InvalidNameFormat`, `NotFound` |
| `MonthError` | `AlreadyExists`, `InvalidFormat`, `NotFound`, `NoSourceMonthForCopy` |
| `EntryError` | `CategoryAlreadyInMonth`, `NotFound`, `HasTransactions` |
| `TransactionError` | `InvalidAmount`, `NotFound`, `EntryNotFound` |

#### 3.2.2 `db` Crate

This crate contains the **SQLx adapter** — concrete implementations of the domain repository traits using SQLx and SQLite.

Contents include:

**Repository implementations:** Structs that hold an `SqlitePool` and implement the domain traits. All SQL queries live here, using SQLx's compile-time checked queries where possible.

**Migrations:** SQL migration files managed by `sqlx migrate`. The migration directory lives within this crate.

**Database-specific type mappings:** Conversions between SQLite types and domain newtypes (e.g. `TEXT` ↔ `Ulid`, `INTEGER` ↔ `Money`, `TEXT YYYY-MM` ↔ `BudgetMonth { year, month }`).

**Database initialization:** Connection setup including enabling `PRAGMA foreign_keys = ON` on every connection (see Section 4).

#### 3.2.3 `api` Crate

This is the application's entry point and the **driving adapter**. It contains:

**Axum handlers:** HTTP request handlers that deserialize incoming JSON, call domain services, and serialize responses. Each handler receives domain services via Axum's dependency injection (shared state).

**Request/response types:** Serde-annotated structs for JSON serialization/deserialization, with `utoipa` derive macros for OpenAPI generation. These are separate from domain entities — the API layer maps between them. This is the boundary where strings like `YYYY-MM` are parsed into typed domain representations and vice versa.

**Error mapping:** Conversion from domain errors to HTTP responses (status code + structured error JSON).

**Middleware:** Request ID generation and propagation, structured logging (request ID, route, status, latency), and CORS configuration.

**Wiring:** The `main` function assembles the application: creates the database pool, constructs the repository implementations, injects them into domain services, and starts the Axum server.

**Configuration loading:** Reads the TOML config file and environment variable overrides, then makes configuration available to other layers.

**OpenAPI generation:** The `utoipa` crate generates an OpenAPI spec from the request/response types and handler annotations. The generated `openapi.json` is committed to the repository and used to generate frontend TypeScript types.

### 3.3 Workspace Layout

```
backend/
├── Cargo.toml              # Workspace root
├── config.toml             # Default configuration
├── openapi.json            # Generated OpenAPI spec (committed)
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/      # Newtypes (Money, BudgetMonth, etc.)
│   │       ├── entities/   # Category, Month, BudgetEntry, Transaction
│   │       ├── ports/      # Repository traits
│   │       ├── services/   # Use case implementations
│   │       └── errors/     # Error enums per module
│   ├── db/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repos/      # Trait implementations
│   │       └── migrations/ # SQL migration files
│   └── api/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs     # Entry point, wiring
│           ├── handlers/   # Axum route handlers
│           ├── requests/   # Request deserialization types
│           ├── responses/  # Response serialization types
│           ├── errors.rs   # Domain → HTTP error mapping
│           ├── middleware.rs # Request ID, logging
│           └── config.rs   # Configuration loading
```

-----

## 4. Database Schema

All tables use ULID as the primary key (stored as `TEXT`). All timestamp fields (`created_at`, `updated_at`) are stored as `TEXT` in **UTC, formatted as RFC 3339 / ISO 8601 with the `Z` suffix** (e.g. `2026-02-07T12:00:00Z`). Every table includes `created_at` and `updated_at` timestamps.

**`created_at`** is set on insert by the application. **`updated_at`** is managed automatically via SQLite `BEFORE UPDATE` triggers — the application does not set it manually. This ensures consistency and reduces code logic. If the application is later migrated to Postgres, equivalent triggers can be created there.

**Foreign key enforcement:** Every database connection must execute `PRAGMA foreign_keys = ON` immediately after opening. This is configured at the connection pool level in the `db` crate. Without this pragma, SQLite silently ignores foreign key constraints.

**All foreign keys use `ON DELETE RESTRICT`** to prevent accidental orphaning. Deletion rules are enforced at both the domain level (service logic) and the database level (constraints) for defense in depth.

### 4.1 `categories`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | ULID |
| `name` | TEXT | NOT NULL, UNIQUE | Hierarchical path (e.g. `utils/electricity`) |
| `created_at` | TEXT | NOT NULL | UTC RFC 3339 datetime |
| `updated_at` | TEXT | NOT NULL | UTC RFC 3339 datetime (trigger-managed) |

Renaming a category updates its `name` across the system. All months and transactions referencing that category will display the new name. No rename history is maintained in Phase 1 — renames are retroactive.

### 4.2 `months`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | ULID |
| `month` | TEXT | NOT NULL, UNIQUE | Format `YYYY-MM` (e.g. `2026-02`) |
| `created_at` | TEXT | NOT NULL | UTC RFC 3339 datetime |
| `updated_at` | TEXT | NOT NULL | UTC RFC 3339 datetime (trigger-managed) |

The `month` column is a human-readable identifier used by the frontend for display and routing. The backend API uses `id` (ULID) for all resource identification in URL paths.

### 4.3 `budget_entries`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | ULID |
| `month_id` | TEXT | NOT NULL, REFERENCES months(id) ON DELETE RESTRICT | Parent month |
| `category_id` | TEXT | NOT NULL, REFERENCES categories(id) ON DELETE RESTRICT | Linked global category |
| `budgeted` | INTEGER | NOT NULL | Budgeted amount in minor currency units |
| `due_day` | INTEGER | NULLABLE | Day of month (1–31) when payment is expected |
| `created_at` | TEXT | NOT NULL | UTC RFC 3339 datetime |
| `updated_at` | TEXT | NOT NULL | UTC RFC 3339 datetime (trigger-managed) |

Additional constraint: `UNIQUE(month_id, category_id)` — a category can appear at most once per month.

### 4.4 `transactions`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | ULID |
| `entry_id` | TEXT | NOT NULL, REFERENCES budget_entries(id) ON DELETE RESTRICT | Parent budget entry |
| `amount` | INTEGER | NOT NULL | Amount in minor currency units. No CHECK constraint in Phase 1 — the UI restricts entry to non-negative values, but the domain and DB layer allow any integer to simplify future refund support. |
| `date` | TEXT | NOT NULL | Payment date in `YYYY-MM-DD` format |
| `created_at` | TEXT | NOT NULL | UTC RFC 3339 datetime |
| `updated_at` | TEXT | NOT NULL | UTC RFC 3339 datetime (trigger-managed) |

Note: The `date` field is informational only — it records when the payment was made but does not determine which budget month the transaction belongs to. Budget month attribution is determined by the `entry_id` → `budget_entries.month_id` relationship.

### 4.5 Indexes

The following indexes are created to support expected query patterns and ensure efficient joins:

| Index | Table | Columns | Rationale |
|-------|-------|---------|-----------|
| `idx_budget_entries_month_id` | budget_entries | `month_id` | Listing entries by month |
| `idx_budget_entries_category_id` | budget_entries | `category_id` | Looking up entries by category across months |
| `idx_transactions_entry_id` | transactions | `entry_id` | Listing transactions by budget entry |

### 4.6 Triggers

Each table has an `updated_at` trigger:

```sql
CREATE TRIGGER trg_categories_updated_at
BEFORE UPDATE ON categories
BEGIN
    UPDATE categories SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;
```

Equivalent triggers are created for `months`, `budget_entries`, and `transactions`. The `strftime` format produces UTC timestamps in the canonical RFC 3339 format with `Z` suffix.

-----

## 5. API Design

### 5.1 General Conventions

**Base URL:** Configurable (e.g. `http://localhost:3000/api/v1`). In production, the nginx reverse proxy maps `/api/*` to the backend, so the frontend accesses the API at the same origin.

**Content type:** `application/json` for all requests and responses.

**Resource identification:** All endpoints use the resource's ULID (`id`) in URL paths. There is no slug-based or natural-key-based routing in the API. The `month` field (e.g. `2026-02`) is a data attribute, not a route parameter.

**Response envelope:** Successful responses return the resource or resource array directly (no wrapper). Error responses follow the structured error format described in Section 6.

**Timestamps:** All responses include `created_at` and `updated_at` as UTC RFC 3339 strings with `Z` suffix.

**Request IDs:** Every request is assigned a unique request ID via middleware. The request ID is included in the response header `X-Request-Id` and in all structured log entries for that request.

### 5.2 Default Sort Orders

All list endpoints return results in a deterministic order. The default sort orders are:

| Endpoint | Sort Order |
|----------|------------|
| `GET /categories` | `name` ascending |
| `GET /months` | `month` descending (most recent first) |
| `GET /months/{id}/entries` | `due_day` ascending (nulls last), then `category.name` ascending |
| `GET /transactions` | `date` descending, then `created_at` descending |
| `GET /months/{id}/summary` categories array | Same as entries: `due_day` ascending (nulls last), then `category.name` ascending |

Pagination is not implemented in Phase 1. All results are returned in full.

### 5.3 Endpoints

#### 5.3.1 Categories

**`GET /categories`** — List all global categories.

Response `200`:

```json
[
  {
    "id": "01HXK3D7W8ABCDEF12345678",
    "name": "utils/electricity",
    "created_at": "2026-02-07T12:00:00Z",
    "updated_at": "2026-02-07T12:00:00Z"
  }
]
```

**`POST /categories`** — Create a new global category.

Request:

```json
{
  "name": "utils/electricity"
}
```

Response `201`: Same shape as a single category object above.

Errors: `409 CATEGORY_NAME_ALREADY_EXISTS` if a category with the same name already exists. `422 CATEGORY_INVALID_NAME` if the name format is invalid.

**`PATCH /categories/{id}`** — Rename a global category. The `{id}` parameter accepts a ULID.

Request:

```json
{
  "name": "utilities/electricity"
}
```

Response `200`: Updated category object.

Errors: `404 CATEGORY_NOT_FOUND` if category not found. `409 CATEGORY_NAME_ALREADY_EXISTS` if new name conflicts with an existing category. `422 CATEGORY_INVALID_NAME` if the name format is invalid.

#### 5.3.2 Months

**`GET /months`** — List all budget months (most recent first).

Response `200`:

```json
[
  {
    "id": "01HXK3D7W8ABCDEF12345678",
    "month": "2026-02",
    "created_at": "2026-02-07T12:00:00Z",
    "updated_at": "2026-02-07T12:00:00Z"
  }
]
```

**`POST /months`** — Create a new budget month. Budget entries (category assignments, budgeted amounts, and due dates) are copied from the most recently stored month. If no months exist yet, the month is created empty.

Request:

```json
{
  "month": "2026-03"
}
```

Response `201`: Month object.

Errors: `409 MONTH_ALREADY_EXISTS` if the month already exists. `422 MONTH_INVALID_FORMAT` if the month format is invalid.

**`GET /months/{id}`** — Get a single month by its ULID.

Response `200`: Month object.

Errors: `404 MONTH_NOT_FOUND` if month not found.

#### 5.3.3 Budget Entries

**`GET /months/{id}/entries`** — List all budget entries for a month. Each entry includes its category inlined.

Response `200`:

```json
[
  {
    "id": "01HXK3D7W8ABCDEF12345678",
    "category": {
      "id": "01HXK3D7W8ABCDEF12345678",
      "name": "housing/mortgage"
    },
    "budgeted": 300000,
    "due_day": 10,
    "created_at": "2026-02-07T12:00:00Z",
    "updated_at": "2026-02-07T12:00:00Z"
  }
]
```

**`POST /months/{id}/entries`** — Add a category to a month by creating a budget entry.

Request:

```json
{
  "category_id": "01HXK3D7W8ABCDEF12345678",
  "budgeted": 300000,
  "due_day": 10
}
```

The `due_day` field is optional and may be omitted or set to `null`.

Response `201`: Budget entry object with inlined category.

Errors: `404 MONTH_NOT_FOUND` if the month does not exist. `404 CATEGORY_NOT_FOUND` if the category does not exist. `409 ENTRY_CATEGORY_ALREADY_IN_MONTH` if the category is already linked to this month. `422 ENTRY_INVALID_DUE_DAY` if the due day is outside 1–31.

**`PATCH /months/{id}/entries/{entry_id}`** — Update the budgeted amount or due date of an entry.

Request (all fields optional):

```json
{
  "budgeted": 350000,
  "due_day": 15
}
```

Response `200`: Updated budget entry object with inlined category.

Errors: `404 ENTRY_NOT_FOUND` if entry not found. `422 ENTRY_INVALID_DUE_DAY` if the due day is outside 1–31.

**`DELETE /months/{id}/entries/{entry_id}`** — Remove a category from a month. This operation is only permitted if there are no transactions recorded against this entry.

Response `204`: No content.

Errors: `404 ENTRY_NOT_FOUND` if entry not found. `409 ENTRY_HAS_TRANSACTIONS` if the entry has transactions (response includes transaction count in error details).

#### 5.3.4 Transactions

**`GET /transactions`** — List transactions. The `month` query parameter is **required** in Phase 1.

| Query Parameter | Type | Required | Description |
|-----------------|------|----------|-------------|
| `month` | String (ULID) | Yes (Phase 1) | Filter by budget month ID |

If the `month` parameter is missing, the API returns `400 TRANSACTIONS_MONTH_REQUIRED`.

Response `200`:

```json
[
  {
    "id": "01HXK3D7W8ABCDEF12345678",
    "entry_id": "01HXK3D7W8ABCDEF12345678",
    "amount": 15000,
    "date": "2026-02-05",
    "created_at": "2026-02-07T12:00:00Z",
    "updated_at": "2026-02-07T12:00:00Z"
  }
]
```

Note: The response returns `entry_id` only, not inlined entry/category data. The frontend is expected to have already loaded entries from the entries endpoint. A future `?expand=entry` query parameter may be added for cross-month search use cases.

**`POST /transactions`** — Record a new transaction.

Request:

```json
{
  "entry_id": "01HXK3D7W8ABCDEF12345678",
  "amount": 15000,
  "date": "2026-02-05"
}
```

Response `201`: Transaction object.

Errors: `404 TRANSACTION_ENTRY_NOT_FOUND` if the entry does not exist. `422 TRANSACTION_INVALID_AMOUNT` if the amount is negative (UI-level restriction in Phase 1). `422 TRANSACTION_INVALID_DATE` if the date format is invalid.

**`PATCH /transactions/{id}`** — Edit a transaction. All fields are optional. Setting `entry_id` to a different entry moves the transaction to another category within the same month (cross-month moves are not supported in Phase 1).

Request (all fields optional):

```json
{
  "entry_id": "01HXK3D7W8ABCDEF12345678",
  "amount": 16000,
  "date": "2026-02-06"
}
```

Response `200`: Updated transaction object.

Errors: `404 TRANSACTION_NOT_FOUND` if transaction not found. `404 TRANSACTION_ENTRY_NOT_FOUND` if the target entry not found. `422 TRANSACTION_INVALID_AMOUNT` if the amount is negative. `422 TRANSACTION_INVALID_DATE` if the date format is invalid.

**`DELETE /transactions/{id}`** — Delete a transaction.

Response `204`: No content.

Errors: `404 TRANSACTION_NOT_FOUND` if transaction not found.

#### 5.3.5 Month Summary

**`GET /months/{id}/summary`** — Get computed budget summary for a month. All computation is performed in the domain layer.

Response `200`:

```json
{
  "month": "2026-02",
  "total_budgeted": 500000,
  "total_paid": 320000,
  "remaining": 180000,
  "categories": [
    {
      "category": {
        "id": "01HXK3D7W8ABCDEF12345678",
        "name": "housing/mortgage"
      },
      "budgeted": 300000,
      "paid": 300000,
      "remaining": 0,
      "status": "on_budget"
    },
    {
      "category": {
        "id": "01HXK3D7W8ABCDEF12345678",
        "name": "utils/electricity"
      },
      "budgeted": 20000,
      "paid": 25000,
      "remaining": -5000,
      "status": "overspent"
    }
  ]
}
```

**Status values:**

| Status | Condition |
|--------|-----------|
| `unpaid` | `paid == 0` and `budgeted > 0` |
| `underspent` | `paid > 0` and `paid < budgeted` |
| `on_budget` | `paid == budgeted` |
| `overspent` | `paid > budgeted` (includes zero-budget categories with any payment) |

The `remaining` field can be negative when total payments exceed total budget. Overspend is not special-cased away — it is reported as-is.

Errors: `404 MONTH_NOT_FOUND` if month not found.

-----

## 6. Error Handling

### 6.1 Error Response Structure

Every error response from the API follows a consistent JSON structure:

```json
{
  "error": {
    "code": "ENTRY_HAS_TRANSACTIONS",
    "details": {
      "transaction_count": 3
    }
  }
}
```

The `code` field is a machine-readable uppercase string. The frontend maps these codes to user-facing translated strings via vue-i18n. The `details` field is optional and contains context-specific data that can be interpolated into the translated message (e.g. `"Cannot remove — {transaction_count} transactions exist"`).

All error responses include the `X-Request-Id` header for correlation with server logs.

### 6.2 HTTP Status Code Mapping

The API uses a small, well-defined set of HTTP status codes. The `code` field within the error body carries the precise meaning.

| Status | Meaning | Example Scenarios |
|--------|---------|-------------------|
| `400` | Malformed request | Invalid JSON, wrong field types, missing required fields, missing required query parameters |
| `404` | Resource not found | Category, month, entry, or transaction does not exist |
| `409` | Conflict | Duplicate category name, duplicate month, deleting entry with transactions |
| `422` | Validation failed | Negative amount, invalid date format, invalid month format, due day out of range |
| `500` | Internal server error | Unexpected database error, uncaught panic |

### 6.3 Error Code Catalog

Each domain module defines its own error codes. Below is the full catalog for Phase 1.

**Category errors:**

| Code | HTTP Status | Details | Description |
|------|-------------|---------|-------------|
| `CATEGORY_NOT_FOUND` | 404 | — | Category with given ID does not exist |
| `CATEGORY_NAME_ALREADY_EXISTS` | 409 | `{ "name": "..." }` | A category with this name already exists |
| `CATEGORY_INVALID_NAME` | 422 | `{ "reason": "..." }` | Category name format is invalid (empty, bad characters, double slashes, leading/trailing slash) |

**Month errors:**

| Code | HTTP Status | Details | Description |
|------|-------------|---------|-------------|
| `MONTH_NOT_FOUND` | 404 | — | Month does not exist |
| `MONTH_ALREADY_EXISTS` | 409 | `{ "month": "..." }` | A budget month with this identifier already exists |
| `MONTH_INVALID_FORMAT` | 422 | `{ "value": "..." }` | Month string does not match `YYYY-MM` format or is not a valid calendar month |

**Budget entry errors:**

| Code | HTTP Status | Details | Description |
|------|-------------|---------|-------------|
| `ENTRY_NOT_FOUND` | 404 | — | Budget entry does not exist |
| `ENTRY_CATEGORY_ALREADY_IN_MONTH` | 409 | `{ "category_id": "...", "month": "..." }` | This category is already linked to this month |
| `ENTRY_HAS_TRANSACTIONS` | 409 | `{ "transaction_count": N }` | Cannot remove entry because it has linked transactions |
| `ENTRY_INVALID_DUE_DAY` | 422 | `{ "value": N }` | Due day is outside the 1–31 range |

**Transaction errors:**

| Code | HTTP Status | Details | Description |
|------|-------------|---------|-------------|
| `TRANSACTION_NOT_FOUND` | 404 | — | Transaction does not exist |
| `TRANSACTION_ENTRY_NOT_FOUND` | 404 | — | The referenced budget entry does not exist |
| `TRANSACTION_INVALID_AMOUNT` | 422 | `{ "value": N }` | Amount is negative (UI-level restriction in Phase 1) |
| `TRANSACTION_INVALID_DATE` | 422 | `{ "value": "..." }` | Date format is invalid or is not a valid calendar date |

**Generic errors:**

| Code | HTTP Status | Details | Description |
|------|-------------|---------|-------------|
| `BAD_REQUEST` | 400 | `{ "reason": "..." }` | Malformed request body |
| `TRANSACTIONS_MONTH_REQUIRED` | 400 | — | The `month` query parameter is required for listing transactions |
| `INTERNAL_ERROR` | 500 | — | Unexpected server error (details are logged server-side, not exposed to client) |

### 6.4 Validation Strategy

Validation is split across two layers:

**API layer (structural validation):** Is the JSON well-formed? Are required fields present? Are field types correct? This is handled by Serde deserialization and Axum extractors. Failures result in `400 BAD_REQUEST`.

**Domain layer (business validation):** Are amounts non-negative (UI rule)? Is the month format valid? Is the category name correctly formatted? Does the referenced resource exist? This is handled by newtype constructors and service methods. Failures result in `404`, `409`, or `422` depending on the nature of the violation.

**Boundary rule:** The API layer accepts strings (e.g. `YYYY-MM`, `YYYY-MM-DD`) and parses them into strict domain types (`BudgetMonth { year, month }`, `TransactionDate`). The API layer serializes domain types back into strings for responses. The domain never deals with string representations of dates or months.

-----

## 7. Configuration

### 7.1 Approach

Application configuration uses the `config` crate with a two-layer strategy: a TOML configuration file provides defaults, and environment variables override any setting. This is the standard pattern for containerized applications.

### 7.2 Configuration Schema

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite://data/budget.db"

[currency]
code = "PLN"                  # ISO 4217 currency code
minor_unit_name = "grosz"     # Name of the minor unit (for display)
decimal_places = 2            # Number of decimal places for display formatting

[cors]
allowed_origins = ["http://localhost:5173"]
```

Note: In production with the nginx reverse proxy (same-origin setup), CORS can be locked down to the same origin or disabled entirely since all requests originate from the same domain.

### 7.3 Environment Variable Override

Environment variables follow the pattern `APP__SECTION__KEY` (double underscore as separator). For example:

| TOML Path | Environment Variable |
|-----------|---------------------|
| `server.port` | `APP__SERVER__PORT` |
| `database.url` | `APP__DATABASE__URL` |
| `currency.code` | `APP__CURRENCY__CODE` |

-----

## 8. Frontend Architecture

### 8.1 Project Structure

The frontend is a separate directory from the backend, built with Vite as the build tool.

```
frontend/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── playwright.config.ts
├── public/
├── src/
│   ├── main.ts                  # App entry point
│   ├── App.vue                  # Root component
│   ├── router/
│   │   └── index.ts             # Vue Router configuration
│   ├── api/
│   │   ├── client.ts            # Typed fetch wrapper
│   │   ├── types.ts             # Generated from OpenAPI spec
│   │   ├── categories.ts        # Category API functions
│   │   ├── months.ts            # Month API functions
│   │   ├── entries.ts           # Budget entry API functions
│   │   ├── transactions.ts      # Transaction API functions
│   │   └── summary.ts           # Summary API functions
│   ├── components/
│   │   ├── charts/
│   │   │   ├── BudgetVsActualChart.vue
│   │   │   └── PaymentProgressDonut.vue
│   │   ├── entries/
│   │   │   ├── EntryList.vue
│   │   │   ├── EntryForm.vue
│   │   │   └── CategoryAutocomplete.vue
│   │   ├── transactions/
│   │   │   ├── TransactionList.vue
│   │   │   ├── TransactionDrawer.vue
│   │   │   └── TransactionForm.vue
│   │   └── layout/
│   │       ├── BottomNav.vue
│   │       └── MonthTabs.vue
│   ├── views/
│   │   ├── MonthBudgetView.vue
│   │   ├── MonthTransactionsView.vue
│   │   ├── MonthListView.vue
│   │   └── CategoryListView.vue
│   ├── i18n/
│   │   ├── index.ts             # vue-i18n setup
│   │   ├── pl.json              # Polish translations
│   │   └── en.json              # English translations
│   ├── theme/
│   │   └── vuetify.ts           # Custom Vuetify theme configuration
│   └── utils/
│       └── currency.ts          # Currency formatting helpers
├── e2e/
│   └── *.spec.ts                # Playwright E2E test files
```

### 8.2 Routing

The frontend uses Vue Router with human-readable month strings in URLs. The frontend resolves month strings to backend ULIDs via the months list (fetched on navigation or cached in memory).

| Route | View | Description |
|-------|------|-------------|
| `/` | — | Redirects to `/months/{current-yyyy-mm}/budget` |
| `/months` | `MonthListView` | Browse and create budget months |
| `/months/{yyyy-mm}/budget` | `MonthBudgetView` | Charts + budget entry list for a month |
| `/months/{yyyy-mm}/transactions` | `MonthTransactionsView` | Transaction list + drawer for add/edit |
| `/categories` | `CategoryListView` | Manage global categories |

The month view uses tabs (`MonthTabs` component) to switch between the budget and transactions sub-views. The tab selection is reflected in the URL so that every view state has a shareable, bookmarkable URL.

**Month string to ULID resolution:** When navigating to a month route, the frontend looks up the month's ULID from a cached list of months (fetched from `GET /months`). All subsequent API calls for that month use the ULID. If the month string is not found in the cache, the frontend refreshes the list and retries before showing a "month not found" error.

### 8.3 Navigation

The application uses a **bottom navigation bar** (`v-bottom-navigation` from Vuetify) as the primary navigation element. This is the standard mobile navigation pattern and provides quick access to the main destinations.

Bottom nav items for Phase 1:

| Item | Icon | Route |
|------|------|-------|
| Budget | chart icon | `/months/{current-month}/budget` |
| Months | calendar icon | `/months` |
| Categories | tag icon | `/categories` |

### 8.4 API Client

The API client is a thin typed wrapper around the native `fetch` API. It lives in `src/api/client.ts` and provides:

**Base URL management:** Uses `/api/v1` as the base URL. In production with the nginx reverse proxy, this is same-origin. In development, Vite's proxy forwards `/api` to `localhost:3000`.

**Request helpers:** Typed functions for `GET`, `POST`, `PATCH`, and `DELETE` that handle JSON serialization, content-type headers, and response parsing.

**Error mapping:** Parses error responses into a typed `ApiError` object matching the backend's `{ error: { code, details } }` structure. The frontend can then map the `code` to a translated message via vue-i18n.

### 8.5 Internationalization (i18n)

The application supports two languages from day one: Polish (PL) and English (EN). All user-facing strings are stored in JSON translation files and accessed through vue-i18n's `$t()` function.

Translation files include:

**UI strings:** Labels, button text, navigation items, placeholders, chart titles, and all other static text in the interface.

**Error messages:** Every backend error code has a corresponding translation entry. Error details are interpolated into the translated string.

The backend is entirely language-agnostic — it returns only error codes and data, never human-readable messages.

### 8.6 Vuetify Custom Theme

The application uses Vuetify 3 with a custom theme. Theme customization includes:

**Custom color palette:** Green primary (`#2E7D32`), blue secondary (`#1565C0`), amber accent (`#FF8F00`).

**Dark/light mode:** Both modes are supported via Vuetify's built-in theme switching. The initial mode follows the user's system preference.

### 8.7 Key UI Components

#### 8.7.1 CategoryAutocomplete

A Vuetify `v-autocomplete` component that allows users to select an existing global category or create a new one inline.

Behavior: As the user types, the dropdown filters existing categories. If the typed text does not match any existing category, a "Create new" option appears in the dropdown. Clicking "Create new" triggers a `POST /categories` call to create the category, then proceeds to create the budget entry.

#### 8.7.2 TransactionDrawer

A Vuetify `v-bottom-sheet` that slides up when the user taps to add or edit a transaction. It contains the transaction form. This keeps the user in context without navigating away from the month view.

#### 8.7.3 Charts

**BudgetVsActualChart:** A Chart.js bar chart rendered via `vue-chartjs`. Displays side-by-side bars for each category showing budgeted amount vs. total paid. Categories with overspend are visually differentiated (red bar).

**PaymentProgressDonut:** A Chart.js doughnut chart rendered via `vue-chartjs`. Shows the percentage of total budgeted amount that has been paid, with the remaining amount displayed in the center.

Both charts receive their data from the `GET /months/{id}/summary` endpoint.

-----

## 9. Testing Strategy

### 9.1 Backend Domain Unit Tests

**Scope:** All newtypes, domain entities, service logic, and error handling within the `domain` crate.

**Approach:** Standard Rust `#[cfg(test)]` modules co-located with the code they test. Since the `domain` crate has no framework or database dependencies, tests are fast and deterministic with no setup required.

**What to test:**

Newtype construction: Valid and invalid inputs for every newtype (`Money`, `BudgetMonth`, `DueDay`, `CategoryName`, `TransactionDate`). Edge cases such as `BudgetMonth { year: 2026, month: 0 }`, `DueDay(32)`, `CategoryName` with double slashes, leading slashes, empty segments, and disallowed characters.

Service logic: Month summary computation with various combinations (underspent, overspent, on budget, zero-budget categories, no payments). Month creation copying logic. Entry removal validation (with and without transactions). Status derivation logic for all four status values.

Error variants: Correct error types are returned for each failure case.

### 9.2 Backend Integration Tests

**Scope:** Full HTTP request/response cycle through the Axum application, including database operations.

**Approach:** Integration tests in the `api` crate's `tests/` directory. Each test spins up the full Axum app with an in-memory SQLite database, sends real HTTP requests via `tower::ServiceExt::oneshot()`, and asserts on the responses.

**What to test:**

Happy paths: Every endpoint returns correct status codes and response bodies for valid requests.

Error paths: Every documented error code is triggered and returns the correct HTTP status and error body.

Business rules: Creating a month copies entries from the latest month. Deleting an entry with transactions returns `409`. Renaming a category updates the name across all months. Transaction move (PATCH with new `entry_id`) works within the same month.

Data integrity: Creating and then reading back resources returns consistent data. Deleting a transaction updates the summary. UNIQUE constraints are enforced.

Sort order verification: List endpoints return results in the documented default order.

### 9.3 Frontend E2E Tests (Playwright)

**Scope:** Full user flows from the browser through the real API and database.

**Approach:** Playwright tests in the `e2e/` directory of the frontend project. Tests run against a real backend instance with a fresh database for each test suite.

-----

## 10. Deployment

### 10.1 Default Architecture: Nginx Reverse Proxy

The default deployment model uses nginx as the single entry point:

**Frontend:** nginx serves the Vue SPA's static files (HTML, JS, CSS, assets) directly from disk.

**API proxy:** nginx proxies all `/api/*` requests to the Axum backend running on a local port.

**Benefits:** Same-origin setup eliminates CORS complexity. Single port exposed to the network. TLS termination at nginx if needed in the future.

Example nginx configuration snippet:

```nginx
server {
    listen 80;
    server_name budget.local;

    # Frontend static files
    root /var/www/budget-tracker/frontend;
    index index.html;

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API reverse proxy
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header X-Request-Id $request_id;
    }
}
```

### 10.2 Docker

The application consists of two containers orchestrated by `docker-compose.yml`:

**Backend container:** Multi-stage Rust build. The final image is a minimal Debian slim image containing only the compiled binary and the default config file. The SQLite database file is mounted as a volume for persistence.

**Frontend + nginx container:** Multi-stage Node build. The first stage builds the Vue application with Vite. The final image is an nginx container serving the static files and proxying `/api/*` requests to the backend container.

### 10.3 Data Persistence

The SQLite database file is the single source of persistence. For Docker deployments, it must be mounted as a volume.

Backups consist of copying the SQLite database file. No additional backup infrastructure is needed in Phase 1.

-----

## 11. Observability

### 11.1 Structured Logging

The backend uses the `tracing` crate with `tracing-subscriber` for structured logging. Every log entry includes:

**Request ID:** A unique identifier generated per request via middleware and propagated through the entire request lifecycle. The request ID is also returned in the `X-Request-Id` response header.

**Standard fields per request:** Request ID, HTTP method, route/path, response status code, and request latency.

**Domain error logging:** When a domain error occurs, the error code and relevant context are logged at the WARN level.

**Internal errors:** Unexpected errors (500s) are logged at the ERROR level with full context for debugging, while the API response to the client contains only a generic `INTERNAL_ERROR` code.

### 11.2 Health Check

A simple `GET /health` endpoint returns `200 OK` with a JSON body indicating the service is running. This is used by Docker health checks and monitoring systems.

-----

## 12. Out of Scope (Phase 1)

The following items are explicitly deferred to future phases:

User authentication and accounts. Tracking which user made a transaction. Bank API integration for automatic transaction import. Children's access and budgeting education features. Custom or advanced dashboard visualizations beyond the two required charts. Month locking or month states. Automatic month creation or scheduling. Hierarchy semantics beyond the name/path convention. Refunds or negative transaction amounts (though the domain and DB allow negative values for future-proofing). Moving transactions between months. Global category deletion. Sidebar navigation or hamburger menu. Slug-based category identifiers (may be added in Phase 2). Category rename history. Pagination on list endpoints.
