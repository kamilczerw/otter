## Why

The uncategorized-transaction review queue (`bulk-categorize-transactions`) always presents a single head transaction and only advances past it by saving a categorization rule. A transaction the user doesn't want to decide on right now (unclear merchant, needs manual follow-up, etc.) blocks review of everything behind it — there's no way to move on without committing a rule.

Using the shipped skip feature surfaced three follow-up problems, addressed in this same change:

1. The head transaction card never shows the raw import description, even though the backend already has it (`metadata.raw_description`) and it's already usable as a match criterion field — the one piece of context most likely to disambiguate an otherwise-unclear transaction is hidden from the reviewer.
2. Saving a rule resets `skip` to 0, throwing away review position. (Original rationale: avoid landing past a transaction never actually decided on. Superseded below — see "Reset-on-save removed".)
3. Changing `skip` by any means other than clicking the Skip button (hand-editing the URL's query string, browser back/forward, pasting a link into an already-open tab) silently fails to reload the queue: the displayed transaction doesn't match the `skip` value in the URL.

## What Changes

- `GET /api/v1/transactions-review` accepts an optional `skip` integer query parameter (default 0) and returns `head = queue[skip % remaining]` (the queue's existing stable `date DESC, created_at DESC` order), wrapping automatically once `skip` reaches the end. `remaining` continues to report the total uncategorized count, unaffected by `skip`.
- `UncategorizedReviewView.vue` gains a "Skip" action that increments a `skip` value kept in the route's query string (so a page reload preserves position), displayed to the user as "skipped: N" using the raw (un-wrapped) counter.
- No new persistence: the offset lives only in the URL query string, not the database.
- **Raw description shown on the head card:** the head transaction card renders `metadata.raw_description` (already present on `ImportDetailsResponse`, no API change needed) as a single-line, CSS-truncated row; tapping it toggles full text. No new dependency.
- **Skip reactivity fixed:** `skip` changing by any means (button, direct URL edit, back/forward, same-document navigation) now reliably reloads the queue, via a single reactive `watch` on the `skip` value rather than ad hoc calls scattered across handlers.
- **Reset-on-save removed:** saving a rule no longer resets `skip` to 0. The raw skip value is preserved as-is across a save (not adjusted for how many transactions the save just removed), superseding the original reset-on-save rationale in favor of preserving review position across a long session.
- **Frontend test infra introduced:** `vitest` + `@vue/test-utils` added as dev dependencies (the frontend currently has none), scoped narrowly to covering the skip-reactivity fix and the raw-description rendering — not a general test-everything initiative.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `category-rules`: adds a requirement that the review queue can return its head transaction offset by a client-supplied skip count, wrapping back to the start once the count reaches the end of the queue — a non-destructive "skip" that cycles past the current transaction without excluding or reordering anything server-side. Also adds requirements that the head card displays the transaction's raw import description (truncated, expandable), that a skip-value change reloads the queue regardless of how that change originates, and that a successful save preserves rather than resets the skip value. This capability's base spec has not been archived into `openspec/specs/` yet (it currently lives in the pending `bulk-categorize-transactions` change); this delta is written against that pending capability and will apply once it's archived.

## Impact

- Backend: `backend/crates/api/src/handlers/uncategorized.rs` (`get_uncategorized_queue`, `build_queue_response`), its route query extraction, and its unit tests. Unchanged by the three follow-up fixes below — they're frontend-only.
- Frontend: `frontend/src/views/UncategorizedReviewView.vue` (route query handling, Skip button, raw-description row, `watch(skip, loadQueue)`) and `frontend/src/api/uncategorized.ts` (`getQueue` call signature).
- Frontend test infra: new `frontend/vitest.config.ts`, `vitest` + `@vue/test-utils` dev dependencies, and a new spec file for `UncategorizedReviewView.vue`.
- No database schema changes, no new endpoints.
