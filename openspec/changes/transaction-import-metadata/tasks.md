## 1. Domain model

- [x] 1.1 Add `ImportDetails` struct to `crates/domain` (`source: String`, and `Option<String>` for `merchant`, `city`, `country`, `card_last4`, `raw_type`, `currency`, `raw_description`, `original_amount_fragment`), with `serde::{Serialize, Deserialize}` derives, and verify with a unit test asserting serde round-trip (serialize then deserialize yields an equal struct, including with all optional fields absent).
- [x] 1.2 Add `import_details: Option<ImportDetails>` and `tags: Vec<String>` to `Transaction` and `NewTransaction` in `crates/domain/src/entities/transaction.rs`, and verify existing domain tests still compile and pass after updating their fixtures.

## 2. Database layer

- [x] 2.1 Add a migration adding `import_details TEXT` (nullable) and `tags TEXT NOT NULL DEFAULT '[]'` to the `transactions` table, and verify it applies cleanly via `sqlx migrate run` against a fresh database.
- [x] 2.2 Update `transaction_repo.rs` to serialize `import_details`/`tags` to JSON on write and deserialize on read, and verify with a repo test that creates a transaction with populated `import_details`/`tags`, reads it back, and asserts equality.
- [x] 2.3 Verify a transaction created with `import_details: None`, `tags: vec![]` round-trips as `NULL`/`'[]'` respectively via the same repo test suite.

## 3. CSV extraction

- [ ] 3.1 Add `original_amount_fragment: Option<String>` to `ParsedTransaction` in `crates/csv_import/src/extract.rs`, populated from `RawRow.original_amount_fragment`, and verify with a unit test covering both a row with a non-empty original-amount column and a row with an empty one.

## 4. Service and import pipeline wiring

- [ ] 4.1 Extend `TransactionService::create` to accept `import_details: Option<ImportDetails>` and `tags: Vec<String>`, and verify existing domain service tests still pass after updating call sites.
- [ ] 4.2 Update the manual-transaction-creation API handler to pass `None`/`vec![]` for the new parameters, and verify the existing manual-creation integration test still passes.
- [ ] 4.3 Update `api::import::pipeline` to build an `ImportDetails` from each row's `ParsedTransaction` (with `source: "csv_import"`) and pass `tags: vec!["imported/csv".into()]` into `transaction_service.create(...)`, and verify with an import integration test asserting a successfully imported transaction has `tags == ["imported/csv"]` and `import_details` populated from the row's fields.
- [ ] 4.4 Verify existing duplicate-detection import tests (keyed on entry+date+amount+title) still pass unchanged.

## 5. API response

- [ ] 5.1 Add `ImportDetailsResponse` to `crates/api/src/responses/mod.rs` mirroring `ImportDetails` 1:1, with `utoipa` annotations matching existing response types, and verify the crate builds with `utoipa`'s OpenAPI generation including the new type.
- [ ] 5.2 Add `metadata: Option<ImportDetailsResponse>` and `tags: Vec<String>` to `TransactionResponse`, mapped from the domain entity's `import_details`/`tags`, and verify with an integration test that an imported transaction's response has non-null `metadata` and a manually-created transaction's response has `metadata: null`, `tags: []`.

## 6. Frontend

- [ ] 6.1 Add `tags: string[]` and `metadata: ImportMetadata | null` (camelCased: `source`, `merchant`, `city`, `country`, `cardLast4`, `rawType`, `currency`, `rawDescription`, `originalAmountFragment`) to the `Transaction` type in `src/api/types.ts`, and verify with `npm run type-check`.
- [ ] 6.2 Show `metadata?.merchant` as a muted subtitle under the title in `TransactionList.vue` and `PanelTransactionList.vue` when present, and verify manually in the browser with an imported transaction that has a merchant and one without.
- [ ] 6.3 Add a read-only "Imported from CSV" section to `TransactionForm.vue` (merchant, city, country, card last 4, type, currency, original amount fragment) rendered only when `metadata` is non-null, plus `tags` rendered as chips, and verify manually in the browser that the section is absent for a manually-created transaction and populated for an imported one.
- [ ] 6.4 Add PL and EN i18n keys for the new labels (merchant, city, country, card, type, currency, original amount, tags), and verify with `npm run type-check` and a manual language-toggle check in the browser.

## 7. Full verification

- [ ] 7.1 Run `nix build ./nix#backend-tests-cargo` and verify it passes.
- [ ] 7.2 Run `npm run type-check` in `frontend/` and verify it passes.
- [ ] 7.3 Import a real sample CSV through the running app and manually verify imported transactions show merchant/city/country/tags in both the list and the edit drawer, and that a manually-created transaction shows neither.
