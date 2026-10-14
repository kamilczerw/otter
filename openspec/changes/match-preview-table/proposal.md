## Why

The uncategorized-review screen shows a live count of how many uncategorized transactions a draft match rule matches, but no way to see *which* ones — so a user who edits a rule and sees the count jump (e.g. loosening a regex suddenly matches 40 more rows) can't verify the query makes sense before saving it.

## What Changes

- `POST /api/v1/transactions-review/preview` gains optional pagination: a request can supply `exclude_transaction_id` and a `page { offset, limit }`, and the response then also includes `other_count` (matches excluding that transaction) and that page's `transactions`. Omitting `page` keeps today's count-only response unchanged.
- `UncategorizedReviewView.vue` gains an expandable table of the other uncategorized transactions the current draft rule matches (excluding the head transaction, already shown above), with infinite scroll for large match counts.
- Clicking a row in that table opens a read-only detail peek for that transaction (not the editable transaction form).
- The head card's existing transaction-detail markup is extracted into a shared, reusable `TransactionDetailCard` component so the new peek doesn't duplicate it.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `category-rules`: adds a requirement that a draft rule's preview can return a paginated page of the actual matching transactions (excluding a caller-specified transaction) alongside the existing match count, so the review UI can show the user which transactions a rule change pulled in or dropped before saving it. This capability's base spec has not been archived into `openspec/specs/` yet (it currently lives in the pending `bulk-categorize-transactions` change, with further pending deltas in `skip-uncategorized-head`); this delta is written against that pending capability and will apply once it's archived.

## Impact

- Backend: `backend/crates/api/src/handlers/uncategorized.rs` (`preview_uncategorized_rule`, `count_matches`), `backend/crates/api/src/requests/mod.rs` (`PreviewRuleRequest`), `backend/crates/api/src/responses/mod.rs` (`PreviewRuleResponse`).
- Frontend: `frontend/src/views/UncategorizedReviewView.vue`, `frontend/src/api/uncategorized.ts`, `frontend/src/api/types.ts`; new components `frontend/src/components/uncategorized/MatchPreviewTable.vue` and `frontend/src/components/transactions/TransactionDetailCard.vue`.
- No database schema changes, no new endpoints (existing `/preview` endpoint extended, backward compatible).
