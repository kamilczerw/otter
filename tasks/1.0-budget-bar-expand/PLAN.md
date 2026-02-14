# Interactive Budget Bar — Implementation Plan

**Feature:** 1.0-budget-bar-expand
**Status:** Planning
**Last Updated:** 2026-02-14

-----

## Current State Analysis

### What exists today

The `MonthBudgetView` loads all data upfront via `Promise.all()` — summary, entries, and transactions for the entire month. The budget bars are rendered by `BudgetProgressBars.vue`, which is a purely presentational component that iterates over `CategoryBudgetSummary[]` from the summary endpoint. Transactions are shown in a separate `TransactionList` section below the bars.

### Key gaps between current code and spec

1. **No `entry_id` in `CategoryBudgetSummary`** — The summary response only includes `category` (id/name/label), `budgeted`, `paid`, `remaining`, `status`. The expanded panel needs `entry_id` to fetch transactions and link to drawers. This must be added to the backend response and frontend type.

2. **No per-entry transaction filtering** — The `GET /transactions?month={month_id}` endpoint returns ALL transactions for a month. There is no `entry_id` filter parameter. The spec requires fetching transactions for a single budget entry.

3. **No pagination on transactions endpoint** — No `limit`/`offset` parameters exist. The spec requires initial load of 3, then batches of 10.

4. **`BudgetProgressBars.vue` is a flat list** — It renders all bars in a loop with no click handling, no expand/collapse state, no slots for expanded content. It needs to be refactored into individual bar components with accordion behavior.

5. **Drawers don't support pre-filling from external context** — `TransactionDrawer` receives `entries[]` and `transaction | null` but doesn't accept a pre-selected `entry_id`. `EntryDrawer` receives `entry | null` but is opened from `EntryList`, not from the budget bar.

-----

## Implementation Tasks

### Phase 1: Backend — API Enhancements

#### Task 1.1: Add `entry_id` to CategoryBudgetSummary response

**Files to modify:**
- `backend/crates/domain/src/services/summary_service.rs` — Add `entry_id: Ulid` to `CategoryBudgetSummary` struct, populate it from the budget entry during summary construction
- `backend/crates/api/src/responses/mod.rs` — Add `entry_id: String` to `CategoryBudgetSummaryResponse`, update the `From` impl
- `frontend/src/api/types.ts` — Add `entry_id: string` to `CategoryBudgetSummary` interface

**Why:** Every downstream feature (transaction fetching, drawer pre-fill, accordion keying) depends on knowing which budget entry a bar represents. This is the foundational change.

**Testing:** Update existing summary integration tests in `backend/crates/api/` to verify `entry_id` is present in the response.

#### Task 1.2: Add `entry_id` filter to transactions endpoint

**Files to modify:**
- `backend/crates/api/src/requests/mod.rs` — Add `entry_id: Option<String>` to `TransactionListQuery`
- `backend/crates/domain/src/ports/transaction_repo.rs` — Add `list_by_entry(&self, entry_id: &Ulid, limit: u32, offset: u32) -> Result<Vec<Transaction>, TransactionError>` method to trait
- `backend/crates/domain/src/services/transaction_service.rs` — Add `list_by_entry(entry_id, limit, offset)` method
- `backend/crates/db/src/repos/transaction_repo.rs` — Implement `list_by_entry` with SQL query using `WHERE entry_id = ? ORDER BY date DESC, created_at DESC LIMIT ? OFFSET ?`
- `backend/crates/api/src/handlers/transactions.rs` — Route to `list_by_entry` when `entry_id` query param is present

**Design decision:** Use `entry_id` as an alternative to `month` (either one must be provided, not both required). When `entry_id` is provided, the response is scoped to that entry.

**Testing:** Add integration tests for `GET /transactions?entry_id={id}&limit=3&offset=0`.

#### Task 1.3: Add `limit`/`offset` pagination to transactions endpoint

**Files to modify:**
- `backend/crates/api/src/requests/mod.rs` — Add `limit: Option<u32>`, `offset: Option<u32>` to `TransactionListQuery`
- `backend/crates/api/src/handlers/transactions.rs` — Pass limit/offset to service, default limit to a reasonable max (e.g., 100)
- `backend/crates/domain/src/ports/transaction_repo.rs` — Update `list_by_entry` signature (already included in Task 1.2)
- `backend/crates/db/src/repos/transaction_repo.rs` — SQL `LIMIT ? OFFSET ?` clauses

