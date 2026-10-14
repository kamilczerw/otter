## 1. Backend: skip param on the queue endpoint

- [x] 1.1 Add an optional `skip` integer query parameter (default 0) to `GET /api/v1/transactions-review`, parsed in `get_uncategorized_queue`, and verify via a unit test that `skip=1` returns the second queue transaction as `head`.
- [x] 1.2 Update `build_queue_response` (or its caller) so `head` is computed as `queue[skip % remaining]` when the queue is non-empty, wrapping automatically, and verify with a unit test that `skip >= remaining` wraps back into the queue instead of returning `null`.
- [x] 1.3 Verify with a unit test that `remaining` is unaffected by `skip`, and that an empty queue returns `head: null, remaining: 0` regardless of `skip`.
- [x] 1.4 Update the `#[utoipa::path]` doc/annotation on `get_uncategorized_queue` to document the new `skip` query parameter.

## 2. Frontend: Skip action and URL-persisted offset

- [x] 2.1 Update `uncategorizedApi.getQueue` in `frontend/src/api/uncategorized.ts` to accept an optional `skip` number and pass it as the `skip` query parameter.
- [x] 2.2 In `UncategorizedReviewView.vue`, read/write `skip` from the current route's query string (e.g. `route.query.skip`), add a "Skip" button next to Save, and have it increment `skip` by 1 (updating the route query) and reload the queue.
- [x] 2.3 Display the raw (un-wrapped) skip count to the user, e.g. "skipped: N", alongside the existing remaining-count text.
- [x] 2.4 After a successful `handleSave`, reset `skip` to 0 (clear the route query param) before reloading the queue, and verify manually that saving a rule doesn't leave a stale offset pointing past an undecided transaction.
- [x] 2.5 Verify manually that reloading the page while `skip` is nonzero in the URL preserves the same head transaction (no client-side state needed).
- [x] 2.6 Add the `uncategorized.skip` and `uncategorized.skippedCount` (or similarly named) i18n keys for the Skip button and the "skipped: N" text, in both PL and EN locale files.

## 3. Verification (original skip feature)

- [x] 3.1 Run `nix build ./nix#backend-tests-cargo` and confirm all backend tests pass.
- [x] 3.2 Run `npm run type-check` in `frontend/` and confirm it passes.
- [x] 3.3 Manually exercise the review page with at least 3 uncategorized transactions: skip the head twice, confirm the queue advances each time without categorizing anything, reload the page and confirm position is preserved, then skip past the end and confirm it wraps back to the first transaction.

## 4. Frontend: raw description on the head card

- [x] 4.1 In `UncategorizedReviewView.vue`'s head transaction card, add a row for `head.transaction.metadata.raw_description` (rendered only when non-null/non-empty), styled with a `text-truncate`-style single-line CSS class.
- [x] 4.2 Add a `showFullDescription` ref (reset to `false` in `resetFormForHead`) and a click handler on the description row that toggles it, switching the row's class between truncated and full-wrap.
- [x] 4.3 Add the needed i18n keys (if any new label text is introduced) to both PL and EN locale files; verify `npm run type-check` passes.

## 5. Frontend: fix skip reactivity and stop resetting skip on save

- [x] 5.1 Add `watch(skip, loadQueue)` in `UncategorizedReviewView.vue` as the single reactive trigger for "the effective skip value changed, reload the queue."
- [x] 5.2 Remove the now-redundant explicit `loadQueue()` call at the end of `handleSkip` (the new watcher fires instead, since `setSkip` changes `skip`).
- [x] 5.3 Remove the `await setSkip(0)` call in `handleSave`, and remove its now-redundant explicit `loadQueue()` call, so a successful save preserves the current raw `skip` value and reloads via the watcher.
- [x] 5.4 Double check `resetFormForHead`/`loadQueue` guard against the watcher firing a duplicate load on initial mount (the watcher should only react to *changes*, not the initial value) — verify manually that mounting the view fetches the queue exactly once.

## 6. Frontend: test infra setup

- [x] 6.1 Add `vitest`, `@vue/test-utils`, and `jsdom` as dev dependencies in `frontend/package.json`.
- [x] 6.2 Add a minimal `frontend/vitest.config.ts` that reuses the existing `vite.config.ts` (via `mergeConfig` or `defineConfig` importing it) with `test: { environment: 'jsdom' }`.
- [x] 6.3 Add a `"test": "vitest run"` script to `frontend/package.json`; verify it runs (even with zero test files yet) without config errors.

## 7. Frontend: skip-reactivity regression test

(Depends on section 6. Independent of section 8 — can run as a separate subagent once 6 is done.)

- [x] 7.1 Write a test that mounts `UncategorizedReviewView.vue` with an injected router/route stub (or a real `createMemoryHistory` router) whose initial route has `?skip=5`, mocking `uncategorizedApi.getQueue`; assert `getQueue` is called with `skip: 5` on mount.
- [x] 7.2 Write a test that, after mount with `skip=0`, changes the route's `skip` query param (simulating direct navigation/back-forward, not the Skip button) and asserts `getQueue` is called again with the new value — this is the regression test for the bug fixed in section 5.
- [x] 7.3 Write a test that clicking the Skip button still advances the queue exactly once per click (no double-fetch from the new watcher plus the old explicit call).
- [x] 7.4 Write a test that saving a rule while `skip` is nonzero reloads the queue with the same raw `skip` value, not 0.

## 8. Frontend: raw-description rendering test

(Depends on section 6. Independent of section 7 — can run as a separate subagent once 6 is done.)

- [x] 8.1 Write a test that mounting with a head transaction whose `metadata.raw_description` is set renders the truncated row.
- [x] 8.2 Write a test that mounting with a head transaction whose `metadata` is null (or `raw_description` is null) renders no description row.
- [x] 8.3 Write a test that clicking/tapping the truncated row toggles it to show the full text.

## 9. Final verification

- [x] 9.1 Run `npm run test` (or `vitest run`) in `frontend/` and confirm all new tests pass.
- [x] 9.2 Run `npm run type-check` in `frontend/` and confirm it still passes.
- [x] 9.3 Run `nix build ./nix#backend-tests-cargo` and confirm backend tests are unaffected (this change is frontend-only from section 4 onward).
- [ ] 9.4 Manually verify: a head transaction with a long raw description truncates and expands on tap; editing `skip` directly in the URL (after the `#`) while the view is open reloads to the right transaction; saving a rule while skipped keeps the same `skip` value instead of returning to the start.
