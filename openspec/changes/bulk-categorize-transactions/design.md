## Context

Today's rule engine (`csv_import::rules`) is a pure, in-memory matcher: `MatchCriteria` is four fixed `Option<String>` fields, `match_rule`/`categorize` take a `&ParsedTransaction` (a freshly-parsed CSV row), and rules only ever come from `AppConfig.transaction.import.matching_rules` — resolved once in `main.rs:96` into an `Arc<ImportDeps>` built once at process start (`main.rs:97-105`). `ImportDeps.rules: Vec<csv_import::Rule>` is a frozen snapshot for the life of the process; the only way to add a rule today is to edit `config.toml` (or an included file) and restart.

Rules currently exist in two possible places in config: inline `matching_rules` in the main file, or the same key in a file named by top-level `include = [...]` (`config.rs:240-269`), merged by the generic `config` crate before `AppConfig` is deserialized once. That merge step has no notion of "the one file the app writes to" — it just flattens N TOML sources into one.

The new review queue needs to match against transactions that are already persisted (`Transaction` + its optional `ImportDetails`), not just freshly-parsed CSV rows — `match_rule` needs to run against both shapes.

## Goals / Non-Goals

**Goals:**
- A rule saved through the UI is visible to the very next CSV import and the very next queue/preview call, with no server restart.
- One rule engine, one `MatchCriteria` shape, shared unchanged by the CLI, the CSV-import API pipeline, and the new review queue.
- Reuse `import/provisioning.rs`'s find-or-create helpers for re-pointing a transaction's `entry_id` at a (possibly new) budget entry — no parallel implementation.

