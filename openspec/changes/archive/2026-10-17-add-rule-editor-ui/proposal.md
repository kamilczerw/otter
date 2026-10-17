## Why

There is no UI to see, edit, or delete categorization rules, or to move rules learned on one Otter instance to another. The current mutable rule store (`rules.toml`, hand-edited/appended-to by the "learn a rule" flow) also won't survive planned multi-user support, so this is the right time to move it into the database instead of building more UI around file I/O.

## What Changes

- New `/rules` view: lists every active rule (read-only config defaults first, then the user's own rules, in match order), with create/edit/delete for the user's own rules.
- Import: upload a `.toml` rules file (as another instance would export via hand-editing/copying `rules.toml` today) to add its rules to the current instance, skipping exact duplicates.
- **BREAKING (internal only, no user-facing migration path)**: the mutable rule set moves from `rules.toml` into a new `matching_rules` database table. Existing `rules.toml` files are no longer read by the API server; anyone with rules there must re-enter or re-import them via the new UI.
- Config-defined defaults (`config.toml`'s `[transaction.import] matching_rules`) stay file-based, read-only, and keep matching first, unchanged.
- The standalone `csv_import` CLI binary and its own `--rules-file` flag are untouched (separate, file-based offline tool).

## Capabilities

### New Capabilities
- `matching-rules`: CRUD and file-import of user-owned categorization rules (stored in the database), merged with read-only config-default rules into the ordered rule set the import pipeline matches against.

### Modified Capabilities
(none — `transaction-import`'s requirement that imports use "the same rule-matching semantics as the CSV import rules engine (first-rule-wins, normalized field matching)" is unaffected; only where rules are stored changes, not how matching behaves.)

## Impact

- **Domain** (`backend/crates/domain`): new `MatchingRule`/`NewMatchingRule`/`MatchCriterion` entities, new `MatchingRuleRepository` port, new `MatchingRuleError`.
- **DB** (`backend/crates/db`): new migration adding the `matching_rules` table; new `MatchingRuleRepo` implementation.
- **API** (`backend/crates/api`): `ImportDeps::current_rules()` becomes async and DB-backed instead of reading `rules_path`; new `handlers/rules.rs` with `GET/POST/PUT/DELETE /api/v1/rules[/{id}]` and `POST /api/v1/rules/import`; existing callers of `current_rules()` (import pipeline, uncategorized "save rule" handler) updated for the new async signature.
- **Frontend**: new `RulesView.vue`, `RuleEditorSheet.vue` (reusing existing `MatchCriterionRow.vue`), `api/rules.ts`, new `/rules` route, new entry in `SettingsDrawer.vue`.
- **Removed**: `ImportDeps.rules_path` and the API server's use of `csv_import::load_rules_or_empty`/`save_rules` for the live rule set (the `csv_import` crate itself keeps these functions for its standalone CLI).
