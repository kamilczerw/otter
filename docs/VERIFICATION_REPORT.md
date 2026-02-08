# Implementation Spec Verification Report

**Date:** 2026-02-08
**Spec version:** IMPLEMENTATION_SPEC.md v1.1 (Phase 1)

This report documents all deviations found between the implementation and the specification.

---

## Summary

| Area | Status | Issues Found |
|------|--------|-------------|
| Database Schema (Section 4) | PASS | 0 |
| Domain Newtypes (Section 3.2.1) | Minor deviation | 1 |
| Domain Entities | PASS | 0 |
| Domain Ports | PASS | 0 |
| Domain Services | Issues found | 2 |
| Domain Errors (Section 6.3) | Extra variants | 1 |
| API Endpoints (Section 5.3) | PASS | 0 |
| Error Response Format (Section 6.1) | PASS | 0 |
| Configuration (Section 7) | PASS | 0 |
| Workspace Layout (Section 3.3) | Missing file | 1 |
| DB Crate / Repos | Issue found | 1 |
| Frontend Routing (Section 8.2) | Issues found | 2 |
| Frontend Theme (Section 8.6) | Issue found | 1 |
| Frontend Deployment (Section 10) | Issue found | 1 |
| Testing (Section 9) | Missing items | 3 |

---

## HIGH Severity

### H1. Docker build context mismatch (Section 10.2)

`docker-compose.yml` sets the frontend service build context to `.` (repo root) with `dockerfile: frontend/Dockerfile`. However, `frontend/Dockerfile` runs `COPY package.json package-lock.json* ./` and `COPY . .`, which would copy the entire repo root into the container rather than just the frontend directory. The `npm install` and `npm run build` commands would fail because the root-level directory does not contain the frontend `package.json`.

**Fix:** Either change the docker-compose context to `./frontend` with `dockerfile: Dockerfile`, or update the Dockerfile COPY commands to reference `frontend/` subdirectory paths.

---

## MEDIUM Severity

### M1. `MonthError::NoSourceMonthForCopy` is defined but never raised (Section 3.2.1 services)

The `MonthError` enum defines a `NoSourceMonthForCopy` variant, but `MonthService::create` silently succeeds when no previous month exists to copy entries from (it creates an empty month). The error variant is dead code.

**Files:** `backend/crates/domain/src/errors/mod.rs`, `backend/crates/domain/src/services/month_service.rs`

### M2. `MonthRepository::find_latest` is defined but never called

The `MonthRepository` trait defines a `find_latest` method, but `MonthService::create` uses `list_all()` + manual sort/filter instead. This makes `find_latest` dead interface code that all repository implementations must still satisfy.

**Files:** `backend/crates/domain/src/ports/month_repo.rs`, `backend/crates/domain/src/services/month_service.rs`

### M3. `entry_repo.rs` does not handle FOREIGN KEY constraint failures (Section 6)

In `SqliteBudgetEntryRepository::create`, a FOREIGN KEY constraint failure (from a non-existent `month_id` or `category_id`) falls through to the generic `EntryError::Repository(e.to_string())` arm. The domain enum provides dedicated `EntryError::CategoryNotFound` and `EntryError::MonthNotFound` variants, but neither is used by the repo. By contrast, `SqliteTransactionRepository` correctly maps FK errors to `TransactionError::EntryNotFound`.

**Impact:** If the service layer pre-validates existence this is mitigated, but it is inconsistent with how the transaction repo handles the same scenario.

**File:** `backend/crates/db/src/repos/entry_repo.rs`

### M4. Frontend month ULID resolution is not cached (Section 8.2)

The spec says the frontend resolves month strings to ULIDs via a "cached months list." Both `MonthBudgetView.vue` and `MonthTransactionsView.vue` call `monthsApi.list()` fresh on every navigation, including on route param changes. There is no shared cache, composable, or store.

**Files:** `frontend/src/views/MonthBudgetView.vue`, `frontend/src/views/MonthTransactionsView.vue`

### M5. No error handling when month string doesn't resolve to a ULID (Section 8.2)

The spec says "the frontend refreshes the list and retries before showing a 'month not found' error." In both month views, if `months.find()` returns `undefined`, the `monthId` remains empty and `loadData()` silently returns, leaving the user with a blank page and no error message.

**Files:** `frontend/src/views/MonthBudgetView.vue`, `frontend/src/views/MonthTransactionsView.vue`

### M6. No dark/light mode toggle (Section 8.6)

The spec says "Both modes are supported via Vuetify's built-in theme switching. The initial mode follows the user's system preference." The implementation defines both `customLight` and `customDark` themes, but the default is always `customLight` with no toggle UI and no `prefers-color-scheme` detection. The dark theme is unreachable.

**File:** `frontend/src/theme/vuetify.ts`

---

## LOW Severity

### L1. No domain newtype wrapper for Ulid (Section 3.2.1)

The spec lists `Ulid` as a domain newtype with "Valid ULID format" validation. The implementation uses `ulid::Ulid` from the external crate directly throughout entities without a domain-level newtype wrapper. Functionally equivalent since the external crate validates ULID format, but doesn't match the spec's pattern of domain-defined newtypes.

**Files:** All entity files in `backend/crates/domain/src/entities/`

### L2. Extra error variants not listed in spec (Section 6.3)

Several error variants exist in the domain error enums that are not documented in the spec's error catalog:

- `EntryError::InvalidDueDay { value: u8 }`
- `EntryError::CategoryNotFound`
- `EntryError::MonthNotFound`
- `TransactionError::InvalidDate { value: String }`
- `Repository(String)` variant on all four error enums
- `DomainError` enum (for newtype validation errors)

