# Match Preview Table for Uncategorized Review

## Context

The uncategorized-review screen (`frontend/src/views/UncategorizedReviewView.vue`) lets the user build a match rule (field/contains/regex criteria) for the current head transaction and shows a live count of how many uncategorized transactions it matches (`uncategorized.matchCount`), via `POST /api/v1/transactions-review/preview` (`backend/crates/api/src/handlers/uncategorized.rs`).

The count alone doesn't let the user sanity-check *which* transactions a rule change pulled in or dropped — e.g. loosening a regex suddenly matches 40 more rows, but there's no way to see what they are without saving. This change adds an expandable table of the other matching transactions, so the user can verify the query before committing to it.

## Goals

- Show the user which *other* uncategorized transactions (excluding the head, already shown above) the current draft rule matches, on demand.
- Keep it lightweight when collapsed: no extra payload or rendering cost beyond today's count-only preview unless the user opens the table.
- Let the user see full transaction detail (including import metadata) for any matched row without leaving the review screen.
- Scale to queues with hundreds of matches without rendering them all at once.

## Non-Goals

- Matching against already-categorized transactions. The table only ever shows uncategorized-queue matches, same scope as today's count.
- Editing a matched transaction from this screen. Opening a row shows a read-only detail peek, not the editable `TransactionDrawer`/`TransactionForm`. Making imported transactions non-editable app-wide is a separate, future decision and out of scope here.
- Pagination/async processing of the uncategorized queue itself. The preview/list endpoints keep scanning the full in-memory queue per call, as today — see the performance note below.

## Performance note

`count_matches` (`backend/crates/api/src/handlers/uncategorized.rs`) loads the entire `UNCATEGORIZED`-category queue into memory on every preview call and filters it in Rust. This change adds a second, paginated variant of the same scan (see below) — it does not fix or hide that cost, it reuses it. This is an accepted tradeoff for now (single-user, modest data volume); don't introduce queue pagination or async job infrastructure as part of this change.

## Design

### 1. Backend: extend the preview endpoint with optional pagination

`PreviewRuleRequest` (`backend/crates/api/src/requests/mod.rs`) gains two optional fields:

- `exclude_transaction_id: Option<String>` — the head transaction's id, so the returned list never re-shows the transaction already displayed above.
- `page: Option<PageRequest>` where `PageRequest { offset: u32, limit: u32 }` (limit capped server-side, e.g. at 50, to bound response size regardless of what the client sends).

`PreviewRuleResponse` (`backend/crates/api/src/responses/mod.rs`) gains two optional fields, populated only when `page` was present in the request:

- `other_count: Option<u32>` — total matches excluding `exclude_transaction_id`.
- `transactions: Option<Vec<TransactionResponse>>` — that page's rows, in the queue's existing order.

`count` keeps its current meaning and is always returned — existing count-only callers (today's debounced preview-on-keystroke) are unaffected; they simply don't send `page` and get the same response shape as today plus two `null` fields.

`count_matches` (backend/crates/api/src/handlers/uncategorized.rs:166) is extended with a sibling helper that additionally excludes the head id and slices `[offset, offset+limit)` out of the filtered results, so no new SQL query or index is needed — it's the same in-memory scan, just also returning a page of the matches it already found.

### 2. Frontend: `MatchPreviewTable.vue`

New presentational component, sibling to `MatchCriterionRow.vue`, mounted in `UncategorizedReviewView.vue` directly below the existing match-count text.

- Collapsed state: a single toggle row, `▸ Show N other matches` (using `other_count`, distinct from the existing top-level `count` text which is unchanged and still includes the head). Clicking it expands and triggers the first paginated fetch.
- Expanded state: `▾ Hide matches`, followed by a `v-table` in a fixed-height scrollable container (`max-height: 300px; overflow-y: auto`). Columns: date, title, amount, merchant.
- Infinite scroll: a `scroll` listener on the container checks `scrollTop + clientHeight >= scrollHeight - threshold`; when true and `hasMore` is true and not already loading, emits `load-more`.
- Row click emits `row-click` with that row's transaction; the parent opens the peek.
- Props: `transactions: TransactionResponse[]`, `expanded: boolean`, `otherCount: number | null`, `loading: boolean`, `hasMore: boolean`. Emits: `toggle`, `load-more`, `row-click`. No API calls inside the component — same ownership split as `MatchCriterionRow`.

### 3. Frontend: view-level state and data flow (`UncategorizedReviewView.vue`)

New refs: `matchedTransactions`, `matchesExpanded`, `otherCount`, `hasMoreMatches`, `matchesLoading`, `peekTransaction`.

- `schedulePreview`'s existing debounced call stays count-only (no `page`) — today's "N transactions match" text is untouched.
- When `matchesExpanded` is true, the same debounce also fires a paginated call (`offset: 0, limit: 20`, `exclude_transaction_id: head.transaction.id`), replacing `matchedTransactions` and resetting scroll — so editing criteria while the table is open refreshes it, which is the actual point of this feature.
- Toggling the table open for the first time (when criteria are complete) fires that same paginated call if `matchedTransactions` is empty for the current criteria; toggling closed just hides it, no re-fetch.
- `load-more` fires the paginated call again at `offset: matchedTransactions.length`, appending results and updating `hasMoreMatches` from whether the returned page was full.
- `row-click` sets `peekTransaction`, opening the peek dialog.

### 4. Shared presentational component: `TransactionDetailCard.vue`

Extracts the existing inline head-card markup (`UncategorizedReviewView.vue` lines 26–61: title/amount/date/metadata fields including the click-to-expand raw description) into its own component, `props: { transaction: Transaction }`. Two call sites:

- The existing head card (replacing the inline block — no behavior change).
- A new read-only peek, rendered in a `v-dialog` (or `v-bottom-sheet`, matching `TransactionDrawer`'s pattern) shown when `peekTransaction` is set. No form, no save/cancel — just the card and a close action.

### 5. Types

`frontend/src/api/types.ts`: `PreviewRuleRequest` gains optional `exclude_transaction_id?: string` and `page?: { offset: number; limit: number }`; `PreviewRuleResponse` gains optional `other_count?: number` and `transactions?: TransactionResponse[]`.

## Testing

- Backend: unit tests for the new pagination/exclusion branch of the match-scanning helper (empty queue, exclude-head removes exactly one row, offset/limit slicing, limit cap enforcement). Integration test hitting `/transactions-review/preview` with and without `page` to confirm the count-only response shape is unchanged for existing callers.
- Frontend: `MatchPreviewTable.spec.ts` for toggle/expand, scroll-triggered `load-more` emission, row-click emission. `UncategorizedReviewView.spec.ts` additions for: debounced criteria change refreshing the table when expanded vs. not fetching when collapsed, and the peek opening on row click. `TransactionDetailCard.spec.ts` for the extracted rendering (metadata presence/absence, raw-description expand toggle) — regression coverage carried over from the existing head-card tests in `UncategorizedReviewView.descriptions.spec.ts`.
