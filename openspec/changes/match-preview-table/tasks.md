## 1. Backend: extend preview endpoint with pagination

- [x] 1.1 Add optional `exclude_transaction_id: Option<String>` and `page: Option<PageRequest>` (`PageRequest { offset: u32, limit: u32 }`) to `PreviewRuleRequest` (`backend/crates/api/src/requests/mod.rs`), each with a doc comment explaining purpose (`exclude_transaction_id`: omits the head transaction, already shown elsewhere in the review UI, from the returned "other matches" page and count), and verify it deserializes correctly with and without those fields.
- [x] 1.2 Add optional `other_count: Option<u32>` and `transactions: Option<Vec<TransactionResponse>>` to `PreviewRuleResponse` (`backend/crates/api/src/responses/mod.rs`).
- [x] 1.3 Extend `count_matches` (or add a sibling helper) in `backend/crates/api/src/handlers/uncategorized.rs` to optionally exclude a transaction id and slice `[offset, offset+limit)` out of the filtered matches, capping `limit` at a fixed server-side constant; verify with unit tests covering: no page requested (unchanged response), excluded id removed from both page and total, paging past the end returns empty, requested limit above the cap is clamped.
- [x] 1.4 Wire the new fields through `preview_uncategorized_rule`, returning `other_count`/`transactions` only when `page` was present in the request; verify via an integration test hitting `/transactions-review/preview` with and without `page`, confirming the count-only shape is byte-for-byte unchanged when omitted.
- [x] 1.5 Run `nix build ./nix#backend-tests-cargo` and confirm it passes.

## 2. Frontend: types and API client

- [x] 2.1 Add `exclude_transaction_id?: string` and `page?: { offset: number; limit: number }` to `PreviewRuleRequest`, and `other_count?: number` and `transactions?: TransactionResponse[]` to `PreviewRuleResponse` in `frontend/src/api/types.ts`.
- [x] 2.2 Verify `uncategorizedApi.preview` (`frontend/src/api/uncategorized.ts`) passes the new optional request fields through unchanged (no client-side logic needed beyond the existing `client.post` call).

## 3. Frontend: shared `TransactionDetailCard` component

- [x] 3.1 Extract the head-card markup (`frontend/src/views/UncategorizedReviewView.vue` lines 26–61: title/amount/date/metadata fields, raw-description truncate/expand) into `frontend/src/components/transactions/TransactionDetailCard.vue`, taking `transaction: Transaction` as a prop.
- [x] 3.2 Replace the head card's inline markup with `<TransactionDetailCard>` and verify existing tests in `UncategorizedReviewView.spec.ts` / `UncategorizedReviewView.descriptions.spec.ts` still pass unmodified (no behavior change).
- [x] 3.3 Add `TransactionDetailCard.spec.ts` covering metadata presence/absence and the raw-description expand toggle.

## 4. Frontend: `MatchPreviewTable` component

- [x] 4.1 Create `frontend/src/components/uncategorized/MatchPreviewTable.vue`: props `transactions`, `expanded`, `otherCount`, `loading`, `hasMore`; emits `toggle`, `load-more`, `row-click`. Collapsed state renders a `▸ Show N other matches` toggle; expanded state renders a `v-table` (columns: date, title, amount, merchant) inside a fixed-height scrollable container.
- [x] 4.2 Add a scroll listener on the container that emits `load-more` when scrolled near the bottom (and not already loading/no more pages).
- [x] 4.3 Add `MatchPreviewTable.spec.ts` covering: toggle emits `toggle`, scrolling near the bottom emits `load-more` only when `hasMore` and not `loading`, clicking a row emits `row-click` with that transaction.

## 5. Frontend: wire into `UncategorizedReviewView`

- [x] 5.1 Add view state: `matchedTransactions`, `matchesExpanded`, `otherCount`, `hasMoreMatches`, `matchesLoading`, `peekTransaction`.
- [x] 5.2 Mount `<MatchPreviewTable>` below the existing match-count text, bound to that state.
- [ ] 5.3 On `toggle`: flip `matchesExpanded`; if expanding and `matchedTransactions` is empty for the current criteria, fetch page `{offset: 0, limit: 20}` with `exclude_transaction_id` set to the head transaction's id.
- [ ] 5.4 Extend the existing debounced criteria-change handler (`schedulePreview`) so that, when `matchesExpanded` is true, it also re-fetches page `{offset: 0, limit: 20}` and replaces `matchedTransactions` — the count-only call stays as-is otherwise.
- [ ] 5.5 On `load-more`: fetch the next page at `offset: matchedTransactions.length`, append results, update `hasMoreMatches` from whether the page returned a full page.
- [ ] 5.6 On `row-click`: set `peekTransaction`, opening a `v-dialog` (or `v-bottom-sheet`) containing `<TransactionDetailCard>` for that transaction, with a close action and no save/cancel.
- [ ] 5.7 Add/extend `UncategorizedReviewView.spec.ts` covering: table only fetches matches once expanded (not on every keystroke while collapsed), expanded table refreshes on criteria change, row click opens the peek with the correct transaction.
- [ ] 5.8 Run `npm run type-check` and the frontend test suite, and confirm both pass.

## 6. Manual verification

- [ ] 6.1 Run the app (`run` skill or `npm run dev` + backend), open the uncategorized review screen, edit a draft rule, expand the match table, scroll to trigger pagination, and click a row to confirm the read-only peek shows full transaction detail including raw description.
