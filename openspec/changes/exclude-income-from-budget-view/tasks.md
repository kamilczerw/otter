## 1. Backend: expense-only paid totals

- [x] 1.1 Update `TransactionRepository::sum_by_entry`'s query in `backend/crates/db/src/repos/transaction_repo.rs` to add `AND amount < 0`, and update its doc comment; verify by running the existing `cargo test -p db` suite plus a new/updated unit test asserting an income-only entry sums to zero.
- [x] 1.2 In `backend/crates/domain/src/services/summary_service.rs::get_month_summary`, skip pushing an entry into `categories` (and skip folding it into `total_budgeted`/`total_paid`) when `entry_repo.transaction_count(entry.id) > 0 && paid.is_zero()`; verify via unit tests (next task).
- [x] 1.3 Rewrite the existing test `income_entry_with_matching_paid_amount_is_on_budget` in `summary_service.rs` to assert the new behavior (income-only entry excluded from the summary, not shown as on-budget); verify it passes.
- [x] 1.4 Add unit tests: income-only entry excluded from `get_month_summary`'s entry list and totals; entry with zero transactions still appears; entry with mixed expense+income transactions appears with expense-only `paid`; verify via `nix build ./nix#backend-tests-cargo`.
- [x] 1.5 Update `openspec/specs/budget-summary/spec.md` is already drafted as a delta in this change — no action needed here beyond archiving at the end of the change lifecycle (handled by `openspec-archive-change`, not part of this task list).

## 2. Backend: `kind` filter on transaction listing

- [x] 2.1 Add `kind: Option<String>` to `TransactionListQuery` in `backend/crates/api/src/requests/mod.rs`, parsed into a direction filter (`expense`/`income`) in the handler, rejecting unrecognized values with a 400; verify with a new API test for an invalid `kind` value.
- [x] 2.2 Extend `TransactionRepository::list_by_entry`'s signature (trait in `backend/crates/domain/src/ports/transaction_repo.rs` and implementation in `backend/crates/db/src/repos/transaction_repo.rs`) to accept an optional direction filter, applied in SQL (`AND amount < 0` / `AND amount >= 0` / no clause); update the one call site in `backend/crates/api/src/handlers/transactions.rs::list_transactions`; verify by running `cargo build` across the workspace to confirm no other implementors/callers were missed.
- [x] 2.3 Add unit/integration tests: `kind=expense` returns only negative-amount transactions, `kind=income` returns only non-negative, no `kind` returns both; verify via `nix build ./nix#backend-tests-cargo`.

## 3. Frontend: budget panel excludes income

- [x] 3.1 Add `kind: 'income' | 'expense'` to the `Transaction` interface in `frontend/src/api/types.ts`; verify with `npm run type-check`.
- [x] 3.2 Update the budget panel's transaction fetch (`frontend/src/components/budget/PanelTransactionList.vue` and its data-fetching composable) to pass `kind=expense` to the transactions API call; verify by running the dev server and confirming only expense transactions list under a category panel.
- [ ] 3.3 Manually verify in the dev server that a category with only income transactions no longer renders as a budget bar on the budget page, and that the two summary charts (`BudgetVsActualChart`, `PaymentProgressDonut`) no longer include it.

## 4. Frontend: manual transaction form supports income

- [x] 4.1 Remove `min="0"` from the amount field in `frontend/src/components/transactions/TransactionForm.vue` and add an Income/Expense `v-btn-toggle` (defaulting to Expense) that determines the sign applied to the submitted amount, keeping the numeric input itself showing a positive magnitude; verify with `npm run type-check` and a manual dev-server check that toggling to Income produces a positive-amount transaction and Expense produces a negative-amount one.

## 5. Final verification

- [x] 5.1 Run `nix build ./nix#backend-tests-cargo` and confirm all backend tests pass.
- [x] 5.2 Run `npm run type-check` in `frontend/` and confirm it passes.
- [x] 5.3 Run `openspec validate exclude-income-from-budget-view --strict` and confirm it passes.
