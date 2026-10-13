## Why

CSV imports fall back to a fixed `"uncategorized"` category whenever no configured rule matches (`transaction-import`'s "Rule-based categorization on import"), and today the only way to add a matching rule is to hand-edit the `matching_rules` TOML consulted once at server startup (`api/src/config.rs`, `main.rs:96`). There is no way, from the app itself, to look at the pile of uncategorized transactions and teach the system what they are. This change adds a web UI (and the API behind it) to review uncategorized transactions one at a time, derive a reusable rule from each, and apply it immediately to every other uncategorized transaction it matches — the same one-by-one learning loop the `csv_import` CLI already has (`learn.rs`), moved into the app and made live-editable instead of requiring a restart.

## What Changes

- Add a global review queue of every transaction currently categorized `"uncategorized"`, independent of month.
- For the transaction at the head of the queue, derive a starting match rule from its import metadata (merchant first, falling back to raw type), the same heuristic `derive_rule` uses today, and let the user edit it before saving.
- **BREAKING**: Restructure the rule-matching shape from four fixed optional fields (`merchant_contains`, `raw_type`, `city`, `country`) to an ordered list of criteria, each naming one field (`merchant`, `raw_type`, `city`, `country`, or `description`) and a match type (plain substring-contains, or regex), all AND'ed together. Existing `rules.toml` files using the old flat shape must be rewritten by hand to the new shape; there is no auto-migration.
- Add `description` as a matchable field, matched against an imported transaction's raw bank description (`ImportDetails.raw_description`); it has no value for manually-created transactions, which are simply unreachable by rules using it.
- Let the UI preview, before saving, how many of the currently-uncategorized transactions a draft rule would match, updating live as the rule is edited.
- On save, persist the rule to the shared rules file and immediately re-categorize every uncategorized transaction it matches (not just the one on screen), moving each one onto a budget entry for the target category in its own transaction month (auto-creating that budget entry if needed, via the same provisioning logic CSV import already uses).
- Make the running server pick up rule file edits without a restart: rules are read fresh at the point they're used (import time, queue listing, preview, apply) rather than cached once at startup.
- No standalone rule list/edit/delete UI in this change — rules can still be hand-edited in the TOML file if a saved one needs correcting.
- No free-text/path-based query language and no AND/OR rule trees in this change (explicitly out of scope — see Design doc's "Future direction" note); the criteria-list shape is chosen so a future AND/OR generalization doesn't require reshaping it again.

## Capabilities

### New Capabilities
- `category-rules`: the persisted, hand-and-UI-editable rule engine (criteria-list match shape, file-backed storage, live reload) and the uncategorized-transaction review queue API/UI built on it (preview match count, save-and-apply-to-all-matches).

### Modified Capabilities
- `csv-import`: the rule-matching requirement changes from four fixed optional fields to an ordered list of `{field, match_type}` criteria, with `description` added as a fifth matchable field and `regex` added as a second match type alongside substring-contains.
- `transaction-import`: rules are no longer loaded once at server startup — each import request resolves rules fresh from the rules file, so a rule saved through the new UI applies to the very next import without a restart.

## Impact

- `backend/crates/csv_import`: `MatchCriteria`/`Rule` shape change (`rules.rs`), `match_rule`/`categorize` logic rewritten for the criteria-list + regex shape, TOML (de)serialization updated, `derive_rule` (`learn.rs`) updated to emit the new shape.
- `backend/crates/api`: `config.rs` (rules resolution no longer cached into a startup `Vec`, read fresh from the resolved rules file path instead), `import/categorize.rs` and `import/pipeline.rs` (generalize matching to run against both a freshly-parsed CSV row and an already-persisted `Transaction`'s `ImportDetails`), new handlers + routes for the uncategorized queue (list next/count, preview match count for a draft rule, save rule & apply), reuse of `import/provisioning.rs`'s `find_or_create_category`/`find_or_create_entry`.
- `backend/crates/domain`: no entity changes — `ImportDetails` and `Transaction` already carry everything the new matching needs.
- `frontend`: new review-queue view/component and API client calls; no changes to existing views.
- Existing hand-written `rules.toml` files (if any exist in deployed configs) need manual rewriting to the new criteria-list shape — flagged as a breaking, manual migration step, not automated by this change.