**Non-Goals:**
- No rule list/edit/delete UI (proposal's scope).
- No AND/OR rule trees or free-text/path query language. The criteria-list shape (`Vec<MatchCriterion>`, each `{field, match_type}`) is deliberately not named or structured around "AND" so that a future `Or(Vec<MatchCriteria>)` layer can wrap it without reshaping `MatchCriterion` itself — but that layer is not built here.
- No auto-migration of existing hand-written `matching_rules` — flagged as a manual step in the proposal.

## Decisions

### 1. `MatchCriteria` becomes a criteria list, not four fixed fields

```rust
pub enum MatchField { Merchant, RawType, City, Country, Description }
pub enum MatchType {
    Contains(String), // case/diacritic-insensitive substring, same normalize() as today
    Regex(String),    // compiled once per load, no normalization applied
}
pub struct MatchCriterion { pub field: MatchField, pub match_type: MatchType }
pub struct MatchCriteria(pub Vec<MatchCriterion>); // AND'ed, same as today's implicit AND
```

`raw_type`/`city`/`country` lose their "exact match only" restriction — a `Contains` criterion against them is now substring, same as merchant was. This is a quiet behavior change but keeps one match semantics per match type instead of per field, which is simpler to reason about and test. Regex gets no normalization (no diacritic/case folding): a regex author can write `(?i)` or the diacritic themselves; folding would silently rewrite the pattern's character classes in surprising ways.

**Alternative considered**: keep the four fixed fields and bolt on a `*_regex` sibling per field (the first, narrower idea from brainstorming). Rejected once `description` and the "all fields get regex" requirement landed — five fields × two match types as flat struct fields is 10 near-duplicate optional fields and a 10-branch `match_rule`; the list form is one loop over a 2-case match type, and adding a 6th field later is a new enum variant, not a struct-shape change propagated through serialization, matching, and the UI form.

### 2. Matching runs against a small `Matchable` trait, not `ParsedTransaction` directly

```rust
pub trait Matchable {
    fn merchant(&self) -> Option<&str>;
    fn raw_type(&self) -> Option<&str>;
    fn city(&self) -> Option<&str>;
    fn country(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
}
```

`match_rule`/`categorize` become generic over `T: Matchable`. `ParsedTransaction` implements it directly (its `raw_type` is a plain `String`, not `Option`, so the impl wraps it `Some`). A second impl reads straight off `&ImportDetails` (merchant/city/country/raw_type/raw_description — exactly the fields it already has); a transaction with `import_details: None` simply returns `None` for everything, so it matches only a rule with zero criteria (never saved in practice) and sits unreachable by rules, as the proposal already calls out.

**Alternative considered**: convert a persisted `Transaction`'s `ImportDetails` into a synthetic `ParsedTransaction` before matching. Rejected — `ParsedTransaction` carries parse-only fields (`date`, `amount`, `currency`, `card_last4`, `original_amount_fragment`) that have no meaning for an already-persisted row and would need fake values; the trait only exposes what matching actually touches.

### 3. Rules live in exactly one file, read fresh on every use — no more `matching_rules` in config

Replace `ImportConfig.matching_rules: Option<Vec<Rule>>` with `ImportConfig.rules_file: PathBuf` (resolved relative to the config file, same as today's `include` paths). `ImportDeps` drops its `rules: Vec<Rule>` field in favor of `rules_path: PathBuf`, and gains:

```rust
impl ImportDeps {
    fn current_rules(&self) -> Result<Vec<Rule>, RulesError> {
        csv_import::load_rules_or_empty(&self.rules_path)
    }
}
```

called at the top of `import_one_transaction`, and by the new queue/preview/apply handlers — never cached. Saving a rule calls `csv_import::save_rules` (already exists, unchanged) on the same path. This is a direct implementation of the brainstorm steer ("config.toml supports including extra files so we should create an empty file with rules we can edit via the API") without inventing new persistence machinery — `load_rules_or_empty`/`save_rules` already exist in `csv_import::rules` and already round-trip through the `RulesFile { matching_rules }` TOML wrapper; this change only points the API at one fixed, known, writable path instead of resolving rules once through the general config-merge pipeline.

**Alternative considered**: keep inline `matching_rules` as a static fallback layered under the dynamic file. Rejected per the brainstorm discussion — two sources of truth to reconcile and test for one feature's worth of value; the project is early enough that a clean breaking cutover is cheaper than permanent dual-path logic.

**Trade-off accepted**: every import row and every queue/preview/apply call now does a disk read of a small TOML file instead of a `Vec` clone. Rule files are expected to stay in the tens-to-low-hundreds of entries; this is not a hot path (import is a manual, occasional user action), so the simplicity win outweighs the I/O cost. If it ever matters, an `Arc<ArcSwap<Vec<Rule>>>` refreshed by a file-watcher is the natural upgrade — not needed now.

### 4. Apply = regular provisioning, not a new code path

"Save rule & apply" does, for every currently-`"uncategorized"` transaction the saved rule matches: resolve the category via `find_or_create_category` (already exists), resolve/create the budget entry for `(transaction's own month, that category)` via `find_or_create_entry` (already exists), then update the transaction's `entry_id` via the existing transaction update path. No new auto-provisioning rule is introduced — a matched transaction can move to a different month's budget entry only in the sense that "its own month" is fixed (a transaction's date never changes), so this only ever touches the entry linkage, never the month.

### 5. Match-count preview is just `categorize`-the-list, not a stored count

Previewing a draft rule (before saving) runs the same `Matchable`-generic matching function over every currently-uncategorized transaction's `ImportDetails` in memory and counts matches — no new index, no new query language, consistent with "no query DSL" from the brainstorm. Uncategorized transactions are expected to be at most a few thousand at a time; a linear scan is the same order of work `find_or_create_category`/`find_or_create_entry` already do today (both documented as linear scans over `list_all`/`list_by_month`).

## Risks / Trade-offs

- **[Risk]** Removing `matching_rules` from config is a breaking change for any already-deployed `config.toml` using the inline/include form → **Mitigation**: proposal already flags this as a manual, documented migration (rewrite the file at `rules_file`'s path, in the new criteria-list shape); this is a single-operator app, not a multi-tenant service, so the blast radius is one config file.
- **[Risk]** A user-supplied regex could be catastrophically slow (ReDoS-style) or simply invalid → **Mitigation**: validate the regex compiles at rule-save time (reject with a clear error, same place `CategoryName::new` validation already happens in the pipeline); no execution timeout is added since this is a trusted single-operator tool running on data it imported itself, not untrusted user input from the internet.
- **[Risk]** Reading the rules file from disk on every request adds a small amount of latency/IO to CSV import and queue endpoints → **Mitigation**: accepted per Decision 3; revisit only if it's ever measured as a problem.

## Migration Plan

1. Ship the new `MatchCriteria`/`Matchable`/rules-file-path code as one deploy; there is no dual-running period.
2. Anyone with an existing `config.toml` using inline or included `matching_rules` must hand-rewrite those rules into the new criteria-list shape and point `transaction.import.rules_file` at that file before restarting. No automated converter is provided (proposal's explicit choice).
3. No DB migration — no schema changes; `ImportDetails`/`Transaction` already carry everything this change matches against.
4. Rollback is reverting the deploy; the rules file itself (new TOML shape) isn't read by the old code, so a rollback would need the old flat-field rules shape restored from backup/VCS if rules were edited in the interim — acceptable given this is a personal, single-operator app.
