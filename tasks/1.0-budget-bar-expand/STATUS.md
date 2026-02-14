# Task Status — 1.0 Budget Bar Expand

**Feature:** Interactive Budget Bar Component
**Overall Status:** Not Started
**Last Updated:** 2026-02-14

-----

## Phase Tracker

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Backend — API Enhancements | Not Started |
| Phase 2 | Frontend — Core Infrastructure | Not Started |
| Phase 3 | Frontend — Component Refactoring | Not Started |
| Phase 4 | Styling & Animation | Not Started |
| Phase 5 | Internationalization | Not Started |
| Phase 6 | Testing | Not Started |

## Task Tracker

### Phase 1: Backend

- [ ] 1.1 — Add `entry_id` to `CategoryBudgetSummary` response
- [ ] 1.2 — Add `entry_id` filter to transactions endpoint
- [ ] 1.3 — Add `limit`/`offset` pagination to transactions endpoint

### Phase 2: Frontend Infrastructure

- [ ] 2.1 — Create `frontend/src/constants.ts`
- [ ] 2.2 — Update transactions API client with `listByEntry`
- [ ] 2.3 — Create `useCategoryTransactions` composable

### Phase 3: Frontend Components

- [ ] 3.1 — Refactor `BudgetProgressBars.vue` into accordion
- [ ] 3.2 — Create `BudgetCategoryPanel.vue`
- [ ] 3.3 — Create `PanelActionRow.vue`
- [ ] 3.4 — Create `PanelTransactionList.vue`
- [ ] 3.5 — Update `MonthBudgetView.vue` integration
- [ ] 3.6 — Update `TransactionDrawer`/`TransactionForm` for pre-selected entry

### Phase 4: Styling

- [ ] 4.1 — Accordion transitions (height, chevron, border-radius)
- [ ] 4.2 — Panel styling (dark theme, transaction rows, action buttons)

### Phase 5: i18n

- [ ] 5.1 — Add translation keys to `en.json` and `pl.json`

### Phase 6: Testing

- [ ] 6.1 — Backend integration tests (pagination, entry_id filter, summary entry_id)
- [ ] 6.2 — Frontend unit tests for `useCategoryTransactions` composable
- [ ] 6.3 — Frontend component tests
- [ ] 6.4 — E2E tests (Playwright)

-----

## Codebase State Notes

Key facts about the current codebase relevant to this feature:

- **`CategoryBudgetSummary` lacks `entry_id`**: The summary endpoint (`GET /months/{id}/summary`) returns category info but not the budget entry ID. This is the first thing to add.
- **No per-entry transaction filtering**: `GET /transactions?month={id}` returns all transactions for a month. No `entry_id` filter exists.
- **No pagination on transactions**: No `limit`/`offset` support anywhere in the transaction query path.
- **`BudgetProgressBars.vue`**: Pure presentational component, renders `v-progress-linear` bars in a loop. No click handling, no slots.
- **`TransactionDrawer`**: Takes `entries[]` and `transaction | null`. Does not support pre-selecting an entry.
- **`EntryDrawer`**: Takes `entry | null` and `monthId`. Currently only opened from `EntryList`.
- **All transactions loaded upfront**: `MonthBudgetView.loadData()` fetches all transactions for the month in `Promise.all()`. The separate per-entry lazy loading is a new pattern.
