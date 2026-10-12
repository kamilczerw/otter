## 1. Rules format migration (TOML)

- [x] 1.1 Add a `matching_rules`-wrapped TOML shape for `Rule`/`MatchCriteria` in `backend/crates/csv_import/src/rules.rs` (e.g. a `RulesFile { matching_rules: Vec<Rule> }` wrapper), switch `load_rules`/`save_rules` from `serde_yaml` to `toml`, and verify the existing rules-loading unit tests pass against TOML fixtures instead of YAML.
- [x] 1.2 Update `learn.rs`'s rule-persisting flow to write the new TOML shape and verify its existing tests pass.
- [x] 1.3 Migrate `backend/crates/csv_import/rules.yaml` to `backend/crates/csv_import/rules.toml` by hand (same rules, new format) and update the CLI's `--rules-file` default/docs/Justfile recipe to point at it; delete `rules.yaml` once verified. Verify by running `csv_import` against the existing fixture CSV and confirming the same categorization output as before.
- [x] 1.4 Run `nix build ./nix#backend-tests-cargo` and verify all `csv_import` tests pass.

## 2. Domain: duplicate-detection port method

- [x] 2.1 Add `find_duplicate(entry_id, date, amount, title) -> Result<Option<Transaction>, TransactionError>` to `TransactionRepository` in `backend/crates/domain/src/ports/`.
- [x] 2.2 Implement `find_duplicate` in `SqliteTransactionRepository` (`backend/crates/db`) as an indexed query matching `entry_id`, `date`, `amount`, and `title` (including the `title IS NULL` case); add a unit test covering both a match and a non-match.
- [x] 2.3 Run `nix build ./nix#backend-tests-cargo` and verify the new test passes alongside the existing suite.

## 3. Backend config for import rules

- [x] 3.1 Add a `[transaction.import]` section to `AppConfig` (`backend/crates/api/src/config.rs`) with `rules_file: Option<PathBuf>` and `matching_rules: Option<Vec<Rule>>` (reusing `csv_import::Rule`), following the existing `APP__SECTION__KEY` override pattern.
- [x] 3.2 Implement rules resolution: inline `matching_rules` wins when both are set; otherwise load from `rules_file`; otherwise empty. Add unit tests for all three cases (inline-only, file-only, both-set-inline-wins, neither-set-empty).
- [x] 3.3 Document the new config section in `backend/config.toml` (commented example) and verify `nix build ./nix#backend-tests-cargo` passes.

## 4. Import orchestration

- [x] 4.1 Add `csv_import` as a path dependency of `backend/crates/api`.
- [x] 4.2 Create `backend/crates/api/src/import/` module with: CSV decoding (charset from the multipart part's `Content-Type`, defaulting to UTF-8, rejecting unsupported charsets), structural-parse-failure detection, and per-row extraction with failure collection (reusing `csv_import::decode_windows1250`/`parse_rows`/`extract_transaction`).
- [x] 4.3 Implement find-or-create helpers in the import module: month-by-date (via `MonthRepository::find_by_month` / `MonthService::create` with `empty: true`), category-by-name (linear scan of `CategoryRepository::list_all` / `CategoryService::create`), and entry-by-(month,category) (linear scan of `BudgetEntryRepository::list_by_month` / `EntryService::create` with `budgeted = 0`).
- [x] 4.4 Implement categorization-to-category-name resolution: run `csv_import::categorize` against the resolved rules, falling back to `"uncategorized"` when no rule matches or the matched category fails `CategoryName` validation.
- [x] 4.5 Implement the per-row pipeline: resolve month → resolve category → resolve entry → check `find_duplicate` → create transaction via `TransactionService::create`, tracking imported/duplicate/uncategorized counts and newly created month/category names.
- [x] 4.6 Add unit tests for the orchestration pipeline covering: a matched rule, no match (falls to uncategorized), an empty/invalid-category match (falls to uncategorized), a duplicate skip, and auto-creation of a missing month/category/entry.

## 5. HTTP endpoint

- [ ] 5.1 Add `ImportResponse` to `backend/crates/api/src/responses/` matching the design's shape (`imported`, `duplicates`, `uncategorized`, `created_months`, `created_categories`, `failed: Vec<{ row, raw, reason }>`).
- [ ] 5.2 Add the `POST /api/v1/imports` handler in `backend/crates/api/src/handlers/imports.rs`: parse multipart, extract the file part and its charset, invoke the import pipeline from Task 4, map structural parse failures to `400` via `ApiError::bad_request`, and return `201` when `imported > 0` or `200` otherwise.
- [ ] 5.3 Register the route in `backend/crates/api/src/main.rs`.
- [ ] 5.4 Add an OpenAPI annotation for the new endpoint consistent with existing handlers (`utoipa`).

## 6. Integration testing

- [ ] 6.1 Add an integration test (real SQLite, following the existing `api` integration test pattern) that POSTs the fixture CSV (`backend/crates/csv_import/tests/fixtures/sample_export.csv`) to `/imports` and verifies the resulting `Transaction`/`BudgetEntry`/`Category`/`Month` rows and response counts.
- [ ] 6.2 Add a test that re-posts the same file and verifies the second response reports all rows as duplicates with `imported: 0` and a `200` status.
- [ ] 6.3 Add a test with one deliberately corrupted row (e.g. bad amount) mixed into otherwise-valid rows, verifying the response is `201`, the bad row appears in `failed` with row number/raw content/reason, and the remaining rows are imported.
- [ ] 6.4 Add a test with structurally invalid CSV content, verifying a `400` response and that nothing is persisted (no new months/categories/entries/transactions).
- [ ] 6.5 Add a test for charset handling: one upload with `charset=windows-1250`, one with no charset (expect UTF-8 decoding), one with an unsupported charset (expect `400`).
- [ ] 6.6 Run `nix build ./nix#backend-tests-cargo` and verify the full suite passes.