**Response metadata:** To support `hasMore` in the frontend, the simplest approach is to request `limit + 1` items from the DB, return only `limit` items, and set a `has_more: bool` field in the response. This avoids a separate COUNT query.

**New response wrapper:**
```rust
pub struct PaginatedTransactionsResponse {
    pub items: Vec<TransactionResponse>,
    pub has_more: bool,
}
```

**Files to modify additionally:**
- `backend/crates/api/src/responses/mod.rs` — Add `PaginatedTransactionsResponse`
- `frontend/src/api/types.ts` — Add `PaginatedTransactionsResponse` interface

**Backward compatibility:** When neither `limit` nor `offset` is provided AND `entry_id` is absent (i.e., the existing `month`-only query), return the current `Vec<TransactionResponse>` format unchanged. The paginated wrapper is only used when `entry_id` is provided. This avoids breaking the existing `TransactionList` component.

**Testing:** Test edge cases — offset beyond available items, limit=0, missing entry.

-----

### Phase 2: Frontend — Core Infrastructure

#### Task 2.1: Create `frontend/src/constants.ts`

**New file:** `frontend/src/constants.ts`

```typescript
export const INITIAL_TRANSACTION_COUNT = 3
export const TRANSACTION_BATCH_SIZE = 10
export const TRANSACTION_LIST_MAX_HEIGHT = 280
```

#### Task 2.2: Update transactions API client

**File to modify:** `frontend/src/api/transactions.ts`

Add a new method for paginated per-entry fetching:

```typescript
listByEntry: (entryId: string, limit: number, offset: number) =>
  client.get<PaginatedTransactionsResponse>('/transactions', {
    entry_id: entryId,
    limit: String(limit),
    offset: String(offset),
  })
```

Keep the existing `list(monthId)` method unchanged for backward compatibility.

**File to modify:** `frontend/src/api/types.ts` — Add:

```typescript
export interface PaginatedTransactionsResponse {
  items: Transaction[]
  has_more: boolean
}
```

#### Task 2.3: Create `useCategoryTransactions` composable

**New file:** `frontend/src/composables/useCategoryTransactions.ts`

**Responsibilities:**
- Maintains a reactive `Map<string, CacheEntry>` keyed by entry ID
- Each `CacheEntry`: `{ transactions: Transaction[], offset: number, hasMore: boolean, loading: boolean }`
- `load(entryId)` — If cache exists, return it. Otherwise fetch `INITIAL_TRANSACTION_COUNT` transactions.
- `loadMore(entryId)` — Fetch next `TRANSACTION_BATCH_SIZE` from current offset, append to cache.
- `invalidate(entryId)` — Clear cache for that entry. On next `load()`, re-fetch up to the previously loaded offset (so the user sees the same depth of data).
- `invalidateAll()` — Clear entire cache (used on month navigation).
- Exposes per-entry reactive getters: `getTransactions(entryId)`, `getHasMore(entryId)`, `getIsLoading(entryId)`.

**Testing:** Unit test this composable thoroughly (see spec Section 9.1).

-----

### Phase 3: Frontend — Component Refactoring

#### Task 3.1: Refactor `BudgetProgressBars.vue` into accordion

The current `BudgetProgressBars.vue` is a flat list of `v-progress-linear` bars. It needs to become an accordion where each bar is clickable and can expand to show a panel.

**Approach:** Rather than creating a separate `BudgetCategoryBar.vue` (as spec suggests), the cleaner approach given the current architecture is:

1. **Modify `BudgetProgressBars.vue`** — Add accordion state (`expandedEntryId` ref), click handlers on each bar, chevron icon, and conditional rendering of the expanded panel below each bar.

2. **New props needed:**
   - `entries: Entry[]` — To map category → entry for drawer integration
   - `expandedEntryId: string | null` — Controlled by parent (MonthBudgetView)
   - Emit: `update:expandedEntryId` — For v-model binding

3. **Per-bar changes:**
   - Add `cursor: pointer` and click handler to toggle expansion
   - Add chevron icon (right side, rotates on expand)
   - Change `border-radius` when expanded (square bottom corners)
   - Render `BudgetCategoryPanel` below the bar when expanded

