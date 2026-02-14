# Task Status — 1.0 Budget Bar Expand

**Feature:** Interactive Budget Bar Component
**Overall Status:** Implementation Complete (Testing Pending)
**Last Updated:** 2026-02-14

-----

## Phase Tracker

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Backend — API Enhancements | Done |
| Phase 2 | Frontend — Core Infrastructure | Done |
| Phase 3 | Frontend — Component Refactoring | Done |
| Phase 4 | Styling & Animation | Done |
| Phase 5 | Internationalization | Done |
| Phase 6 | Testing | Not Started |

## Task Tracker

### Phase 1: Backend

- [x] 1.1 — Add `entry_id` to `CategoryBudgetSummary` response
- [x] 1.2 — Add `entry_id` filter to transactions endpoint
- [x] 1.3 — Add `limit`/`offset` pagination to transactions endpoint

### Phase 2: Frontend Infrastructure

- [x] 2.1 — Create `frontend/src/constants.ts`
- [x] 2.2 — Update transactions API client with `listByEntry`
- [x] 2.3 — Create `useCategoryTransactions` composable

### Phase 3: Frontend Components

- [x] 3.1 — Refactor `BudgetProgressBars.vue` into accordion
- [x] 3.2 — Create `BudgetCategoryPanel.vue`
- [x] 3.3 — Create `PanelActionRow.vue`
- [x] 3.4 — Create `PanelTransactionList.vue`
- [x] 3.5 — Update `MonthBudgetView.vue` integration
- [x] 3.6 — Update `TransactionDrawer`/`TransactionForm` for pre-selected entry

### Phase 4: Styling

- [x] 4.1 — Accordion transitions (height, chevron, border-radius) — using Vuetify `v-expand-transition`
- [x] 4.2 — Panel styling (dark theme, transaction rows, action buttons)

### Phase 5: i18n

- [x] 5.1 — Add translation keys to `en.json` and `pl.json`

### Phase 6: Testing

- [ ] 6.1 — Backend integration tests (pagination, entry_id filter, summary entry_id)
- [ ] 6.2 — Frontend unit tests for `useCategoryTransactions` composable
- [ ] 6.3 — Frontend component tests
- [ ] 6.4 — E2E tests (Playwright)

-----

## Implementation Notes

### Backend Changes
- `CategoryBudgetSummary` now includes `entry_id: Ulid` in domain and `entry_id: String` in API response
- `TransactionRepository` trait has new `list_by_entry(entry_id, limit, offset)` method
- `GET /transactions` supports `entry_id`, `limit`, `offset` query params; returns `PaginatedTransactionsResponse` when `entry_id` is provided
- Legacy `month`-only queries still return `Vec<TransactionResponse>` for backward compatibility

### Frontend Changes
- `useCategoryTransactions` composable manages per-entry transaction cache with lazy loading
- `BudgetProgressBars.vue` refactored with accordion behavior (v-model:expanded-entry-id)
- New components: `BudgetCategoryPanel`, `PanelActionRow`, `PanelTransactionList`
- `TransactionDrawer`/`TransactionForm` support `preselectedEntryId` prop
- `MonthBudgetView` integrates accordion state, drawer pre-fill, and cache invalidation
- `EntryDrawer` now accessible from budget bar panel "Edit budget" button

### Build Status
- Backend: `cargo build` succeeds, all 77 domain tests pass
- Frontend: `vue-tsc --noEmit` type-check passes
