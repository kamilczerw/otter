## Context

See `proposal.md` - Why. `ParsedTransaction` (`crates/csv_import/src/extract.rs`) already computes merchant, city, country, card_last4, raw_type, currency, and raw_description for rule-based categorization, but `api::import::pipeline` only forwards `amount`, `date`, and `title` to `TransactionService::create` — everything else is dropped after categorization. The original-amount fragment (foreign-currency amount before conversion) is captured positionally in `RawRow` but never read into `ParsedTransaction` at all.

## Goals / Non-Goals

**Goals:**
- Persist the import-time fields that are already computed in memory, without discarding them after categorization.
- Keep `TransactionResponse`'s core shape (amount, date, title, kind) unpolluted by import-specific fields — nest them under a clearly-scoped `metadata` field instead.
- Keep the storage shape flexible enough to add fields later without a schema migration.

**Non-Goals:**
- Parsing the original-amount fragment into a typed amount + currency pair, or any other multi-currency support.
- User-facing tag management (add/remove tags in the UI).
- Backfilling transactions imported before this change ships.
- Supporting a second bank/CSV format — this change only prepares the shape (via a `source` field) for that to happen later without a redesign.

## Decisions

**JSON column over per-field columns.** `import_details` and `tags` are stored as JSON (`TEXT`) columns on `transactions`, serialized/deserialized via `serde_json` in `transaction_repo.rs`, rather than one column per field. Adding a field later (e.g. for a second bank format) means adding an `Option<T>` with `#[serde(default)]` to the Rust struct, not a migration. Alternative considered: a column per field, matching the project's usual SQLx-typed-query style — rejected because this data is read-only display/provenance metadata, not something queried or filtered on, so the type-safety benefit of individual columns doesn't pay for the migration churn of an evolving import format.

**`metadata`, not `import_details`, on the API response.** The domain field is named `import_details` (it only exists for imported transactions), but `TransactionResponse` exposes it as `metadata: Option<ImportDetailsResponse>`. This keeps the general transaction response shape from reading as if "import details" were an inherent property of every transaction. `ImportDetailsResponse` carries a `source` field (e.g. `"csv_import"`) so that if a second import format or source is added later, API consumers can distinguish where a given transaction's metadata came from without a shape change.

**`ImportDetails` stays one flat struct, not one type per source.** Rather than modeling per-bank-format detail shapes now, `ImportDetails` keeps today's flat set of optional fields plus `source`. We don't yet know what a second bank's CSV would need to express, so designing a polymorphic shape now would be guessing. If a future format needs genuinely different data, the pragmatic extension is more optional fields on this struct (cheap, backward compatible via `#[serde(default)]`), not a redesign of this change.

**Original-amount fragment captured unparsed.** The raw fragment is only available at import time — if it isn't captured now, transactions imported before multi-currency support exists lose that data permanently (the source CSV won't be re-importable after the fact). Capturing it as an opaque `String` costs nothing now; parsing it into a typed amount/currency belongs with the multi-currency feature itself.

**Service signature grows rather than gaining an overload.** `TransactionService::create` takes `import_details: Option<ImportDetails>` and `tags: Vec<String>` as new parameters. The manual-creation API handler passes `None`/`vec![]`; the import pipeline builds real values. No new service method, since both call sites need the same validation/persistence path and the only difference is what's passed in.

## Risks / Trade-offs

- **[Risk] JSON columns bypass SQLx's compile-time query checking for their contents.** → Mitigation: repo-level round-trip tests (write metadata, read it back, assert equality) cover correctness; the columns are never filtered/sorted on, so this doesn't affect query safety elsewhere.
- **[Risk] A second bank's CSV format might need fields `ImportDetails` doesn't anticipate.** → Mitigation: addressed in the Decisions section above — extending the struct is backward-compatible and cheap; no action needed until that format is actually being built.
