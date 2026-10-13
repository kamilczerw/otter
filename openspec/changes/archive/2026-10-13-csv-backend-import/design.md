## Context

See proposal.md for motivation. Relevant current state:

- `backend/crates/csv_import` is a CLI-only binary crate. Its `rules.rs` module owns `Rule`/`MatchCriteria` (serde types) and `normalize`/`match_rule`/`categorize` (pure functions), currently persisted as a bare YAML list via `serde_yaml`.
- The backend follows strict hexagonal architecture: `api -> domain <- db`. `domain` has zero framework/HTTP/DB dependencies. Each aggregate (`Transaction`, `BudgetEntry`, `Category`, `Month`) has a repository port trait in `domain::ports` and a `*Service` in `domain::services` that does cross-aggregate validation before delegating to the repo.
- `TransactionService::create(entry_id, amount, date, title)` requires an existing `entry_id` — there is no "find or create" helper anywhere for month/category/entry today, and no bulk/batch endpoint exists in the codebase.
- `EntryService::create(month_id, category_id, budgeted, due_day)` already verifies both month and category exist.
- `MonthService::create(month, copy_from, empty)` already supports creating an empty month with no copied entries (`empty: true`).

## Goals / Non-Goals

**Goals:**
- Reuse `csv_import`'s existing pure parsing/categorization functions as a library from `api`, without duplicating that logic.
- Reuse the existing domain services unchanged — no new domain service type, just new orchestration calling the ones that exist.
- Make the rules file format (TOML) usable both as a standalone file and inlined in `config.toml`, without two separate schemas to maintain.

**Non-Goals:**
- Rules management via API (add/edit rules through HTTP) — deferred.
- Any frontend/UI trigger for import — this change is API-only.
- Transactional atomicity across the whole import (see Risks below) — row-level auto-creation is deliberately incremental, not wrapped in one giant DB transaction.

## Decisions

### Where the orchestration logic lives
New module `api/src/import/` (not a new `domain` service). The CSV/bank-format concern (decoding, row parsing, rule matching) is adapter-level, not core domain logic, and the per-row orchestration (resolve-or-create month → resolve-or-create category → resolve-or-create entry → dedup check → create transaction) is a use case that composes four existing domain services — it doesn't introduce new domain rules of its own. Putting it in `api` keeps `domain` free of the `csv_import` dependency and avoids inventing a fifth service for what is really a one-off pipeline.

Alternative considered: a new `domain::services::ImportService`. Rejected because `domain` would then need to depend on `csv_import` (a crate full of bank-CSV-specific parsing, not business logic), breaking the "domain has no framework/external-format dependencies" rule, and a service that only calls other services end-to-end adds a layer of indirection with no reuse benefit — nothing else will call it.

### Rules file format: TOML, with a `matching_rules` wrapper key
`Rule`/`MatchCriteria` keep their current fields; only the serde backend changes (`serde_yaml` → `toml`). TOML has no bare top-level array, so both the standalone file and the inline config block use the same shape:

```toml
[[matching_rules]]
category = "groceries"
[matching_rules.match]
merchant_contains = "netto"
```

A thin wrapper struct (`struct RulesFile { matching_rules: Vec<Rule> }`) is deserialized from either the standalone `rules.toml` or the `[transaction.import]` section of `config.toml`; both paths produce the same `Vec<Rule>` that `categorize()` already consumes. The CLI's `load_rules`/`save_rules` and `learn.rs` switch to this format too, so there is exactly one rules format and one (de)serialization path shared by the CLI and the API — not two formats to keep in sync.

Alternative considered: keep YAML for the CLI and add a separate TOML reader for the API's config-embedded case. Rejected — two formats for the same `Rule` schema is pure duplication for no benefit, and the user explicitly wants config.toml and rules file in the same format.

