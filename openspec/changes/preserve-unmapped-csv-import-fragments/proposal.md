## Why

A bank CSV row's trailing columns (8–13) hold different free-text fragments depending on `Typ transakcji` (e.g. `Lokalizacja:`/`Numer karty:` for card payments). Extraction is hard-coded to the card-payment shape, so other shapes silently lose data: a `Płatność web - kod mobilny` row was found where the phone number (`Numer telefonu:`) and reference number (`Numer referencyjny:`) fragments are dropped entirely, and the merchant-domain fragment (`Lokalizacja: Adres: kociaszamka.pl`) doesn't match the `Adres:`/`Miasto:`/`Kraj:` pattern and is also dropped. New row shapes will keep appearing; rather than writing a bespoke parser per shape now, this change adds a safety net that preserves everything, so nothing is lost even when typed extraction doesn't recognize a shape.

## What Changes

- `csv_import::extract_transaction` additionally builds `raw_fragments: Option<String>` on `ParsedTransaction`: columns 8–13 (`location_fragment`, `execution_date_fragment`, `original_amount_fragment`, `card_number_fragment`, and the two always-empty trailing columns) joined with `" | "`, skipping empty columns; `None` if every column is empty. Populated for every row, including ones that parse cleanly, so the rule is uniform rather than conditional on extraction success.
- `domain::entities::ImportDetails` gains `raw_fragments: Option<String>`, carried through unchanged from the imported transaction's `ParsedTransaction`.
- CSV import's `import_details_from_row` mapping passes the new field through.
- `TransactionResponse`'s `metadata` gains `raw_fragments`, exposed to the frontend.
- Frontend `TransactionDetailCard.vue` renders a new "Raw fragments" row (new PL/EN i18n keys) directly under "Raw description", shown whenever the field is present.
- Out of scope: parsing the new row shape's phone/reference-number fragments into typed fields, and a user-configurable parser — both are explicitly deferred to a later change, per the project's YAGNI-first style.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `csv-import`: structured field extraction additionally captures a verbatim join of every unmapped trailing column, so row shapes extraction doesn't yet understand no longer lose data silently.

## Impact

- **Backend csv_import crate** (`crates/csv_import/src/extract.rs`): `ParsedTransaction` gains `raw_fragments`; new private helper joins columns 8–13.
- **Backend domain** (`crates/domain/src/entities/import_details.rs`): `ImportDetails` gains `raw_fragments: Option<String>` (serde-optional, so existing stored JSON without the field still deserializes).
- **Backend api** (`crates/api/src/import/pipeline.rs`): `import_details_from_row` passes the new field through; response DTO for transaction metadata gains `raw_fragments`.
- **Frontend**: `api/types.ts` metadata type gains `rawFragments`; `TransactionDetailCard.vue` renders it conditionally; `en.json`/`pl.json` gain a label key.
- **No DB migration** — `import_details` is stored as a JSON blob column; the new field is additive and optional.
- **No breaking changes.**
