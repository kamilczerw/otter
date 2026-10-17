## 1. Domain

- [x] 1.1 Add `backend/crates/domain/src/entities/matching_rule.rs`: `MatchField`, `MatchType`, `MatchCriterion` (with `MatchCriterion::new` validating a `Regex` pattern compiles, returning `MatchingRuleError::InvalidRegex` on failure), `MatchingRule`, `NewMatchingRule`. Verify with unit tests covering valid/invalid regex construction and a valid `Contains` construction.
- [x] 1.2 Add `MatchingRuleError` (e.g. `InvalidRegex`, `NotFound`) to `backend/crates/domain/src/errors/`. Verify `cargo build -p domain` succeeds.
- [x] 1.3 Add `backend/crates/domain/src/ports/matching_rule_repo.rs`: `MatchingRuleRepository` trait (`list_all`, `create`, `update`, `delete`). Register the new module in `domain`'s `entities`/`ports` `mod.rs` files. Verify `cargo test -p domain` passes.

## 2. Database

- [x] 2.1 Add migration `backend/crates/db/migrations/005_add_matching_rules.sql` creating the `matching_rules` table (`id TEXT PRIMARY KEY`, `criteria TEXT NOT NULL`, `category TEXT NOT NULL`, `position INTEGER NOT NULL UNIQUE`, `created_at TEXT NOT NULL`, `updated_at TEXT NOT NULL`) plus the `updated_at` trigger matching the existing tables. Verify `sqlx migrate run` applies cleanly against a fresh test database.
- [x] 2.2 Implement `backend/crates/db/src/matching_rule_repo.rs`: `MatchingRuleRepo` implementing `MatchingRuleRepository`, serializing `criteria` as JSON, `create` assigning `position = COALESCE(MAX(position), 0) + 1`, `list_all` ordering by `position ASC`. Verify with integration tests against a real SQLite test database: create/list ordering, update, delete, delete/update of a nonexistent id returns `NotFound`.

## 3. API: wire the DB into the import pipeline

- [x] 3.1 Add `rule_repo: Arc<dyn MatchingRuleRepository>` to `ImportDeps` (`backend/crates/api/src/import/provisioning.rs`); remove `rules_path`. Make `current_rules()` `async`, returning config `inline_rules` followed by `rule_repo.list_all()` mapped to `csv_import::Rule` via a new `From`/conversion function. Verify `cargo build -p api` succeeds (compile errors surface every call site needing an `.await`).
- [x] 3.2 Update every caller of `current_rules()` (import pipeline in `backend/crates/api/src/import/pipeline.rs`, the uncategorized "save rule" handler in `backend/crates/api/src/handlers/uncategorized.rs`) for the new async signature. Verify existing `imports_tests.rs`/`uncategorized_tests.rs` integration tests still pass.
- [x] 3.3 Update `backend/crates/api/src/main.rs` wiring: construct `MatchingRuleRepo` and pass it into `ImportDeps` instead of `rules_path`. Verify the API server boots locally against a fresh/migrated database.

## 4. API: rules CRUD + import endpoints

- [x] 4.1 Add request/response types for rule CRUD (criteria + category) and the import summary (`imported`, `skipped`, `failed`) to `backend/crates/api/src/requests/mod.rs` / `responses/mod.rs`, reusing the existing `MatchCriterionResponse` shape where possible.
- [x] 4.2 Add `backend/crates/api/src/handlers/rules.rs`: `GET /api/v1/rules` (defaults + user rules), `POST /api/v1/rules`, `PUT /api/v1/rules/{id}`, `DELETE /api/v1/rules/{id}`, mapping `MatchingRuleError::InvalidRegex`/`NotFound` to 400/404 via `ApiError`. Register routes in `backend/crates/api/src/handlers/mod.rs` / router setup. Verify with integration tests: create, list (defaults before user rules), edit happy path + edit-unknown-id 404, delete happy path + delete-unknown-id 404, create-with-invalid-regex 400.
- [x] 4.3 Add `POST /api/v1/rules/import`: multipart `.toml` upload parsed via `csv_import::RulesFile`, skip rules exactly matching an existing user rule (criteria + category equality), insert the rest via `rule_repo.create`, collect per-rule regex-compile failures without aborting the batch. Verify with integration tests: all-new import, partial-duplicate import, import containing one invalid regex (rest still imported, failure reported).
- [x] 4.4 Add `utoipa` OpenAPI annotations for the new endpoints, matching the existing handler documentation style. Verify the OpenAPI spec generation step (however the project currently runs it) includes the new paths.

## 5. Frontend

- [x] 5.1 Add `frontend/src/api/rules.ts` (`list`, `create`, `update`, `remove`, `importFile`) and corresponding types in `frontend/src/api/types.ts`. Verify `npm run type-check` passes.
- [x] 5.2 Add `frontend/src/views/RulesView.vue`: fetches and renders the merged list (greyed-out/non-interactive default rules first, then user rules with Edit/Delete), header actions for "+ New rule" and "Import file" (native file input, `.toml` accept). Verify by running the dev server and visually confirming the list renders both groups correctly with seeded config defaults + a couple of DB rules.
- [x] 5.3 Add `frontend/src/components/rules/RuleEditorSheet.vue`: bottom-sheet form reusing `MatchCriterionRow.vue` for criteria rows plus a category text field, used for both create and edit. Wire it into `RulesView.vue`. Verify by creating a rule, editing it, and confirming the change reflects in the list without a full page reload.
- [x] 5.4 Add route `/rules` in `frontend/src/router/index.ts`. Verify navigating to `/rules` directly renders `RulesView.vue`.
- [x] 5.5 Add a "Rules" entry to `frontend/src/components/layout/SettingsDrawer.vue`'s Data section, next to "Uncategorized", navigating to `/rules`. Verify by opening Settings in the running app and confirming the link navigates correctly.
- [x] 5.6 Add i18n strings for the new view/labels in both `en` and `pl` locale files. Verify `npm run type-check` passes (and no missing-key warnings in the console when switching locales).

## 6. Full verification

- [x] 6.1 Run `nix build ./nix#backend-tests-cargo` and confirm all backend tests pass (required by project convention before this change is considered done).
- [x] 6.2 Run `npm run build` in `frontend/` and confirm it type-checks and builds cleanly.
- [x] 6.3 Manually verify the end-to-end import flow: export a small `.toml` rules file by hand (same shape as today's `rules.toml`), import it via the new UI, confirm the rules appear in the list and correctly categorize a subsequent CSV import.
