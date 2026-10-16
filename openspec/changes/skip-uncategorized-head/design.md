## Context

The skip feature (backend `skip` query param + frontend Skip button) is already implemented and shipped (`bca638d`, `0df040c`). Using it in practice surfaced three problems, all frontend-only — the backend's `skip % remaining` wrapping logic is correct and untouched by this design.

`UncategorizedReviewView.vue` currently derives a `skip` computed ref from `route.query.skip`, and calls `loadQueue()` from exactly three places: `onMounted`, the end of `handleSkip`, and the end of `handleSave`. There is no `watch` on `skip` itself — reloading only happens because the two handlers remember to call it explicitly.

The frontend has no test runner today (`frontend/package.json` has `dev`/`build`/`preview`/`type-check` only, zero `*.spec.ts` files anywhere).

## Goals / Non-Goals

**Goals:**
- Any change to the effective `skip` value reloads the queue, regardless of what caused the change.
- The head card shows the raw import description already present in the API response.
- A successful save no longer discards review position.
- A minimal, reusable test setup that specifically regression-tests the bug being fixed here — not a general test-everything initiative.

**Non-Goals:**
- No backend/API changes — `metadata.raw_description` and the `skip` endpoint semantics are unchanged.
- No E2E/Playwright setup (out of scope; `vitest` + `@vue/test-utils` is a component-level test, not a browser test).
- No adjustment of `skip` by the count of transactions a save removes — the raw value is preserved as-is (see Decision 2).

## Decisions

### 1. Single reactive trigger: `watch(skip, loadQueue)` replaces ad hoc explicit calls

Today, "reload the queue" is implemented twice (once per handler) instead of once, which is exactly why a third way of changing `skip` — direct URL edit, back/forward, any same-document route change — was never wired up: there was no single place that *all* changes flow through.

```ts
watch(skip, loadQueue)
```

`handleSkip` and `handleSave` stop calling `loadQueue()` themselves; they only change `skip` (via `setSkip`), and the watcher does the rest. This makes "skip changed" the one trigger for "reload," instead of "skip changed and you also remembered to call loadQueue."

**Alternative considered:** add a `watch(() => route.query.skip, ...)` directly on the route, bypassing the `skip` computed. Rejected — `skip` already normalizes non-numeric/negative query values to `0`; watching the computed keeps that normalization as the single source of truth for "what does skip currently mean," rather than duplicating the parsing logic in the watcher.

### 2. Reset-on-save removed outright, not adjusted

The original design reset `skip` to 0 after a save because removing matched transactions shifts queue positions, so a stale raw offset could land past a transaction the user never decided on. This change simply removes that reset rather than trying to compensate for it (e.g. subtracting the removed count) — deliberately: preserving the exact review position is the thing the user asked for, and the removed-count adjustment is extra logic for a problem (occasionally landing on an already-reviewed-adjacent transaction) that's minor compared to losing your place in a long review session.

**Alternative considered:** `skip -= min(skip, matched_count_removed)`. Rejected by explicit choice — raw preservation is simpler and was preferred over the "stay pointed at roughly the same transaction" semantics.

### 3. Raw description: CSS truncation + toggle, no new dependency

`metadata.raw_description` is rendered in a `text-truncate`-style single-line row (CSS `overflow: hidden; text-overflow: ellipsis; white-space: nowrap`), with a click/tap handler flipping a local `showFullDescription` ref that swaps the row's class to full-wrap. This mirrors the existing chevron-toggle pattern in `BudgetProgressBars.vue` without pulling in `v-expand-transition` (no need for an animated reveal here) or a second "show more" link element (`PanelTransactionList.vue`'s pattern) — one element, one class toggle.

### 4. Test infra: `vitest` + `@vue/test-utils`, scoped to this bug

Vite is already the build tool, so `vitest` needs no separate bundler config — `vitest.config.ts` merges the existing `vite.config.ts` and sets `test.environment = 'jsdom'`. `@vue/test-utils` mounts `UncategorizedReviewView.vue` with a real (or memory-history) Vue Router instance so route-query changes can be simulated directly, which is the only way to actually exercise the bug in Decision 1 (a route query change that isn't the Skip button).

**Alternative considered:** skip automated tests, rely on manual verification like the rest of this change's original scope. Rejected per explicit request — this bug's root cause (a missing reactive link) is exactly the kind of regression that manual click-through testing won't reliably catch, since the Skip button's own path still works fine.

The test work is split into two independent task sections (skip-reactivity, raw-description) that both depend only on the infra section, so they can be implemented by separate subagents in parallel once infra lands.

## Risks / Trade-offs

- **[Risk]** Removing the reset-on-save means a save can now land the user on a transaction adjacent to ones just categorized, in rare cases where the raw offset no longer lines up with the shifted queue → **Mitigation**: accepted per explicit user choice (Decision 2); the wrap-around (`skip % remaining`) still prevents an out-of-range offset, so the worst case is "slightly different transaction than expected," not an error.
- **[Risk]** Adding a `watch` alongside removing explicit `loadQueue()` calls could double-fetch on mount if not careful (watchers don't fire on initial value by default, but worth verifying) → **Mitigation**: tasks.md 5.4 calls this out explicitly as a manual check.
- **[Risk]** First-ever frontend test tooling in this repo is unreviewed scope creep risk (deps, config, CI hook-up) → **Mitigation**: scoped deliberately to dev dependencies only, no CI wiring beyond a `package.json` script (CI integration, if wanted, is a separate follow-up).

## Migration Plan

Not applicable — no schema, no API, no persisted-data changes. All three fixes and the test infra are additive frontend changes within a single deploy; rollback is reverting the commit(s).
