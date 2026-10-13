## Why

Budgeting currently requires manually re-entering every bank transaction into the app. Bank statements are exportable as CSV, but the raw export is messy (Windows-1250 encoding, a free-text description column that bundles merchant/location/card number differently per transaction type) and has no category information. A prototype CLI that parses these exports and assigns categories removes the manual entry step and is a first step toward importing bank data into the budget app's UI.

## What Changes

- New standalone Rust binary crate, `csv_import`, in the existing workspace, that:
  - Decodes a bank CSV export from Windows-1250 to UTF-8.
  - Parses each row and extracts structured fields from the free-text description: card last 4 digits, merchant, city, country, transaction title, raw transaction type.
  - Categorizes each transaction against a hand-editable + self-learning YAML rules file (`--rules-file`), matching on any combination of the extracted fields.
  - Prompts interactively for a category when no rule matches (unless `--no-interactive`, which leaves the transaction as `Uncategorized`), and persists the answer as a new rule.
  - Prints a spending breakdown (per-category totals and transaction counts) in `pretty` (table) or `json` format via `--format`.
- No database or UI wiring in this change — categories exist only as strings inside the rules file. Both are explicit future work.

## Capabilities

### New Capabilities
- `csv-import`: parse a bank CSV export, extract structured transaction fields, and categorize transactions using a rules engine that can be hand-edited and that learns from interactive corrections.

### Modified Capabilities
(none)

## Impact

- New crate `backend/crates/csv_import` added to the Cargo workspace; depends on `domain` for the `Money` type and on `csv`, `encoding_rs`, `serde`/`serde_yaml`, `clap`, and `thiserror` (error enums) as runtime dependencies, plus `assert2` as a dev-dependency for tests.
- No changes to `api`, `db`, or the frontend.
- No changes to the production database schema or deployment.
