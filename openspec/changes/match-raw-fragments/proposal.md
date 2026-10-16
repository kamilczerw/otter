## Why

`raw_fragments` (added by `preserve-unmapped-csv-import-fragments`) captures the trailing CSV columns the bank's row-shape parsing doesn't yet understand — the one field most likely to carry the free text a user wants to match on when a transaction's shape isn't one of the recognized ones (e.g. a phone or reference number buried in a `Płatność web - kod mobilny` row). The Review Uncategorized rule editor currently lets a user match on five fields (`merchant`, `raw_type`, `city`, `country`, `description`) but not `raw_fragments`, so there's no way to build a categorization rule around that data short of hand-editing the TOML rules file.

## What Changes

- `category-rules`'s match-criteria model gains a sixth matchable field, `raw_fragments`, usable with both existing match types (`contains`, `regex`), matched against the transaction's joined raw-fragments string exactly like the other five fields.
- Backend: `csv_import::rules::MatchField` gains a `RawFragments` variant; the `Matchable` trait gains a `raw_fragments()` accessor, implemented for both `ParsedTransaction` and the API's `MatchableImportDetails` wrapper (both already carry the field); the request/response wire-format mirrors (`MatchFieldRequest`/`MatchFieldResponse`) gain the matching variant.
- Frontend: the `MatchField` type and the criterion-row field selector (`MatchCriterionRow.vue`) gain a `raw_fragments` option, with new PL/EN i18n labels.
- Not changing: the backend's starting-rule suggestion (`derive_rule`) still only derives from `merchant`/`raw_type` — `raw_fragments` is a manual "Add criterion" choice only, not auto-suggested.
- Out of scope: using `raw_fragments` (or any matched field) to assign labels to transactions — that's a distinct, separate future capability and isn't touched here.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `category-rules`: the match-criteria model's fixed field set grows from five fields to six, adding `raw_fragments`.

## Impact

- **Backend `csv_import` crate** (`crates/csv_import/src/rules.rs`): `MatchField` enum, `Matchable` trait, `field_value()`, and the `ParsedTransaction` `Matchable` impl each gain a `raw_fragments`/`RawFragments` case.
- **Backend `api` crate**: `crates/api/src/import/matchable.rs` (`MatchableImportDetails`'s `Matchable` impl), `crates/api/src/requests/mod.rs` (`MatchFieldRequest` + its `From` impl), `crates/api/src/responses/mod.rs` (`MatchFieldResponse` + its `From` impl) each gain the new variant/case.
- **Frontend**: `frontend/src/api/types.ts` (`MatchField` union), `frontend/src/components/uncategorized/MatchCriterionRow.vue` (`fieldOptions`), `frontend/src/i18n/en.json` + `pl.json` (new `uncategorized.fields.raw_fragments` label).
- **No DB migration, no API route changes** — purely an added enum case threaded through the existing generic field-matching machinery.
- **No breaking changes** — existing rules (none of which reference `raw_fragments`) are unaffected; this is strictly additive to the field set.