**Alternative considered:** Extract each bar into its own `BudgetCategoryBar.vue` component. This would be cleaner for separation but adds indirection. Given that `BudgetProgressBars` already handles all bar rendering logic (colors, fill widths, overspend), keeping it as the orchestrator and adding the accordion behavior inline is simpler. The panel content is extracted into a new component.

#### Task 3.2: Create `BudgetCategoryPanel.vue`

**New file:** `frontend/src/components/budget/BudgetCategoryPanel.vue`

**Props:**
- `entryId: string` — The budget entry ID
- `entry: Entry` — The full entry object (for drawer pre-fill)
- `categorySummary: CategoryBudgetSummary` — For display context

**Emits:**
- `edit-budget` — User clicked "Edit budget" (payload: `Entry`)
- `add-transaction` — User clicked "Add" (payload: `{ entryId: string }`)
- `edit-transaction` — User clicked a transaction row (payload: `Transaction`)

**Template structure:**
```
<div class="budget-panel">
  <PanelActionRow @edit-budget="..." @add-transaction="..." />
  <PanelTransactionList :entry-id="entryId" @edit-transaction="..." />
</div>
```

Uses `useCategoryTransactions` composable to fetch and display transactions.

#### Task 3.3: Create `PanelActionRow.vue`

**New file:** `frontend/src/components/budget/PanelActionRow.vue`

Simple component with two buttons:
- "Edit budget" (mdi-pencil icon + label, muted gray)
- "Add" (mdi-plus icon + label, accent green)

Both emit events to parent. Styled per spec Section 10 (minimal, icon + text, no background, right-aligned).

#### Task 3.4: Create `PanelTransactionList.vue`

**New file:** `frontend/src/components/budget/PanelTransactionList.vue`

**Props:**
- `entryId: string`

**Behavior:**
- On mount, calls `load(entryId)` from `useCategoryTransactions`
- Renders transaction rows (date + amount, clickable)
- Shows "No transactions yet" empty state
- Shows "+N more" link when `hasMore` is true and not yet in scroll mode
- After first "Show more" tap, switches to scrollable container with `max-height` from constants
- Infinite scroll via `@scroll` event on the container, triggering `loadMore()` when near bottom

**Emits:**
- `edit-transaction` — User tapped a transaction row (payload: `Transaction`)

#### Task 3.5: Update `MonthBudgetView.vue`

**Modifications:**
- Add `expandedEntryId` ref (accordion state)
- Pass `entries` and `expandedEntryId` to `BudgetProgressBars`
- Add `EntryDrawer` integration (currently only used from `EntryList`, now also triggered from budget bar panel)
- Add event handlers for `edit-budget`, `add-transaction`, `edit-transaction` from the panel
- On drawer save/delete: re-fetch summary, invalidate transaction cache for affected entry, keep accordion open
- On month navigation (`watch` on `route.params.month`): call `invalidateAll()` on the transaction cache, reset `expandedEntryId` to null

**TransactionDrawer pre-fill:** The existing `TransactionDrawer` accepts `entries[]` and `transaction | null`. For "add from panel" mode, we need to also pass a `preselectedEntryId: string | null` prop so the form starts with that entry selected.

#### Task 3.6: Update `TransactionDrawer` / `TransactionForm` for pre-selected entry

**Files to modify:**
- `frontend/src/components/transactions/TransactionDrawer.vue` — Add `preselectedEntryId` prop, pass to `TransactionForm`
- `frontend/src/components/transactions/TransactionForm.vue` — When `preselectedEntryId` is provided and no `transaction` (add mode), initialize the entry select with that value

This is a minimal change — just defaulting the `entry_id` field value.

-----

### Phase 4: Styling & Animation

#### Task 4.1: Accordion transitions

- Height transition on panel expand/collapse (CSS `transition` on `max-height` or use Vue `<Transition>` with explicit height)
- Chevron rotation (CSS `transform: rotate(180deg)` with `transition`)
- Border radius change on bar (`.budget-bar--expanded` class that sets `border-bottom-left-radius: 0; border-bottom-right-radius: 0`)

#### Task 4.2: Panel styling

- Panel background: `#151f2e` (dark, consistent with app theme)
- Transaction row styling: date left, amount right, subtle separator
- Transaction dot color: matches parent category overspend status
- Action buttons: see spec Section 10

-----

### Phase 5: Internationalization

