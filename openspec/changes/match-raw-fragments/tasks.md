## 1. Backend: `csv_import` crate — `raw_fragments` as a matchable field

- [x] 1.1 Add `RawFragments` to `csv_import::rules::MatchField`, add `fn raw_fragments(&self) -> Option<&str>` to the `Matchable` trait, implement it for `ParsedTransaction` (returns `self.raw_fragments.as_deref()`), and add the matching arm in `field_value()`; verify `cargo build -p csv_import` compiles (the exhaustive `match` in `field_value` forces the new arm).
- [x] 1.2 Add `contains`/`regex` test cases for `MatchField::RawFragments` to `rules.rs`'s existing test module (mirroring `matches_description_field`/`matches_regex_criterion`), including a case where the transaction has no `raw_fragments` value and the criterion correctly fails to match; verify via `cargo test -p csv_import`.

## 2. Backend: `api` crate — wire `raw_fragments` through `Matchable` and the request/response DTOs

- [x] 2.1 Add `fn raw_fragments(&self) -> Option<&str>` to `MatchableImportDetails`'s `Matchable` impl in `crates/api/src/import/matchable.rs` (returns `self.0.raw_fragments.as_deref()`), and extend its existing `single_criterion_never_matches_a_bare_import_details` test loop to include `MatchField::RawFragments`; verify via `cargo test -p api`.
- [x] 2.2 Add a `RawFragments` variant to `MatchFieldRequest` in `crates/api/src/requests/mod.rs` and its `From<MatchFieldRequest> for MatchField` arm; add the mirrored `RawFragments` variant to `MatchFieldResponse` in `crates/api/src/responses/mod.rs` and its `From<MatchField> for MatchFieldResponse` arm; verify `cargo build -p api` compiles.
- [x] 2.3 Add an integration test case (in `crates/api/tests/uncategorized_tests.rs`, alongside the existing preview/save-rule tests) that previews or saves a rule with a `raw_fragments` criterion against a persisted transaction whose `ImportDetails.raw_fragments` is set, and asserts it matches; verify via `cargo test -p api` (or `nix build ./nix#backend-tests-cargo`).

## 3. Frontend: expose `raw_fragments` in the rule editor

- [x] 3.1 Add `'raw_fragments'` to the `MatchField` union type in `frontend/src/api/types.ts`.
- [x] 3.2 Add a `raw_fragments` entry to `fieldOptions` in `frontend/src/components/uncategorized/MatchCriterionRow.vue`, labeled via `$t('uncategorized.fields.raw_fragments')`.
- [x] 3.3 Add the `uncategorized.fields.raw_fragments` key to `frontend/src/i18n/en.json` ("Raw fragments") and `frontend/src/i18n/pl.json` (matching existing PL field-label style); verify `npm run type-check` passes.

## 4. Verification

- [x] 4.1 Run `nix build ./nix#backend-tests-cargo` and confirm all backend tests pass.
- [x] 4.2 Run `npm run type-check` in `frontend/` and confirm it passes.
- [x] 4.3 Manually exercise the Review Uncategorized page: open a transaction whose `metadata.raw_fragments` is populated (e.g. one imported from a row shape that didn't fully parse), add a criterion with field `raw_fragments` and a `regex` pattern matching part of that string, confirm the live match-count preview reflects it, and confirm saving the rule categorizes the matching transactions. Verified at the API level (see report) against a real seeded transaction; UI-level browser automation was not used.
