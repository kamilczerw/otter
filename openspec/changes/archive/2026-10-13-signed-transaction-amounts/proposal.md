## Why

The CSV import pipeline currently takes the absolute value of every bank transaction's amount before persisting it, because transaction creation rejects negative amounts outright. This discards the bank's own sign, so an incoming salary deposit or a refund is indistinguishable from an expense once imported. The longer-term plan is to add income tracking, inter-account transfers, and multi-bank integrations, but those depend on an `Account` entity that doesn't exist yet. This change scopes out the smallest useful slice: stop losing the sign on import, and let the system distinguish expense from income, without building accounts, transfers, or any new UI.

## What Changes

- Transaction creation and updates no longer reject negative amounts. A transaction's amount sign is now meaningful: negative means money out (expense), positive or zero means money in (income) — matching how banks report amounts.
- **BREAKING**: `TransactionError::InvalidAmount` is removed (nothing constructs it once the negative-amount check is gone); any caller matching on it will fail to compile.
- A transaction's direction is derived from its amount's sign via a new `TransactionKind` (`Expense` | `Income`) — not a persisted column, so no migration. Exposed on the transaction API response.
- The budget summary's per-entry "paid" total now sums the absolute value of transaction amounts, so a budget entry's spent/received magnitude is correct regardless of a transaction's sign (previously assumed all amounts were positive).
- The CSV import pipeline no longer takes the absolute value of the bank's parsed amount — it persists the signed value as-is.

## Capabilities

### New Capabilities
- `transaction-ledger`: transactions carry a meaningful signed amount and an inferred expense/income direction; creation and updates permit any sign.
- `budget-summary`: a budget entry's paid total is computed as a magnitude, independent of transaction sign, so status derivation (unpaid/underspent/on-budget/overspent) stays correct for both expense and income entries.

### Modified Capabilities
(none — no `openspec/specs/` capability currently documents transaction creation or summary behavior to amend; the above are introduced as new capability specs)

## Impact

- `backend/crates/domain/src/services/transaction_service.rs`: `create`/`update` drop the negative-amount rejection.
- `backend/crates/domain/src/errors/mod.rs`: remove the now-unreachable `TransactionError::InvalidAmount` variant.
- `backend/crates/api/src/errors.rs`: remove the corresponding error-mapping match arm.
- `backend/crates/domain/src/entities/transaction.rs` (or a nearby module): add `TransactionKind` and `Transaction::kind()`.
- `backend/crates/api/src/responses/mod.rs`: add `kind` to `TransactionResponse`.
- `backend/crates/db/src/repos/transaction_repo.rs`: `sum_by_entry`'s SQL sums `ABS(amount)`.
- `backend/crates/api/src/import/pipeline.rs`: remove the `.abs()` call on imported amounts.
- Existing import tests (`backend/crates/api/src/import/pipeline.rs` unit tests, `backend/crates/api/tests/imports_tests.rs`) that assert absolute-valued amounts need updating to the fixture's real signed values.
- No frontend changes; no new `Account`/`Move`/multi-bank work (explicitly deferred).