#### Task 5.1: Add i18n keys

**Files to modify:**
- `frontend/src/i18n/en.json` — Add `budget.panel.*` keys
- `frontend/src/i18n/pl.json` — Add `budget.panel.*` keys

Keys per spec Section 8.

-----

### Phase 6: Testing

#### Task 6.1: Backend integration tests

- `GET /transactions?entry_id={id}&limit=3&offset=0` returns correct subset
- `GET /transactions?entry_id={id}` with `has_more` flag
- Summary response includes `entry_id` for each category
- Pagination edge cases (offset beyond data, empty entry)

#### Task 6.2: Frontend unit tests — `useCategoryTransactions`

Per spec Section 9.1. Test file: `frontend/src/composables/__tests__/useCategoryTransactions.spec.ts`

#### Task 6.3: Frontend component tests

Per spec Section 9.2. Test files alongside components in `__tests__/` directories.

#### Task 6.4: E2E tests (Playwright)

Per spec Section 9.3. Test file: `frontend/e2e/budget-bar-expand.spec.ts`

-----

## Dependency Graph

```
Phase 1 (Backend)
  Task 1.1 (entry_id in summary) ──┐
  Task 1.2 (entry_id filter)  ─────┤
  Task 1.3 (pagination)  ──────────┘
         │
         ▼
Phase 2 (Frontend Infrastructure)
  Task 2.1 (constants)  ───────────┐
  Task 2.2 (API client update)  ───┤
  Task 2.3 (composable)  ──────────┘
         │
         ▼
Phase 3 (Components)
  Task 3.1 (refactor bars)  ───────┐
  Task 3.2 (panel component)  ─────┤
  Task 3.3 (action row)  ──────────┤
  Task 3.4 (transaction list)  ────┤
  Task 3.5 (view integration)  ────┤
  Task 3.6 (drawer pre-fill)  ─────┘
         │
         ▼
Phase 4 (Styling) ─── can overlap with Phase 3
         │
         ▼
Phase 5 (i18n) ─── can be done alongside Phase 3
         │
         ▼
Phase 6 (Testing) ─── backend tests with Phase 1, frontend tests after Phase 3
```

-----

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| `CategoryBudgetSummary` doesn't include `entry_id` | Blocks all panel features | Phase 1, Task 1.1 — straightforward addition |
| No per-entry transaction API | Must fetch ALL month transactions and filter client-side | Phase 1, Task 1.2 — add `entry_id` filter. Fallback: client-side filter from already-loaded `transactions[]` (worse UX for large months) |
| No pagination API | Must load all transactions at once | Phase 1, Task 1.3 — add limit/offset. Fallback: client-side slicing (acceptable for typical budgets with <50 transactions per entry) |
| `v-progress-linear` doesn't support click events cleanly | May need wrapper div | Wrap each bar in a clickable container div |
| Vuetify bottom sheet conflicts with accordion panel | Panel may interfere with sheet z-index | Panel is inline content (not a sheet), should not conflict |
| Transition performance on low-end devices | Janky expand/collapse | Use `will-change: height` and `transform` for GPU acceleration |

-----

## Files Summary

### New files
- `frontend/src/constants.ts`
- `frontend/src/composables/useCategoryTransactions.ts`
- `frontend/src/components/budget/BudgetCategoryPanel.vue`
- `frontend/src/components/budget/PanelActionRow.vue`
- `frontend/src/components/budget/PanelTransactionList.vue`
- Test files (various)

### Modified files
- `backend/crates/domain/src/services/summary_service.rs`
- `backend/crates/domain/src/ports/transaction_repo.rs`
- `backend/crates/domain/src/services/transaction_service.rs`
- `backend/crates/db/src/repos/transaction_repo.rs`
- `backend/crates/api/src/requests/mod.rs`
- `backend/crates/api/src/responses/mod.rs`
- `backend/crates/api/src/handlers/transactions.rs`
- `frontend/src/api/types.ts`
- `frontend/src/api/transactions.ts`
- `frontend/src/components/budget/BudgetProgressBars.vue`
- `frontend/src/components/transactions/TransactionDrawer.vue`
- `frontend/src/components/transactions/TransactionForm.vue`
- `frontend/src/views/MonthBudgetView.vue`
- `frontend/src/i18n/en.json`
- `frontend/src/i18n/pl.json`
