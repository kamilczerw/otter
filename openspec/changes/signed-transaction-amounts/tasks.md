## 1. Domain: allow signed amounts, derive TransactionKind

- [x] 1.1 Remove the `amount.value() < 0` rejection from `TransactionService::create` and `TransactionService::update` (`backend/crates/domain/src/services/transaction_service.rs`); update their doc comments accordingly. Verify by updating the existing tests that asserted `InvalidAmount` on negative input so they instead assert the transaction is created/updated successfully with a negative amount.
- [x] 1.2 Remove the now-unreachable `TransactionError::InvalidAmount { value: i64 }` variant (`backend/crates/domain/src/errors/mod.rs`) and its match arm in `backend/crates/api/src/errors.rs`. Verify with `cargo build -p domain -p api` (no unreachable-pattern or missing-variant errors).
- [x] 1.3 Add a `TransactionKind` enum (`Expense`, `Income`) and `Transaction::kind(&self) -> TransactionKind` (negative amount → `Expense`, zero or positive → `Income`) in `backend/crates/domain/src/entities/transaction.rs`. Add unit tests covering a negative amount, a positive amount, and zero.
- [x] 1.4 Run `nix build ./nix#backend-tests-cargo` and verify the `domain` crate's tests pass.

## 2. API: expose transaction direction

- [x] 2.1 Add a `kind` field to `TransactionResponse` (`backend/crates/api/src/responses/mod.rs`), populated from `Transaction::kind()` in its `From<Transaction>` impl. Serialized as a plain `String` (`"expense"`/`"income"`), matching the existing `CategoryBudgetSummaryResponse.status` convention — domain enums are stringified at the API boundary rather than giving domain types a `utoipa::ToSchema` dependency, which would violate the "domain has no framework dependencies" architectural rule.
- [x] 2.2 Add/update a unit or integration test asserting `TransactionResponse` serializes `kind: "expense"` for a negative-amount transaction and `kind: "income"` for a positive-amount transaction.
- [x] 2.3 Run `nix build ./nix#backend-tests-cargo` and verify the `api` crate's tests pass.

## 3. Budget summary: paid as magnitude

- [ ] 3.1 Change `SqliteTransactionRepository::sum_by_entry`'s query (`backend/crates/db/src/repos/transaction_repo.rs`) from `SUM(amount)` to `SUM(ABS(amount))`.
- [ ] 3.2 Add a test for `sum_by_entry` with a negative (expense) transaction, asserting the returned total is positive; add a test with a positive (income) transaction, asserting the total is also positive.
- [ ] 3.3 Add a `SummaryService` test (or extend an existing one) with an income-category entry (positive-amount transactions) whose `paid` equals `budgeted`, asserting the derived status is `OnBudget`, not `Overspent`.
- [ ] 3.4 Run `nix build ./nix#backend-tests-cargo` and verify the `db` and `domain` crates' tests pass.

## 4. CSV import: preserve the bank's sign

- [ ] 4.1 Remove the `.abs()` call on the parsed amount in `backend/crates/api/src/import/pipeline.rs` so the bank's signed value is passed through unchanged to `Transaction::amount`.
- [ ] 4.2 Update the import pipeline's existing unit tests (in `pipeline.rs`) that currently assert absolute-valued amounts, so they assert the real signed value instead. Add a case where an imported row has a positive (credit) amount, asserting it is persisted as positive and classified `Income`, and is not treated as a failure.
- [ ] 4.3 Update `backend/crates/api/tests/imports_tests.rs` (`imports_fixture_and_categorizes_via_rules`, `partial_failure_reports_bad_row_and_imports_the_rest`, and any other test asserting specific transaction amounts) to the fixture's real signed values.
- [ ] 4.4 Run `nix build ./nix#backend-tests-cargo` and verify the full suite passes, including all sections above.
