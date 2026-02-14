# Interactive Budget Bar Component — Feature Specification

**Version:** 1.0
**Date:** February 2026
**Based on:** AppSpec v1.1, Implementation Spec v1.1

-----

## 1. Overview

This specification extends the existing category budget bar on the `MonthBudgetView` into an interactive, expandable component. Each bar becomes an accordion panel that reveals transaction details and provides quick access to editing budget entries and adding transactions — all without navigating away from the budget view.

The goal is to make the budget view the primary working surface: users can review spending, inspect transactions, adjust budgets, and record new payments from a single screen.

-----

## 2. Scope

### In Scope

- Accordion expand/collapse behavior on budget category bars
- Transaction list with lazy loading inside the expanded panel
- Action buttons to add transactions and edit budget entries (via existing drawers)
- Tapping a transaction to edit it (via existing TransactionDrawer)
- In-memory caching of loaded transactions per category
- Testing (unit, integration, E2E)

### Out of Scope

- Overspend badge (already implemented)
- Inline editing of any kind (all editing goes through existing drawers)
- New API endpoints (reuses existing transactions endpoint; pagination support needs investigation — see Section 5)
- Changes to the BudgetEntryDrawer or TransactionDrawer beyond pre-filling fields

-----

## 3. UX Behavior

### 3.1 Accordion

- **Only one category bar may be expanded at a time.** Tapping a collapsed bar expands it and collapses any previously open bar.
- Tapping an already-expanded bar collapses it.
- The expand/collapse transition should be animated (smooth height transition, chevron rotation).
- When a bar is expanded, its bottom corners become square (connecting visually to the drawer below). When collapsed, all corners are rounded.

### 3.2 Expanded Panel Layout

When expanded, a panel appears directly below the bar containing:

1. **Action row** (top of panel, right-aligned):
- **"Edit budget"** button — icon + label. Opens the existing `BudgetEntryDrawer` pre-filled with the entry's current category, budgeted amount, and due day.
- **"Add"** button — icon + label. Opens the existing `TransactionDrawer` in "add" mode with the category pre-filled.
1. **Transaction list** (below action row):
- Displays transactions for that budget entry within the current month.
- Tapping a transaction row opens the existing `TransactionDrawer` in "edit" mode, pre-filled with that transaction's data.
1. **Empty state:** If no transactions exist for the entry, display centered text: "No transactions yet."

### 3.3 Transaction Loading & Pagination

- On first expand of a category, load the first **3 transactions** (the initial batch size).
- If more transactions exist beyond the initial batch, show a "Show more" link at the bottom of the list (e.g., `"+N more →"`).
- Tapping "Show more" loads the next batch and switches the list into a **scrollable container with a max height**. Subsequent batches are loaded automatically via infinite scroll when the user reaches the bottom of the container.
- **Batch sizes:**
  - Initial load: **3 transactions** (named constant: `INITIAL_TRANSACTION_COUNT`)
  - Subsequent loads: **10 transactions per batch** (named constant: `TRANSACTION_BATCH_SIZE`)
- Both constants should be defined in a single, easy-to-find location in the frontend codebase (e.g., a `constants.ts` file or at the top of the relevant composable).

### 3.4 In-Memory Transaction Cache

- Once transactions are loaded for a category, they are cached in memory (e.g., in a reactive store or composable state, keyed by budget entry ID).
- If a user collapses and re-expands a category, the previously loaded transactions are displayed immediately without re-fetching. No "Show more" tap is needed — the cached data is shown.
- The cache is invalidated when:
  - A transaction is added, edited, or deleted via the TransactionDrawer (re-fetch the visible set).
  - The user navigates away from the month view entirely.

### 3.5 Scrollable Transaction Container

- After the first "Show more" tap, the transaction list becomes a scrollable box with a **max height** (suggested: ~250–300px, should be tuned visually on device).
- The max height should be defined as a named constant or CSS variable for easy adjustment.
- Infinite scroll triggers the next batch load when the user scrolls near the bottom of the container.

-----

