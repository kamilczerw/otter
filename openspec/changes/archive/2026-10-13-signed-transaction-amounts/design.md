## Context

See proposal.md for motivation. Relevant current state:

- `TransactionService::create`/`update` (`backend/crates/domain/src/services/transaction_service.rs`) reject any amount `< 0` via `TransactionError::InvalidAmount`. `Money` itself is already a signed `i64` — the non-negative constraint is an application-level check, not a type constraint.
- `SummaryService::get_month_summary` (`backend/crates/domain/src/services/summary_service.rs`) computes each budget entry's `paid` via `TransactionRepository::sum_by_entry` — a raw `SUM(amount)` — then compares it against `budgeted` to derive a `BudgetStatus`. This assumes `paid` is always a positive "amount spent."
- The CSV import pipeline (`backend/crates/api/src/import/pipeline.rs`) calls `.abs()` on the bank's parsed amount before persisting, specifically to satisfy the non-negative check above. The bank's `Kwota` field is already parsed with its sign intact by `csv_import::extract_transaction` — the sign is only lost at this `.abs()` call.

## Goals / Non-Goals

**Goals:**
- Preserve the bank's signed amount through CSV import instead of discarding it.
- Let a transaction be either an expense or income using the amount's own sign as the single source of truth.
- Keep `paid`/status math correct for both expense and income entries without a schema change.

**Non-Goals:**
- `Account` entity, `Move` transactions, multi-bank integrations — deferred to a later change; `Move` specifically can't be designed before `Account` exists.
- Any frontend UI for entering signed or income amounts manually.
- Persisting a `kind` column — direction is fully derived from sign, so there is nothing to keep in sync and no migration.
- Netting mixed-sign transactions within the same category (e.g. a refund reducing `paid` rather than adding to it) — only matters once income/refunds routinely land in the same category as expenses, which isn't the case today.

## Decisions

### Derive `TransactionKind` from sign; don't persist it
A transaction's direction is 100% determined by its amount's sign (negative = expense, zero/positive = income). Adding a stored `kind` column now would be pure duplication of information already present in `amount`, with a real risk of the two disagreeing after a manual edit. A migration for a real `kind` column is deferred until a future feature (e.g. `Move`) needs a direction that sign alone can't express — at that point sign-derived `Expense`/`Income` and an explicit `Move` variant can coexist cleanly.

Alternative considered: add the column now to "save a migration later." Rejected — it's a redundant source of truth for a distinction that's unambiguous from sign alone today, and the eventual `Move` migration will need to happen anyway since `Move` doesn't fit the expense/income sign convention regardless of when `kind` was introduced.

### Fix `paid` with `SUM(ABS(amount))`, not a per-category direction flag
`SummaryService` needs `paid` to be a magnitude for `derive_status`'s comparisons to keep working. The simplest fix is changing `sum_by_entry`'s SQL to sum absolute values — correct for both of today's real cases (an all-expense entry, an all-income entry), since no category currently mixes directions.

Alternative considered: give each category/entry an explicit direction (expense-oriented vs income-oriented) and negate the raw signed sum accordingly before comparing to `budgeted`. Rejected for now — it correctly nets a same-category refund against spend (real ledger behavior `SUM(ABS())` doesn't capture), but requires a new domain concept (category direction) for a scenario (mixed-sign transactions within one category) that doesn't occur in the current app. Revisit when the accounts feature needs real ledger netting.

### Remove `InvalidAmount` rather than repurpose it
Once negative amounts are allowed, nothing constructs `TransactionError::InvalidAmount` anymore. Keeping an unreachable variant around is dead code; removing it (and its API error-mapping arm) is cleaner than leaving a vestigial error type with no path to it.

## Risks / Trade-offs

- **`SUM(ABS(amount))` is a magnitude fix, not a ledger-accurate netting fix.** A refund posted into the same category as the expense it reverses would inflate `paid` rather than reduce it. Accepted because nothing in the current app routes refunds into the same category as the original expense; revisit when the accounts feature needs real netting.
- **Removing `TransactionError::InvalidAmount` is a breaking API change** for any external caller matching on that error variant. Acceptable — this is an internal domain error type, not a stable public contract, and the whole point of this change is that "negative amount" stops being an error.
- **Manual transaction entry is functionally unaffected.** The frontend form isn't being touched, so a user typing a plain positive number still gets an income-classified transaction by default. This change only gives CSV import an immediate win; a manual "mark as income" flow is deferred.
