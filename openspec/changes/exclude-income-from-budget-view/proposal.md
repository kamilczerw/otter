## Why

The budget page currently counts income transactions (positive amounts) toward a category's "paid" total, which inflates progress on income-tagged categories and makes the budget view show money that was never spent. Income transactions should be tracked separately from spending, not mixed into budget math.

## What Changes

- Budget entry "paid" totals are computed from expense transactions only (negative amounts); income transactions (positive amounts) no longer count toward paid, remaining, or status. **BREAKING**: changes the result of `budget-summary`'s existing paid-total computation for any entry with income transactions.
- A budget entry whose transactions exist but are entirely income is dropped from the month's budget summary (and therefore from the budget bars and both summary charts, which consume the same summary data).
- The transactions list endpoint (`GET /transactions`) gains an optional `kind` filter (`expense` or `income`) so a caller can request only one direction; the budget page's per-category transaction panel uses `kind=expense` so income rows never appear there.
- The manual transaction form no longer blocks negative amounts (previously `min="0"` on the amount field made it impossible to create an expense by hand); it gains an Income/Expense toggle that sets the sign on submit while the number input keeps showing a positive magnitude.
- The frontend's `Transaction` type now includes the `kind` field the API already returns (previously unused/discarded by the frontend).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `budget-summary`: paid-total computation changes from "sum of absolute value of all transactions" to "sum of absolute value of expense (negative-amount) transactions only"; adds a new requirement that entries with only income transactions are excluded from the summary.
- `transaction-ledger`: adds a requirement that transactions can be filtered by direction (expense/income) when listed.

## Impact

- `backend/crates/db/src/repos/transaction_repo.rs` — `sum_by_entry` query gains an `amount < 0` filter.
- `backend/crates/domain/src/services/summary_service.rs` — entry filtering logic for income-only entries; existing test `income_entry_with_matching_paid_amount_is_on_budget` is replaced to assert the new behavior.
- `backend/crates/api` — transactions list handler/query params gain `kind` filtering; API test coverage.
- `frontend/src/api/types.ts` — `Transaction.kind` field added.
- `frontend/src/components/budget/PanelTransactionList.vue` (and its data-fetching composable) — requests `kind=expense`.
- `frontend/src/components/transactions/TransactionForm.vue` — removes `min="0"`, adds Income/Expense toggle.
- `openspec/specs/budget-summary/spec.md` — updated to reflect expense-only aggregation and income-only-entry exclusion.
- No changes needed to `BudgetProgressBars.vue`, `BudgetVsActualChart.vue`, or `PaymentProgressDonut.vue` — they consume the already-filtered summary data.
