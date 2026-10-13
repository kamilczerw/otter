## Context

The bank export (see sample in the change discussion) is a CSV with Windows-1250 encoding and 13 columns, only the first 7 of which are named in the header. The remaining 6 columns carry labeled free-text fragments (`Tytuł:`, `Lokalizacja: Adres: ... Miasto: ... Kraj: ...`, `Numer karty: ...`, etc.) whose population differs by `Typ transakcji` (e.g. `Autooszczędzanie` transfers have no merchant; `Płatność kartą` rows do). This is a standalone prototype — it does not touch the existing `domain`/`db`/`api` crates beyond reusing `domain::Money`, and has no database or UI integration yet.

## Goals / Non-Goals

**Goals:**
- Reliable extraction of merchant, city, country, card last 4 digits, and title from the messy description columns.
- Deterministic, explainable categorization via rules matched on any combination of extracted fields (not merchant alone, since merchant text is noisy).
- A rules file that is both machine-updated (learned from corrections) and human-editable (YAML).
- `pretty`/`json` output selectable via `--format`.
- A non-interactive mode for scripting/repeat runs.

**Non-Goals:**
- No database storage of transactions, categories, or rules — everything lives in the YAML rules file for this prototype.
- No fuzzy/statistical/ML-based matching — substring/exact matching on normalized fields only.
- No UI or API integration — CLI only.
- No multi-file batch import or multi-currency conversion — single CSV in, native currency passthrough.
- No support for encodings other than Windows-1250 input / UTF-8 output.

## Decisions

- **Matching strategy: normalized multi-field rules, not fuzzy/ML.** Merchant text alone is unreliable (e.g. truncated titles, inconsistent spacing), so rules can match on any combination of `merchant_contains`, `city`, `country`, `raw_type`. All comparisons are done on uppercased, diacritic-stripped versions of the fields. This was chosen over a statistical classifier because it stays simple, deterministic, and inspectable for a CLI prototype — explicitly deferred as a non-goal.
- **Learning mechanism: append-only rule file, not weighted/statistical.** Every interactive correction appends one new rule (`merchant_contains` on the normalized merchant by default, or `raw_type` when there's no merchant) to the rules file. First-rule-wins matching. The user can also hand-edit the file directly to merge/broaden rules.
- **Rules/config location: explicit `--rules-file` flag, no fixed path.** The user wants to pass an arbitrary YAML file path rather than relying on a project- or user-config-dir convention. There is no separate "categories" definition — any string used in a `category:` field, whether typed at a prompt or hand-written, is valid.
- **Encoding: hardcoded Windows-1250 → UTF-8 decode, no `--encoding` flag.** Only one bank format is in scope right now; add a flag if/when a second encoding shows up.
- **Interactive correction: inline prompt during the run, not a separate batch-review pass.** When a transaction matches no rule, the tool pauses, shows the extracted fields, and asks for a category before continuing — simpler to implement and matches how the user will actually run this against one statement at a time. `--no-interactive` bypasses prompting entirely (useful for scripted re-runs) and leaves unmatched transactions as `Uncategorized`.
- **Crate placement: new workspace member depending on `domain`.** Keeps the amount type (`Money`) consistent with the rest of the system and makes a future move into the `api`/`db` layers (when this becomes a real import feature) a smaller step, without actually wiring any DB/API dependency now.
- **Error handling: `thiserror`.** Each module defines a focused error enum (e.g. `DecodeError`, `ExtractError`, `RulesError`, `CsvImportError` at the top level aggregating the others via `#[from]`), matching the convention already used in `domain`/`db`/`api`. No `anyhow`/stringly-typed errors.
- **Testing: `assert2`.** Unit tests use `assert2::{check, let_assert}` instead of stdlib `assert_eq!`/`assert!` for clearer failure output. Code is structured to make this easy: extraction, normalization, matching, and aggregation are implemented as small pure functions (`&RawRow -> ParsedTransaction`, `(&Transaction, &[Rule]) -> Option<Category>`, etc.) taking plain data in and returning plain data out, with I/O (file reads, stdin prompts) kept in thin wrappers at the edges — so the core logic is tested directly without mocking CSV files, YAML files, or stdin.

## Risks / Trade-offs

- Substring matching on normalized fields can over-match (e.g. a short merchant substring matching an unrelated transaction). Mitigated by first-rule-wins ordering and the fact that the user curates/can edit the rules file; not a concern worth more machinery at prototype stage.
- Field extraction is regex/pattern-based against the known label format (`Tytuł:`, `Lokalizacja:`, `Numer karty:`); a different bank or a format change would require updating the parser. Acceptable since this prototype targets one known export format.
- No persistence of categorized results — each run recomputes the breakdown from the input CSV; if this needs to accumulate history across statements later, that's a future change.