### Splitting out rules: a generic top-level `include`, not a dedicated `rules_file`
`[transaction.import]` has a single `matching_rules: Option<Vec<Rule>>`. To keep rules in their own file instead of inlining them, `AppConfig::load` supports a generic top-level `include = ["./rules.toml"]` directive: paths (relative to the including file, or absolute) are loaded as additional lower-priority config sources before the main file. The main file's own values win on any key collision — ordinary config precedence, no rules-specific special case. If `matching_rules` is unset everywhere, rules default to empty (nothing categorizes; everything falls to `"uncategorized"`).

Only one level of `include` is resolved (an included file's own `include` list, if any, is ignored) — sufficient for splitting out a rules file, and simpler than supporting arbitrary include nesting.

### Auto-creation order and scope
Per transaction, in order: find-or-create `Month` (empty, no copy-from) → find-or-create `Category` by exact name → find-or-create `BudgetEntry` for (month, category) at `budgeted = 0` → duplicate check → create `Transaction`. This needs two small new lookups that don't exist on any port today:
- "find category by name" — add to the import orchestration as a linear scan over `CategoryRepository::list_all()` (category counts are small; no need for a new indexed repo method).
- "find entry by (month, category)" — same approach, scan `BudgetEntryRepository::list_by_month(month_id)` and match on `category.id`.

Both reuse existing port methods rather than adding new ones, keeping the port surface minimal. The one new port method actually needed is duplicate detection (below), because that check can't be done by scanning a small in-memory list — an entry can accumulate many transactions over time.

### Duplicate detection: new repository port method
`TransactionRepository` gains `find_duplicate(entry_id, date, amount, title) -> Result<Option<Transaction>, TransactionError>`, implemented in `db` as an indexed SQL lookup (`WHERE entry_id = ? AND date = ? AND amount = ? AND title = ?`, or `title IS NULL` when absent). Checked before every `TransactionService::create` call during import.

Alternative considered: fetch all transactions for the entry via `list_by_entry` and compare in memory. Rejected for entries that accumulate many transactions over time (e.g. a recurring category spanning years) — an indexed query is the same amount of code and doesn't degrade.

### Partial success semantics
Two failure classes, handled differently, matching the proposal:
- **Structural** (the CSV can't be parsed into rows at all — `csv_import::csv_input::parse_rows` returns an error): the whole request is rejected before any DB writes happen. Nothing to partially import.
- **Per-row** (`extract_transaction` fails for one row): that row is recorded in the response's `failed` list (position, raw content, reason) and processing continues with the next row. This mirrors the CLI's existing "skip row, print a warning, continue" behavior, just surfaced in the HTTP response instead of stderr.

Duplicates and uncategorized transactions are not failures — they're expected, routine outcomes — so they're counted, not itemized, to keep the response bounded in size even for a large overlapping re-import.

### HTTP status codes
`400` for request-level problems (bad multipart, unrecognized charset, structurally invalid CSV) — mapped via the existing `ApiError::bad_request` pattern, no new domain error variant needed. `201` when `imported > 0`; `200` when the request succeeded but `imported == 0`. No `207` (WebDAV-specific per its RFC; reusing it risks confusing any tooling that assumes that scope).

## Risks / Trade-offs

- **No whole-request DB transaction** → a crash mid-import could leave some rows persisted and others not, with no way to resume cleanly except re-running the import (which is safe, since duplicate detection makes re-running idempotent for already-persisted rows). Mitigation: duplicate detection doubles as the recovery mechanism; no separate rollback logic needed.
- **Category-by-name lookup is a linear scan** → fine at personal-budget-tool scale (tens of categories), would need an indexed lookup if category counts grew into the hundreds. Not expected for this app's use case.
- **Rules format migration is a manual, one-time step** → `rules.yaml` is not auto-converted; the user must hand-migrate it to `rules.toml`. Acceptable since this is a personal tool with one rules file, not a multi-tenant migration concern.
- **`find_duplicate`'s matching key (entry_id, date, amount, title) could under- or over-match** for legitimately repeated transactions (e.g. two separate ATM withdrawals of the same amount on the same day with no title). Accepted trade-off per the proposal discussion — titled card transactions carry a unique-enough reference in `title` in practice, and the alternative (itemizing every duplicate) was explicitly rejected for response-size reasons.
