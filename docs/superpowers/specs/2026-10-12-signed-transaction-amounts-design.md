# Signed Transaction Amounts & Transaction Kind

## Context

The CSV import pipeline (`backend/crates/api/src/import/`) currently calls `.abs()` on every bank transaction's amount before persisting it, because `TransactionService::create` rejects negative amounts outright. This discards the bank's own sign — every imported row looks like an expense, even salary deposits or refunds.

The longer-term plan is to add income tracking, inter-account transfers ("Move"), and multi-bank integrations. Those are each substantial subsystems in their own right (in particular, "Move" and multi-bank import both depend on an `Account` entity that doesn't exist yet). This change scopes out just the smallest useful slice: stop losing the sign on import, and let the system distinguish expense from income — without building accounts, transfers, or any new UI yet.

## Goals

- Preserve the bank's signed amount through CSV import instead of discarding it via `.abs()`.
- Let a `Transaction` be either an expense or income, using the amount's own sign as the source of truth (negative = expense/money out, positive or zero = income/money in — matching bank statement convention).
- Keep the budget summary's `paid`/status math correct regardless of a transaction's sign.

## Non-Goals

- `Account` entity, `Move` transactions, multi-bank integrations — all deferred to a later accounts-focused change.
- Any frontend UI for entering signed or income amounts manually — deferred until there's a concrete need (e.g. the accounts feature).
- Persisting a `kind` column. `Expense`/`Income` is fully derived from the amount's sign; no migration, no risk of a stored `kind` disagreeing with the amount.

## Design

### 1. Domain: allow signed amounts, derive `TransactionKind`

- `TransactionService::create` and `TransactionService::update` (`backend/crates/domain/src/services/transaction_service.rs`) drop the `amount.value() < 0` rejection. Any signed `i64` amount is valid.
- Remove the now-unreachable `TransactionError::InvalidAmount { value: i64 }` variant (`backend/crates/domain/src/errors/mod.rs`) and its corresponding match arm in the API's error mapping (`backend/crates/api/src/errors.rs`). Nothing will construct this variant once the checks above are removed, so keeping it would be dead code.
- Add a `TransactionKind` enum with two variants, `Expense` and `Income`, and a method `Transaction::kind(&self) -> TransactionKind`:
  - `amount.value() < 0` → `Expense`
  - `amount.value() >= 0` → `Income` (zero counts as `Income` — it represents no outflow)
- Add `kind: TransactionKind` to `TransactionResponse` (`backend/crates/api/src/responses/mod.rs`), populated in its existing `From<Transaction>` impl, so API consumers (a future frontend) can read the direction without recomputing sign logic themselves.

### 2. Summary math: fix the sign ripple into budget status

`SummaryService::get_month_summary` (`backend/crates/domain/src/services/summary_service.rs`) computes each budget entry's `paid` via `TransactionRepository::sum_by_entry`, then compares `paid` against `budgeted` to derive a `BudgetStatus`. That comparison assumes `paid` is a positive "amount spent." Once expense amounts can be negative, a raw sum would:

- Make an all-expense entry's `paid` negative, breaking every `derive_status` comparison (budgeted is always ≥ 0).
- Make an income entry's `paid` a large positive number compared against `budgeted` (typically 0), which `derive_status` would currently label `Overspent` — a cosmetic bug, not a data problem, but still wrong.

Fix: change `SqliteTransactionRepository::sum_by_entry`'s query (`backend/crates/db/src/repos/transaction_repo.rs`) from

```sql
SELECT COALESCE(SUM(amount), 0) AS total FROM transactions WHERE entry_id = ?
```

to

```sql
SELECT COALESCE(SUM(ABS(amount)), 0) AS total FROM transactions WHERE entry_id = ?
```

This makes `paid` a magnitude regardless of a transaction's direction, which is correct for both of today's cases (an all-expense entry, an all-income entry) since categories aren't mixed-direction in practice. Netting mixed-sign transactions within one category (e.g. a partial refund that should *reduce* `paid` rather than add to it) is a real ledger-accuracy question, but it only matters once income/refunds routinely land in the same category as expenses — deferred until the accounts feature needs it.

### 3. CSV import: stop discarding the sign

- Remove the `.abs()` call in the import pipeline (`backend/crates/api/src/import/pipeline.rs`) so the bank's signed `Kwota` value (already parsed with its sign intact by `csv_import::extract_transaction`) flows through unchanged into `Transaction::amount`.
- Update the existing tests that currently assert absolute-valued amounts:
  - Unit tests in `backend/crates/api/src/import/pipeline.rs` (added when the orchestration module was built).
  - Integration tests in `backend/crates/api/tests/imports_tests.rs` (`imports_fixture_and_categorizes_via_rules`, `partial_failure_reports_bad_row_and_imports_the_rest`, and any other test asserting specific transaction amounts).
  - These should now assert the fixture's real signed values rather than their absolute value.

## Testing

- Domain: unit tests for `Transaction::kind()` covering a negative amount (`Expense`), a positive amount (`Income`), and zero (`Income`).
- Domain: unit tests confirming `TransactionService::create`/`update` accept a negative amount without error (replacing the old tests that asserted `InvalidAmount` on negative input).
- `db`: a `sum_by_entry` test with a negative (expense) transaction, confirming `paid` comes back positive.
- `api`: `TransactionResponse`'s `kind` field serializes correctly for both an expense and an income transaction.
- `api`: update the import pipeline/integration tests listed above to the fixture's real signed amounts; add or confirm a case where an imported row has a positive (credit) amount and lands with `kind: Income` and is not treated as a failure.
- Full suite: `nix build ./nix#backend-tests-cargo` must pass.

## Risks / Trade-offs

- **Cosmetic-only fix for mixed-direction categories.** `SUM(ABS(amount))` is correct for single-direction categories but would double-count rather than net a refund posted into the same category as the expense it reverses. Accepted for now since nothing in the current app routes refunds into the same category; revisit when the accounts feature needs real ledger netting.
- **No validation on `amount == 0`.** The removed check only ever rejected negative values; a zero-amount transaction was already allowed and remains allowed. Not a new behavior.
- **Manual transaction entry is unaffected by this change in UI terms** — the frontend form isn't being touched, so a user manually entering a transaction today still types a plain number, which will now be interpreted as positive (`Income`) unless they explicitly type a negative value. This is a pre-existing form, not a regression, but worth knowing: this change doesn't give manual users an "expense vs income" toggle, only CSV import benefits immediately.
