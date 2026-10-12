## 1. Crate scaffolding

- [x] 1.1 Add new `csv_import` binary crate under `backend/crates/csv_import`, wired into the Cargo workspace
- [x] 1.2 Add dependencies: `csv`, `encoding_rs`, `serde` + `serde_yaml` (or `serde_yml`), `clap` for CLI args, `thiserror` for error enums, plus `domain` path dependency for `Money`; add `assert2` as a dev-dependency
- [x] 1.3 Define CLI args with `clap`: `--input`, `--rules-file`, `--format <pretty|json>` (default `pretty`), `--no-interactive`
- [x] 1.4 Define top-level `CsvImportError` (`thiserror`, `#[from]` variants aggregating per-module errors) as the crate's `main` error type

## 2. CSV decoding and parsing

- [x] 2.1 Implement Windows-1250 → UTF-8 decoding of the input file; define `DecodeError` (`thiserror`) for invalid-byte failures
- [x] 2.2 Parse decoded CSV into raw rows using the `csv` crate, matching the 13-column header shape from the sample export; define `CsvParseError` (`thiserror`)
- [x] 2.3 Skip and warn on malformed rows instead of aborting
- [x] 2.4 Unit tests (`assert2::check!`/`let_assert!`) using the sample CSV rows as fixtures (both `Autooszczędzanie` and `Płatność kartą` shapes)

## 3. Field extraction

- [ ] 3.1 Implement extraction as a pure function (`&RawRow -> ParsedTransaction`, no I/O) of `Tytuł:`, `Lokalizacja:` (split into `Adres:`/`Miasto:`/`Kraj:`), `Numer karty:` (masked, keep last 4 digits), `Data wykonania operacji:`, `Oryginalna kwota operacji:` from the description columns; define `ExtractError` (`thiserror`) for unparseable amount/date fields
- [ ] 3.2 Build `ParsedTransaction` struct: date, amount (`Money`), currency, raw_type, card_last4, merchant, city, country, title, raw_description
- [ ] 3.3 Handle the no-merchant transfer case (`Autooszczędzanie`) gracefully (absent fields, not errors)
- [ ] 3.4 Unit tests (`assert2`) covering both row shapes from the sample data, calling the pure extraction function directly with in-memory `RawRow` fixtures

## 4. Rules engine

- [ ] 4.1 Define `Rule` struct and YAML schema: `match: { merchant_contains?, city?, country?, raw_type? }`, `category`
- [ ] 4.2 Implement field normalization (uppercase, diacritic-stripping) for both rule patterns and transaction fields, as a pure function
- [ ] 4.3 Implement first-rule-wins matching as a pure function (`(&ParsedTransaction, &[Rule]) -> Option<Category>`) against a loaded rule set
- [ ] 4.4 Implement loading/parsing an existing `--rules-file`; define `RulesError` (`thiserror`) for missing-file/invalid-YAML cases, with `#[from]` wired into `CsvImportError`
- [ ] 4.5 Unit tests (`assert2`) for match precedence and normalization (case/diacritic-insensitive matching), calling the pure matching function directly with in-memory `Rule`/`ParsedTransaction` fixtures

## 5. Interactive learning flow

- [ ] 5.1 Implement inline prompt (stdin) showing extracted fields when a transaction matches no rule
- [ ] 5.2 On answer, derive and append a new rule (`merchant_contains` default, or `raw_type` fallback) to the in-memory rule set and persist it back to `--rules-file`
- [ ] 5.3 Implement `--no-interactive`: skip prompting, assign `Uncategorized`, leave rules file untouched
- [ ] 5.4 Unit test (`assert2`) for rules-file round-trip (load → append learned rule → save → reload matches), and for the pure "derive a rule from a transaction + chosen category" function in isolation

## 6. Breakdown output

- [ ] 6.1 Aggregate categorized transactions into per-category totals and counts, as a pure function (`&[(ParsedTransaction, Category)] -> Breakdown`)
- [ ] 6.2 Implement `pretty` table output to stdout
- [ ] 6.3 Implement `json` output (breakdown + per-transaction extracted fields) to stdout
- [ ] 6.4 Unit tests (`assert2`) for aggregation logic

## 7. Verification

- [ ] 7.1 Wire the new crate into `nix build ./nix#backend-tests-cargo`
- [ ] 7.2 Run the CLI end-to-end against the sample CSV (both `pretty` and `json`, interactive and `--no-interactive`) and confirm output is correct
