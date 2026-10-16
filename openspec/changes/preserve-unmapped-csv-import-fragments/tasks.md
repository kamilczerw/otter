## 1. Backend: csv_import extraction

- [x] 1.1 Add `raw_fragments: Option<String>` to `ParsedTransaction` in `crates/csv_import/src/extract.rs`
- [x] 1.2 Add a private helper that joins `location_fragment`, `execution_date_fragment`, `original_amount_fragment`, `card_number_fragment`, `trailing_fragment_1`, and `trailing_fragment_2` with `" | "`, skipping empty columns, returning `None` if all are empty
- [x] 1.3 Call the helper from `extract_transaction` and set `raw_fragments` on every returned `ParsedTransaction`
- [x] 1.4 Update the existing fixture-based unit tests (`extracts_transfer_row`, `extracts_card_payment_row`) to assert the new field, and verify `cargo test -p csv_import` passes
- [x] 1.5 Add a unit test covering a row shape with unrecognized trailing columns (phone number + reference number, e.g. `Płatność web - kod mobilny`) asserting `merchant`/`city`/`country`/`card_last4` are `None` but `raw_fragments` contains the joined raw text, and verify it passes
- [x] 1.6 Add a unit test for the all-empty-trailing-columns case asserting `raw_fragments` is `None`, and verify it passes

## 2. Backend: domain and pipeline wiring

- [x] 2.1 Add `raw_fragments: Option<String>` to `domain::entities::ImportDetails` in `crates/domain/src/entities/import_details.rs`, updating its serde round-trip tests to cover the field present and absent, and verify `cargo test -p domain` passes
- [x] 2.2 Pass `parsed.raw_fragments.clone()` through in `import_details_from_row` in `crates/api/src/import/pipeline.rs`, updating its test fixture builder accordingly
- [x] 2.3 Confirm (via existing integration test or a new assertion) that a transaction imported from a row with unrecognized trailing columns has `raw_fragments` persisted and returned on `TransactionResponse.metadata`
- [x] 2.4 Run `nix build ./nix#backend-tests-cargo` and confirm it passes

## 3. Frontend: surface the field

- [x] 3.1 Add `rawFragments` (or the equivalent camelCase mapping already used for `rawDescription`) to the transaction metadata type in `frontend/src/api/types.ts`
- [x] 3.2 Render a new "Raw fragments" row in `TransactionDetailCard.vue`, directly under "Raw description", shown only when the field is present
- [x] 3.3 Add the corresponding label key to `frontend/src/i18n/en.json` and `frontend/src/i18n/pl.json`
- [x] 3.4 Add/update a case in `TransactionDetailCard.spec.ts` covering the new row's presence and absence, and run `npm run type-check` and the component test suite to confirm they pass

## 4. Final verification

- [x] 4.1 Manually import a CSV containing a `Płatność web - kod mobilny`-shaped row (or an equivalent fixture row) through the running app and confirm the uncategorized-review card shows the "Raw fragments" row with the phone/reference-number text
- [x] 4.2 Re-run `nix build ./nix#backend-tests-cargo` and `npm run type-check` one final time to confirm the whole change is green
