## Context

See `proposal.md` - Why/Impact for motivation and affected areas. Key existing pieces this builds on:

- `csv_import::rules` (`backend/crates/csv_import/src/rules.rs`): `Rule`/`MatchCriteria`/`MatchCriterion`/`MatchField`/`MatchType`, `match_rule`/`categorize` (the matching engine), `load_rules`/`save_rules`/`load_rules_or_empty` (file I/O). The matching engine stays as-is and keeps being the thing that actually evaluates rules against a transaction; only where rules *come from* changes.
- `ImportDeps` (`backend/crates/api/src/import/provisioning.rs`): bundles `inline_rules` (config defaults, `Vec<csv_import::Rule>`) and `rules_path`, and `current_rules()` concatenates them, read fresh every call.
- `MatchCriterionRow.vue` and the `MatchField`/criteria types in `frontend/src/api/types.ts`: the existing rule-criteria editing UI, built for the uncategorized-review "learn a rule" flow, reused here rather than duplicated.
- Hexagonal layering (`domain` ← `db`, `domain` ← `api`, `api` → `csv_import`): `domain` cannot depend on `csv_import`, so the new DB-backed entity needs its own copy of the match-criteria shape.

## Goals / Non-Goals

**Goals:**
- Replace file-based mutable rule storage with a DB table, keeping config-default rules file-based and read-only.
- Keep the existing matching engine (`csv_import::match_rule`/`categorize`) as the single place matching logic lives; the new code only produces the `Vec<csv_import::Rule>` it already consumes.
- Keep the standalone `csv_import` CLI binary's file-based workflow completely unaffected.

**Non-Goals:**
- No data migration path from an existing `rules.toml` into the database. Per the proposal, this is an internal breaking change with no migration tooling — anyone with rules in `rules.toml` re-enters or re-imports them via the new UI. (If the user later wants a one-time import-on-first-run, that's a separate, small follow-up, not part of this change.)
- No change to how `match_rule`/`categorize` evaluate a rule against a transaction.

## Decisions

### Domain types duplicate csv_import's match-criteria shape, rather than domain depending on csv_import
`domain` has zero framework/adapter dependencies by project rule, and `csv_import` is effectively an adapter-ish crate (TOML I/O, CLI). Rather than restructure crate boundaries, `domain::entities::matching_rule` declares its own `MatchField`/`MatchType`/`MatchCriterion` enums with the same variants as `csv_import::rules`'s. The `api` crate (which already depends on both) owns a small `From<domain::MatchCriterion> for csv_import::MatchCriterion`-style conversion used when building the categorization rule list. This is a few enums' worth of duplication, not logic duplication — the actual matching code (`match_rule`) is never reimplemented.

**Alternative considered:** move `csv_import::rules`'s types and matching logic into `domain` wholesale, and have `csv_import`'s CLI depend on `domain`. Rejected for this change: `domain` currently has no I/O/CLI-adjacent dependencies (`clap`, encoding, etc. all live in `csv_import`), and reshaping that boundary is a bigger, separate refactor than this feature needs.

### Regex validation moves to construction time in the domain type
`csv_import::match_rule` currently treats a regex that fails to compile as "never matches" (silent). `domain::MatchCriterion::new` instead validates and returns `MatchingRuleError::InvalidRegex` immediately. This matches the project's "invalid states unrepresentable" newtype convention and gives the editor UI a real error to show instead of a rule that silently does nothing. `csv_import::match_rule` itself is unchanged (still defensive) since it may still see arbitrarily-sourced `csv_import::Rule` values from the CLI's own files.

### `position` column added now, not exposed for reordering yet
Per the proposal's Non-Goals, there is no reorder UI. The column is added now (assigned as `max(position)+1` on insert) purely to avoid a backfill migration later, since the user has flagged that ordering will matter soon. `list_all` orders by `position ASC`.

### Rule identity: ULID, not array position
Database rows get a real identity (ULID primary key), replacing the old file-position addressing a prior draft of this design considered. This is what makes in-place edit well-defined (the spec's "Edit a user rule in place" requirement needs a stable identity that survives other rows being added/removed) and is also simply how every other entity in this codebase (`Category`, `Transaction`, etc.) is identified.

### Import endpoint reuses `csv_import::RulesFile` for parsing, not a new format
The uploaded file is parsed with the existing TOML shape (`csv_import::RulesFile`/`Rule`) — it's already the documented, hand-editable export format, and reusing it means nothing new needs inventing for "how do I get rules out of instance A" (answer, unchanged: copy/hand-edit a TOML file in that shape). Only the *destination* of an import (the DB, via `MatchingRuleRepository::create`) is new.

### Dedup is exact structural equality, not fuzzy matching
A rule from the uploaded file is skipped only if an existing user rule has identical criteria (same fields, same match types, same patterns, in the same order) and the same category. No semantic/overlap detection (e.g. "this new regex would also catch what an existing rule catches"). Simple, predictable, and matches what "duplicate" means to a user hand-assembling a file to copy between instances.

## Risks / Trade-offs

- **[Risk]** Existing deployments with a populated `rules.toml` lose those rules silently on upgrade (the API server stops reading the file). → **Mitigation**: call this out prominently in the PR/release notes as a breaking change; the file itself is left on disk (not deleted) so the user can still open it and re-import it through the new UI's file-upload.
- **[Risk]** `current_rules()` becoming `async` and DB-backed adds a query to every CSV import request where it was previously a file read. → **Mitigation**: same cost class as a file read for the data volumes this project targets (single household, modest rule counts); no caching introduced, matching the project's existing stance on not pre-optimizing the import path.
- **[Trade-off]** Domain `MatchCriterion` duplicates `csv_import`'s enum shape rather than sharing it. → Accepted: see Decisions above; revisit only if more adapters need the same shape.