## 4. Drawer Integration

### 4.1 TransactionDrawer — Add Mode

Triggered by the **"Add"** button in the expanded panel.

- The drawer opens in "add transaction" mode.
- The **category is pre-filled** and corresponds to the budget entry of the expanded bar. The category field should be read-only or clearly indicate it's pre-selected (user can still change it if the drawer supports it, but it starts filled).
- The **date** defaults to today.
- The **amount** field is empty and focused for immediate input.

### 4.2 TransactionDrawer — Edit Mode

Triggered by **tapping a transaction row** in the expanded panel.

- The drawer opens in "edit transaction" mode.
- All fields are pre-filled with the transaction's current data (amount, date, entry/category).

### 4.3 BudgetEntryDrawer — Edit Mode

Triggered by the **"Edit budget"** button in the expanded panel.

- The existing `BudgetEntryDrawer` opens pre-filled with:
  - **Category** — the category for this entry (read-only / display-only in edit mode)
  - **Budgeted amount** — current budgeted value
  - **Due day** — current due day (if set)

### 4.4 Post-Drawer Refresh

After any drawer action (save/delete) completes successfully:

- The **month summary** should be re-fetched to update the bar's spent/budgeted display.
- The **transaction cache** for the affected entry should be invalidated and re-fetched so the expanded panel reflects the change.
- The accordion should remain open on the same category after the refresh.

-----

## 5. API Considerations

### 5.1 Transaction Listing Endpoint

The expanded panel uses the existing `GET /transactions?month={month_id}` endpoint to fetch transactions.

**Investigation required:** The current API spec (Implementation Spec v1.1, Section 5.3.4) does not define pagination parameters (`offset`, `limit`) or filtering by `entry_id` on the transactions endpoint. The following enhancements are likely needed:

- **`entry_id` filter parameter** — to fetch transactions for a single budget entry rather than the entire month.
- **`limit` and `offset` parameters** (or cursor-based pagination) — to support the initial load of 3 and subsequent batches of 10.

### 5.2 Sort Order

Transactions within the expanded panel should be displayed in the same order defined by the API: **`date` descending, then `created_at` descending** (most recent first).

-----

## 6. Component Structure

### 6.1 New / Modified Components

|Component                 |Status      |Description                                                                  |
|--------------------------|------------|-----------------------------------------------------------------------------|
|`BudgetCategoryBar.vue`   |**Modified**|Existing bar component extended with expand/collapse and accordion logic     |
|`BudgetCategoryPanel.vue` |**New**     |The expanded drawer panel: action row + transaction list                     |
|`PanelTransactionList.vue`|**New**     |Transaction list with lazy loading, infinite scroll, and scrollable container|
|`PanelActionRow.vue`      |**New**     |Action row with "Edit budget" and "Add" buttons                              |

### 6.2 Composable

|Composable               |Description                                                                                                                                                                  |
|-------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|`useCategoryTransactions`|Manages per-entry transaction fetching, pagination state, and in-memory cache. Exposes: loaded transactions, `loadMore()`, `hasMore`, `isLoading`, and `invalidate(entryId)`.|

### 6.3 Constants

All configuration constants should be co-located in a single file (e.g., `src/constants.ts` or similar):

```typescript
/** Number of transactions shown on first expand */
export const INITIAL_TRANSACTION_COUNT = 3;

/** Number of transactions loaded per subsequent batch */
export const TRANSACTION_BATCH_SIZE = 10;

/** Max height (px) of the scrollable transaction container */
export const TRANSACTION_LIST_MAX_HEIGHT = 280;
```

-----

## 7. State Management

### 7.1 Accordion State

- A single reactive ref holds the currently expanded entry ID (or `null` if all collapsed).
- Stored at the `MonthBudgetView` level and passed down to bar components.

### 7.2 Transaction Cache

- Keyed by **budget entry ID**.
- Each cache entry stores: loaded transactions array, current offset, whether more items exist (`hasMore` flag).
- Cache lives in the `useCategoryTransactions` composable and persists for the lifetime of the `MonthBudgetView`.
- Cache is cleared on month navigation.

