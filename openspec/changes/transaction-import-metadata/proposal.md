## Why

CSV import already extracts merchant, city, country, card last-4, raw transaction type, currency, and raw description for each row, but only `amount`, `date`, and `title` are persisted — the rest is used transiently for rule-based categorization and then discarded. Users have no way to see *why* a transaction looks the way it does (which merchant, which city) beyond the often-empty `title` field, and the upcoming "classify uncategorized transactions" feature will need exactly this discarded data to help users make categorization decisions. This change persists what the importer already knows and surfaces it in the UI, so no further CSV parsing work is needed before that feature starts.

## What Changes

- Add a domain `ImportDetails` struct capturing merchant, city, country, card last-4, raw type, currency, raw description, and a raw (unparsed) original-amount fragment for foreign-currency transactions, tagged with a `source` identifying which importer produced it.
- `Transaction` gains `import_details: Option<ImportDetails>` (`None` for manually-created transactions) and `tags: Vec<String>` (`[]` for manual transactions).
- CSV import stops discarding the fields it already extracts, builds an `ImportDetails` per row, and stamps every imported transaction with the tag `imported/csv`.
- `csv_import` crate additionally extracts the raw original-amount fragment (previously captured as a raw CSV column but never read into `ParsedTransaction`) — unparsed, display-only, to avoid losing this data for transactions imported before multi-currency support exists.
- `TransactionResponse` gains a `metadata` field (import-sourced details, `null` for non-imported transactions) and a `tags` field, exposing this to the frontend.
- Frontend shows merchant as a subtitle in transaction lists and a full read-only "Imported from CSV" detail block (plus tag chips) in the transaction edit drawer, only when metadata is present.
- No new endpoints; no backfill of previously-imported transactions; no user-facing tag editing; no parsing of the original-amount fragment into a typed amount/currency pair.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `csv-import`: structured field extraction additionally captures the raw original-amount fragment for foreign-currency rows.
- `transaction-ledger`: transactions carry tags and optional import-sourced metadata; both default to empty/absent for manually-created transactions.
- `transaction-import`: transactions created via CSV import are tagged `imported/csv` and persisted with import metadata built from the row's extracted fields.

## Impact

- **Backend domain** (`crates/domain`): new `ImportDetails` type; `Transaction`/`NewTransaction` gain `import_details` and `tags` fields.
- **Backend db** (`crates/db`): migration adding `import_details` (nullable JSON TEXT) and `tags` (JSON TEXT, default `'[]'`) columns to `transactions`; repo mapping updated for JSON (de)serialization.
- **Backend csv_import crate**: `extract_transaction` reads the already-captured original-amount fragment into `ParsedTransaction`.
- **Backend api** (`crates/api`): `TransactionService::create` takes `import_details`/`tags`; import pipeline builds and passes them; manual-creation handler passes `None`/`[]`; `TransactionResponse` gains `metadata`/`tags` with new response type and OpenAPI annotations.
- **Frontend**: `Transaction` type gains `metadata`/`tags`; `TransactionList.vue`/`PanelTransactionList.vue` show a merchant subtitle; `TransactionForm.vue` shows a read-only import-details block and tag chips; new PL/EN i18n keys.
- **No breaking changes** — all new fields are additive and optional/defaulted.
