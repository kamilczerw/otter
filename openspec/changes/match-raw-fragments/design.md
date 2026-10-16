## Context

The match-criteria model (`csv_import::rules`) is already field-agnostic: `MatchField` is a closed enum, `Matchable` is a trait with one accessor per field, `field_value()` maps a `MatchField` to its accessor, and `match_type_matches`/`criterion_matches`/`match_rule`/`categorize` are generic over any `Matchable` implementor. Both ends of the current `derive_rule`/`categorize_or_learn` CLI flow and the web `MatchableImportDetails` wrapper already carry `raw_fragments: Option<String>` on their underlying types (`ParsedTransaction`, `ImportDetails`) — it's just not wired into the enum/trait/accessor triplet yet. See proposal.md for why this field specifically is being added now.

## Goals / Non-Goals

**Goals:**
- Make `raw_fragments` selectable and matchable (contains + regex) everywhere the existing five fields are, with zero special-casing relative to them.
- Keep the change additive: existing rules, requests, and responses are unaffected.

**Non-Goals:**
- Auto-suggesting `raw_fragments` in `derive_rule` (confirmed manual-only during brainstorming).
- Any label-assignment use of `raw_fragments` (explicitly deferred, per proposal.md).
- Any change to how `raw_fragments` is computed or displayed (`extract.rs`'s join logic, `TransactionDetailCard.vue`'s display row) — those already exist from `preserve-unmapped-csv-import-fragments`.

## Decisions

- **Add `RawFragments` as a sixth `MatchField` variant, not a separate "fragment matching" mechanism.** The existing enum/trait/accessor pattern is exactly shaped for "one more field"; introducing a parallel mechanism (e.g. a dedicated `raw_fragments_regex` request field) would duplicate the contains/regex logic for no benefit. Alternative considered and rejected: special-case `raw_fragments` matching only via `regex` (skip `contains`) — rejected because every other field supports both match types uniformly, and a future CSV row might have a fragment a simple substring check is enough for; asymmetry would need its own documentation/tests for no real gain.
- **No new API shape.** `MatchCriterion`/`MatchCriteria`, `PreviewRuleRequest`, `SaveRuleRequest`, and the derived-rule response already carry a `field: MatchField` (or its wire mirror) generically — adding the enum variant and its two `From` arms (`MatchFieldRequest -> MatchField`, `MatchField -> MatchFieldResponse`) is sufficient; no handler, route, or DTO shape changes.
- **`derive_rule` is untouched.** Per the brainstorming decision, auto-suggestion stays merchant/raw_type-only; a user who wants to match on `raw_fragments` adds the criterion manually via "Add criterion". This keeps `derive_rule`'s existing tests and semantics stable.

## Risks / Trade-offs

- **Regex over a joined, multi-fragment string can be harder to write correctly than over a single-purpose field** (e.g. matching "the second fragment" requires anchoring within the `" | "`-joined text). Mitigated by this being an expert/manual feature (added via "Add criterion", not suggested) and by the live match-count preview already in the review UI, which lets the user verify a pattern's effect before saving.
- **None of the existing five fields' tests change**, so regression risk is limited to additive new test cases plus the mechanical enum/match-arm additions (exhaustive `match` on `MatchField` means a missed call site fails to compile, not silently at runtime).
