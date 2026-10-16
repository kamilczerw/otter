## Context

`TransactionRepository::sum_by_entry` (`backend/crates/db/src/repos/transaction_repo.rs:309-323`) currently does `SELECT COALESCE(SUM(ABS(amount)), 0) ... WHERE entry_id = ?` — a deliberate magnitude sum across both expense and income transactions, per the current `budget-summary` spec. `summary_service::get_month_summary` (`backend/crates/domain/src/services/summary_service.rs:75-126`) calls it per entry and has no notion of entry exclusion today. `BudgetEntryRepository::transaction_count` (`backend/crates/domain/src/ports/entry_repo.rs:11`) already returns an unsigned count of all transactions for an entry, used elsewhere for the delete-guard (`ENTRY_HAS_TRANSACTIONS`).

`GET /transactions` (`backend/crates/api/src/handlers/transactions.rs:15-54`) has two modes — per-entry paginated (via `TransactionListQuery.entry_id`) and legacy month-based — backed by `TransactionRepository::list_by_entry(entry_id, limit, offset)` (`transaction_repo.rs:276-299`), a plain SQL `LIMIT`/`OFFSET` query with no filtering beyond `entry_id`.

See `proposal.md` for motivation; see `specs/budget-summary/spec.md` and `specs/transaction-ledger/spec.md` for the behavior contract.

## Goals / Non-Goals

**Goals:**
- Filter income out of paid/remaining/status math and out of the per-entry transaction listing, at the SQL layer, so pagination and aggregation stay correct.
- Avoid adding a new repository method or schema column where existing data already answers the question (YAGNI).

**Non-Goals:**
- No persisted `kind`/`is_income` column — direction stays derived from `amount`'s sign, per the existing `transaction-ledger` spec.
- No category-level "income type" concept.
- No change to the legacy month-based listing mode (`list_by_month`) — only the per-entry listing used by the budget panel gains the `kind` filter capability, though the filter is written generically enough that other callers could opt in later.

## Decisions

**1. Exclusion rule reuses existing data instead of a new repository method.**
A zero-amount transaction is classified as income (per `transaction-ledger` spec), so every expense transaction has a strictly negative amount. That means `sum_by_entry` (once filtered to `amount < 0`) is zero *if and only if* the entry has no expense transactions. Combined with the already-existing `BudgetEntryRepository::transaction_count` (used today for the delete guard), `summary_service` can determine "has transactions but all are income" as:

```
transaction_count(entry_id) > 0 && paid == 0
```

No new trait method, no new query. Alternative considered: add a dedicated `expense_count`/`has_expense_transactions` repository method — rejected as redundant, since the zero-amount-is-income invariant already makes the existing data sufficient.

**2. `sum_by_entry`'s query gains a `WHERE amount < 0` filter, keeping `ABS()`.**
`SELECT COALESCE(SUM(ABS(amount)), 0) FROM transactions WHERE entry_id = ? AND amount < 0`. `ABS()` stays so `paid` remains a positive magnitude — `remaining = budgeted - paid` and `status` derivation (`derive_status`, comparing two positive magnitudes) need no changes.

**3. `list_by_entry` gains an optional direction filter, added to the trait signature.**
`TransactionRepository::list_by_entry(entry_id, limit, offset, kind: Option<TransactionKind>)`. Filtering happens in SQL (`AND amount < 0` for expense, `AND amount >= 0` for income, no clause when `None`) rather than in Rust after fetching, because the endpoint is paginated — post-fetch filtering would under-fill or mis-count pages. This is a signature change to an existing port method; `SqliteTransactionRepository` is its only implementor, and the handler/query-param layer (`TransactionListQuery`, `backend/crates/api/src/requests/mod.rs:67-73`) gains a matching optional `kind: Option<String>` field parsed into the same enum, following the existing `Option<T>` pattern used by `entry_id`/`month`/`limit`/`offset`. Alternative considered: a separate `list_by_entry_filtered` method to avoid touching the existing signature — rejected, since there's only one implementor and one call site outside tests, and two near-duplicate methods would be more code to keep in sync than one optional parameter.

**4. Entry exclusion happens in `summary_service::get_month_summary`, after computing `paid`.**
The loop already has `entry.id` and the computed `paid` in scope; skip pushing the entry into `categories` (and skip folding its `budgeted`/`paid` into the running totals) when `transaction_count(entry.id) > 0 && paid.is_zero()`. This requires one additional repository call per entry (`transaction_count`, already existing) — acceptable given budgets are small per-month lists (tens of entries, not thousands).

**5. Frontend `TransactionForm` sign toggle, not a free-sign input.**
The amount `<v-text-field>` keeps `min="0"` removed but still displays/accepts a positive magnitude from the user; a `v-btn-toggle` (Expense/Income, defaulting to Expense) determines the sign applied when building the create/update payload. Alternative considered: just remove `min="0"` and let users type a literal negative number — rejected as more error-prone (users would have to remember and correctly type a leading `-` for the common case, expenses, rather than the exception).

## Risks / Trade-offs

- **[Risk]** Changing `list_by_entry`'s signature touches every call site and any test doubles implementing `TransactionRepository` → **Mitigation**: grep for implementors/callers before editing; there is one production implementor (`SqliteTransactionRepository`) and the API handler is the only production caller — contained blast radius.
- **[Risk]** `summary_service` now makes one extra repository call (`transaction_count`) per entry per summary request → **Mitigation**: entry counts per month are small (tens, not thousands); no pagination or batching needed at this scale.
- **[Trade-off]** Income transactions become effectively write-only once created (no view to list/edit/delete them outside the budget panel, which now excludes them) → accepted per explicit scope decision; revisit if it becomes a real usability problem.

## Migration Plan

- Single-PR change, no data migration (derived-from-sign behavior, nothing persisted differently).
- Existing test `income_entry_with_matching_paid_amount_is_on_budget` (`summary_service.rs:~348-383`) encodes the old behavior and must be rewritten, not just supplemented — flagged explicitly in tasks so it isn't missed.
- Rollback is a plain revert — no schema or stored-data changes to unwind.
