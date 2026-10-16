## Context

See `proposal.md` - Why. The relevant code paths are already in place and were extended once before for the same reason (see the archived `transaction-import-metadata` change): `csv_import::extract_transaction` (`crates/csv_import/src/extract.rs`) turns a `RawRow`'s 13 columns into a `ParsedTransaction`; `api::import::pipeline::import_details_from_row` maps that into `domain::entities::ImportDetails`, which is stored as a JSON blob on `transactions.import_details` and surfaced to the frontend via `TransactionResponse.metadata`. `TransactionDetailCard.vue` already renders `raw_description` from that metadata on the uncategorized-review screen.

## Goals / Non-Goals

**Goals:**
- Guarantee no trailing-column content is silently dropped during CSV import, regardless of `Typ transakcji`, without writing a parser for every row shape.
- Keep the rule simple and uniform: always join the same six columns, independent of whether structured extraction (merchant/city/country/card_last4) succeeded.

**Non-Goals:**
- Parsing the new fragment types (phone number, reference number, bare-domain location) into typed fields. Deferred until there's a concrete need (e.g. search/filter on phone number) — see `proposal.md`.
- A user-configurable/pluggable parser for arbitrary row shapes. Noted as a future direction, not designed here.
- Backfilling `raw_fragments` onto previously-imported transactions. Consistent with how `transaction-import-metadata` handled its own new fields.

## Decisions

**Join columns 8–13 verbatim, skip empties, `" | "` separator.** These are exactly the columns `RawRow` already splits out (`location_fragment`, `execution_date_fragment`, `original_amount_fragment`, `card_number_fragment`, `trailing_fragment_1`, `trailing_fragment_2`) and that `extract_transaction` currently either parses selectively or ignores (`execution_date_fragment` is parsed into `RawRow` but never read downstream today). Joining all six, not just the ones current parsing fails on, keeps the rule independent of extraction outcome — see next decision. `" | "` was chosen as a plain, readable separator consistent with the fragments' own `Label: value` shape; no structured encoding (e.g. JSON) since this field is display-only, matching `raw_description`'s existing treatment.

**Always populate `raw_fragments`, even when structured extraction fully succeeds.** Alternative considered: only populate it when at least one typed field (merchant/city/country/card_last4) came back `None` for a non-empty source column, so a cleanly-parsed card payment doesn't show redundant raw text next to its structured fields. Rejected because detecting "did we lose something" requires per-column success tracking that doesn't exist today (e.g. `original_amount_fragment` is captured unconditionally, with no parse step to fail) — building that tracking is exactly the per-shape complexity this change is trying to avoid. The uniform rule costs one redundant line in the UI for well-understood rows; the user confirmed this trade-off is acceptable during brainstorming.

**New field, not a widened `raw_description`.** `raw_description` is specifically column 7 (`Opis transakcji`) and is already relied on by existing tests and the UI's "Raw description" label. Keeping it unchanged and adding `raw_fragments` for columns 8–13 preserves that existing meaning and mirrors the code's existing column-7-vs-trailing-columns split.

**No DB migration.** `import_details` is a nullable JSON TEXT column (added in migration 004); the new field is additive on the Rust struct, and `serde`'s default-on-missing behavior means existing rows without `raw_fragments` deserialize to `None` with no migration needed.

## Risks / Trade-offs

- **UI redundancy on well-understood rows** (e.g. a card payment shows merchant/city/country/card *and* their raw join) → Accepted per the user's explicit choice of the uniform-rule option during brainstorming; revisit only if it becomes a real readability complaint.
- **`raw_fragments` is not machine-parseable** (just a joined string) → Acceptable since this is a fallback/display field, not meant to be queried; if phone/reference-number search is needed later, extract those as typed fields in a follow-up change rather than trying to parse this join.
