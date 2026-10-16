## Why

Budgeting currently requires manually re-entering every bank transaction, or running the `csv_import` CLI prototype locally and reading its printed breakdown — neither puts transactions into the app's actual database. This change adds a backend endpoint that imports a bank CSV export directly into real `Transaction`/`BudgetEntry`/`Category`/`Month` rows, so a bank statement becomes a usable budget without any manual entry step.

## What Changes

- New `POST /api/v1/imports` endpoint: accepts a multipart CSV file upload, decodes it (charset from the file part's own `Content-Type`, defaulting to UTF-8), parses and extracts transactions (reusing the existing `csv_import` crate as a library), categorizes them against configured rules, and persists them.
- Import auto-creates whatever's missing to make the import a one-shot operation: the `Month` for a transaction's date, the `Category` by name (including a fixed fallback `"uncategorized"` category), and the `BudgetEntry` for that (month, category) pair at `budgeted = 0`.
- A transaction whose rule match produced no usable category (no rule matched, or the matched rule's category is empty/invalid) falls into the `"uncategorized"` category rather than blocking the import.
- Duplicate detection: a transaction with the same `(entry_id, date, amount, title)` as an existing one is skipped rather than re-inserted, so re-uploading an overlapping date range is safe.
- Partial-success reporting: structurally corrupt CSV (can't be split into rows/columns) is rejected outright (`400`, nothing persisted); a per-row extraction failure (bad date/amount in one row) does not block the rest — it's itemized in the response (row number, raw content, reason) instead of being silently dropped. Duplicates and uncategorized counts are reported as counts only, not itemized.
- Response status is `201 Created` when at least one transaction was created, `200 OK` when the request was well-formed but nothing new was persisted (e.g. a fully-duplicate re-import).
- **BREAKING**: the rules file format changes from a bare YAML list to TOML (`[[matching_rules]]` array-of-tables), so both the CLI and the new API read/write the same format and the API's rules can live directly in `config.toml`. The existing `rules.yaml` must be migrated to `rules.toml` by hand; `Rule`/`MatchCriteria` field names are unchanged.
- Backend config gains a `[transaction.import]` section with an inline `matching_rules` array. Rules can also be split into a standalone TOML file and pulled in via a generic top-level `include = ["./rules.toml"]` directive (not a dedicated rules-file option) — the main config file's own values win on any key collision.
- New `TransactionRepository::find_duplicate` port method (and its SQLite implementation) backs the duplicate check.

## Capabilities

### New Capabilities
- `transaction-import`: a backend API endpoint that imports a bank CSV export into real database rows — parsing, rule-based categorization, auto-creation of months/categories/entries, duplicate skipping, and partial-success reporting of row-level failures.

### Modified Capabilities
- `csv-import`: the "Rule-based categorization" requirement changes from a hand-editable YAML rules file to a hand-editable TOML rules file (same match semantics, different on-disk format). This is the only behavioral change to the existing CLI capability — decoding, extraction, interactive learning, and breakdown output are unaffected.

## Impact

- `backend/crates/csv_import`: `rules.rs` switches `serde_yaml` → `toml` for `load_rules`/`save_rules`; its public `Rule`/`MatchCriteria` types gain a `matching_rules` wrapper key for the array-of-tables shape. `main.rs`/`cli.rs`/`learn.rs` keep working against the same `Rule` type, just a different file format. Existing `rules.yaml` is migrated to `rules.toml`.
- `backend/crates/api`: new dependency on `csv_import` as a library; new `POST /imports` handler, request/response DTOs, and a new `src/import/` module orchestrating the existing `MonthService`, `CategoryService`, `EntryService`, `TransactionService`. New `[transaction.import]` section in `AppConfig` (`crates/api/src/config.rs`), following the existing `APP__SECTION__KEY` env-override pattern.
- `backend/crates/domain`: `TransactionRepository` trait gains `find_duplicate`.
- `backend/crates/db`: `SqliteTransactionRepository` implements `find_duplicate`.
- No frontend changes in this round (import is triggered via the API directly, e.g. `curl` or a future UI button — out of scope here).
- Out of scope, deferred: rules management CRUD via API, and the CLI's "similar entries" discovery subcommand.