-----

## 8. Internationalization

All new user-facing strings must be added to both `pl.json` and `en.json` translation files:

|Key                          |EN                 |PL             |
|-----------------------------|-------------------|---------------|
|`budget.panel.editBudget`    |Edit budget        |Edytuj budżet  |
|`budget.panel.addTransaction`|Add                |Dodaj          |
|`budget.panel.noTransactions`|No transactions yet|Brak transakcji|
|`budget.panel.showMore`      |+{count} more      |+{count} więcej|
|`budget.panel.showLess`      |Show less          |Pokaż mniej    |

Exact translations should be reviewed — the above are starting suggestions.

-----

## 9. Testing Requirements

### 9.1 Frontend Unit Tests

**`useCategoryTransactions` composable:**

- Initial load fetches `INITIAL_TRANSACTION_COUNT` transactions.
- `loadMore()` fetches `TRANSACTION_BATCH_SIZE` transactions and appends to existing.
- `hasMore` is `true` when server indicates more data, `false` otherwise.
- Cache returns previously loaded data without re-fetching on re-expand.
- `invalidate(entryId)` clears cache for that entry and triggers re-fetch on next access.
- Cache clears on month change.

**Accordion state:**

- Only one entry can be expanded at a time.
- Expanding one entry collapses the previously open one.
- Toggling the same entry collapses it (sets state to `null`).

### 9.2 Frontend Component Tests

**`BudgetCategoryBar.vue`:**

- Click toggles expanded state.
- Chevron rotates when expanded.
- Border radius changes when expanded vs. collapsed.

**`BudgetCategoryPanel.vue`:**

- Renders action row and transaction list when open.
- "Edit budget" button emits correct event / calls correct handler with entry data.
- "Add" button emits correct event / calls correct handler with category pre-fill.
- Transaction row click emits correct event with transaction data.

**`PanelTransactionList.vue`:**

- Renders up to `INITIAL_TRANSACTION_COUNT` transactions on first load.
- Shows "Show more" link when more transactions exist.
- After "Show more" tap, renders scrollable container.
- Shows empty state when no transactions exist.
- Infinite scroll triggers `loadMore()` when scrolled to bottom.

### 9.3 E2E Tests (Playwright)

**Core flow:**

1. Navigate to month budget view.
1. Tap a category bar → verify panel expands, transactions load.
1. Tap another category bar → verify first collapses, second expands.
1. Tap "Add" → verify TransactionDrawer opens with category pre-filled.
1. Save a transaction → verify panel refreshes with new transaction visible.
1. Tap a transaction row → verify TransactionDrawer opens in edit mode with correct data.
1. Tap "Edit budget" → verify BudgetEntryDrawer opens with correct pre-filled data.

**Pagination flow:**

1. Set up a category with more transactions than `INITIAL_TRANSACTION_COUNT`.
1. Expand category → verify only initial count shown + "Show more" link.
1. Tap "Show more" → verify additional transactions load.
1. Collapse and re-expand → verify cached transactions shown immediately without "Show more".

**Edge cases:**

- Category with zero transactions → verify empty state message.
- Category with exactly `INITIAL_TRANSACTION_COUNT` transactions → verify no "Show more" link.

-----

## 10. Design Reference

The visual design follows the prototype provided (React reference implementation). Key visual characteristics:

- Dark theme consistent with existing app (`#151f2e` panel background, `#0d1520` page background).
- Progress bar fill color: green (`#2a7d62`) for normal, red (`#9b2e4a`) for overspent.
- Transaction dots match the overspend color of their parent category.
- Action buttons are minimal (icon + text, no background), right-aligned.
- "Add" button uses accent green (`#3ddc84`), "Edit budget" uses muted gray (`#888`).
- Smooth transitions on expand/collapse (height animation + chevron rotation).

**Note:** The prototype is a React reference for visual design only. The implementation uses Vue 3 + Vuetify 3 as specified in the Implementation Spec.