These are practical additions that support the implementation, but they are not in the spec. Some of them (e.g., `EntryError::InvalidDueDay`) map to error codes that ARE in the spec's HTTP error catalog (e.g., `ENTRY_INVALID_DUE_DAY` at 422), suggesting the spec's domain error list is incomplete rather than the implementation being wrong.

**File:** `backend/crates/domain/src/errors/mod.rs`

### L3. `openapi.json` is not generated/committed (Section 3.2.3, 3.3)

The spec states: "The `utoipa` crate generates an OpenAPI spec from the request/response types... The generated `openapi.json` is committed to the repository." The workspace layout (Section 3.3) also shows `backend/openapi.json` as a committed file. No `openapi.json` file exists in the repository, and no `utoipa` dependency is present in any `Cargo.toml`.

**Expected:** `backend/openapi.json`
**Actual:** File does not exist. No `utoipa` crate usage found.

### L4. No Playwright E2E test configuration (Section 9.3)

The spec describes Playwright E2E tests in `frontend/e2e/` with a `playwright.config.ts`. Neither the config file nor any E2E test files exist.

**Expected:** `frontend/playwright.config.ts`, `frontend/e2e/*.spec.ts`
**Actual:** Neither exists.

### L5. No backend domain unit tests (Section 9.1)

The spec describes unit tests within the `domain` crate using `#[cfg(test)]` modules for newtypes, service logic, and error variants. No `#[cfg(test)]` modules were found in the domain crate source files. Integration tests exist in the `api` crate (`tests/api_tests.rs`), but the domain crate has no standalone unit tests.

### L6. `CurrencyConfig` is loaded but never used (Section 7.2)

The `currency` configuration section (`code`, `minor_unit_name`, `decimal_places`) is deserialized from config into `AppConfig` but is never passed to `AppState`, any service, or any handler. It is dead configuration in the backend.

**Files:** `backend/crates/api/src/config.rs`, `backend/crates/api/src/main.rs`

### L7. Frontend `formatCurrency` lacks locale-aware formatting

`frontend/src/utils/currency.ts` uses simple `toFixed(2)` without locale-aware formatting (no thousands separators, no currency symbol). For a Polish-language app, users would expect comma as decimal separator and the PLN currency symbol.

**File:** `frontend/src/utils/currency.ts`

### L8. Frontend network errors not caught by API client

The `handleResponse` function in `client.ts` only catches JSON parse failures as `NETWORK_ERROR`. Actual network failures (e.g., `fetch` throwing due to no connection) propagate as raw `TypeError` exceptions, not `ApiError` instances. Components that only check `instanceof ApiError` in catch blocks would silently swallow these errors.

**File:** `frontend/src/api/client.ts`

### L9. `api` crate `lib.rs` missing `config` and `middleware` modules

`lib.rs` exposes `errors`, `handlers`, `requests`, and `responses`, but not `config` or `middleware`. These are declared only in `main.rs`. Not a runtime issue since `main.rs` declares them, but the library target of the `api` crate is incomplete.

**File:** `backend/crates/api/src/lib.rs`

---

## Items Verified as Correct

The following spec areas were verified and found to match:

- **Database schema** (Section 4): All tables, columns, constraints, indexes, triggers, and PRAGMA foreign_keys match exactly.
- **All 16 API endpoints** (Section 5.3): Present with correct HTTP methods, status codes, request/response shapes, and error codes.
- **Error response format** (Section 6.1): `{ "error": { "code": "...", "details": {...} } }` correctly implemented.
- **X-Request-Id header** (Section 5.1): Set and propagated via tower-http middleware.
- **Configuration** (Section 7): TOML file with `APP__SECTION__KEY` env var overrides via `config` crate.
- **Sort orders** (Section 5.2): All four list endpoints return results in the documented order.
- **Domain newtypes** (Section 3.2.1): Money, BudgetMonth, DueDay, CategoryName, TransactionDate all have correct inner types and validations.
- **Domain entities**: Category, Month, BudgetEntry, Transaction use correct newtypes.
- **Repository traits**: All four traits define the expected methods with correct signatures.
- **Month creation copies entries**: MonthService::create correctly copies entries from the latest existing month.
- **Entry deletion validates transactions**: EntryService::delete checks transaction count before allowing deletion.
- **Summary computation**: SummaryService correctly computes budgeted/paid/remaining/status per category and totals.
- **Frontend routing** (Section 8.2): All routes present with correct views.
- **Bottom navigation** (Section 8.3): v-bottom-navigation with Budget, Months, Categories items.
- **API client** (Section 8.4): Thin fetch wrapper with `/api/v1` base URL.
- **i18n** (Section 8.5): vue-i18n with PL and EN, all error codes mapped.
- **Theme colors** (Section 8.6): Light theme uses correct primary (#2E7D32), secondary (#1565C0), accent (#FF8F00).
- **Components** (Section 8.7): CategoryAutocomplete (v-autocomplete with inline create), TransactionDrawer (v-bottom-sheet), BudgetVsActualChart (bar via vue-chartjs), PaymentProgressDonut (doughnut via vue-chartjs).
- **Nginx config** (Section 10.1): SPA fallback and `/api/` proxy to backend.
- **Backend integration tests** (Section 9.2): Present in `api/tests/api_tests.rs` using in-memory SQLite and tower::ServiceExt::oneshot.
- **Workspace layout** (Section 3.3): Three-crate workspace (domain, db, api) with correct dependency direction.
- **Hexagonal architecture** (Section 3.2): domain has no framework/DB dependencies; api and db depend on domain but not on each other.
