## 1. `csv_import` crate: criteria-list rule engine

- [x] 1.1 Replace `MatchCriteria`'s four fixed fields with `MatchField` (Merchant/RawType/City/Country/Description), `MatchType` (Contains(String)/Regex(String)), `MatchCriterion { field, match_type }`, and `MatchCriteria(Vec<MatchCriterion>)` in `rules.rs`, with `Serialize`/`Deserialize` producing a hand-editable nested TOML shape (e.g. `[[matching_rules.match]]` entries with `field`/`contains` or `field`/`regex` keys); verify round-trip via an updated `save_rules_round_trips_through_toml` test.
- [x] 1.2 Add a `Matchable` trait (`merchant`/`raw_type`/`city`/`country`/`description` accessors returning `Option<&str>`) and implement it for `ParsedTransaction`; rewrite `match_rule`/`categorize` to be generic over `T: Matchable` and to compile+apply each criterion's match type (reusing `normalize`/`normalized_contains` for `Contains`, `regex::Regex::new` for `Regex`); verify existing `rules.rs` unit tests pass under the new shape (rewritten for the new criteria-list construction) plus new tests for regex criteria and the `description` field.
- [x] 1.3 Update `learn.rs`'s `derive_rule` to emit a single-criterion `Contains` rule on `merchant` (or `raw_type` if no merchant) in the new shape; verify its existing unit tests pass with updated assertions.
- [x] 1.4 Add the `regex` crate as a `csv_import` dependency; verify `cargo build -p csv_import` succeeds.

## 2. API: dynamic, file-backed rules + persisted `Transaction` matching

- [x] 2.1 Replace `ImportConfig.matching_rules` with `ImportConfig.rules_file: PathBuf` (resolved relative to the config file like existing `include` paths) in `config.rs`; remove `resolve_rules`/`ImportConfig::Default` rule-list behavior; verify updated `config.rs` tests cover a configured `rules_file` resolving to an absolute path.
- [x] 2.2 Replace `ImportDeps.rules: Vec<Rule>` with `ImportDeps.rules_path: PathBuf` and add `ImportDeps::current_rules(&self) -> Result<Vec<Rule>, RulesError>` calling `csv_import::load_rules_or_empty`; update `main.rs` wiring to pass the resolved path instead of a pre-loaded `Vec`; verify `cargo build -p api` succeeds and existing `pipeline.rs`/`provisioning.rs` tests are updated to the new `ImportDeps` shape.
- [x] 2.3 Update `import_one_transaction` (`pipeline.rs`) to call `deps.current_rules()` instead of reading a cached field; verify `falls_back_to_uncategorized_when_no_rule_matches` and `imports_transaction_matching_a_rule` tests still pass, plus a new test that a rule written to the rules file after `ImportDeps` is constructed is picked up by the next `run_import` call.
- [x] 2.4 Implement `Matchable` for `domain::entities::ImportDetails` (mapping its `merchant`/`city`/`country`/`raw_type`/`raw_description` fields), returning `None` for every accessor when build from a transaction with no import details; verify a unit test that a `MatchCriteria` with any criterion never matches an all-`None` `ImportDetails`.
- [x] 2.5 Add a transaction-repository query for "all transactions currently in the `uncategorized` category, across all months" (reusing existing category/entry lookup patterns from `provisioning.rs`); verify a repository-level test returns transactions from more than one month.

## 3. API: uncategorized queue endpoints

- [ ] 3.1 Add `GET /api/v1/uncategorized-transactions` returning the queue's remaining count and the transaction at its head (with its import metadata and a derived starting rule per design.md Decision 1 / spec's "A starting rule is derived..." requirement); verify an integration test covering an empty queue and a populated one.
- [ ] 3.2 Add `POST /api/v1/uncategorized-transactions/preview` accepting a draft `MatchCriteria` and returning the count of currently-uncategorized transactions it matches; verify an integration test that narrowing the draft rule's criteria lowers the returned count.
- [ ] 3.3 Add `POST /api/v1/uncategorized-transactions/rules` accepting `{ criteria: MatchCriteria, category: String }`: validates any `regex` criterion compiles (reject with 400 and no persistence if not), appends the rule via `csv_import::save_rules` at `ImportDeps.rules_path`, then re-categorizes every currently-uncategorized matching transaction (via `find_or_create_category`/`find_or_create_entry` from `provisioning.rs`, updating each transaction's `entry_id`), returning the new remaining queue count; verify an integration test asserting saved-rule persistence (reloading the rules file), the count drop, and that a subsequent import applies the new rule.
- [ ] 3.4 Add `utoipa` OpenAPI annotations for the three new endpoints and request/response DTOs in `responses/mod.rs`/`requests/mod.rs`, following the existing `ImportResponse`/`TransactionListQuery` patterns; verify the OpenAPI spec generation step (however this project invokes it) includes the new paths without errors.

## 4. Frontend: review queue view

- [ ] 4.1 Add typed API client functions for the three new endpoints in `frontend/src/api/` (new `uncategorized.ts`, following `transactions.ts`'s patterns), with types added to `types.ts`.
- [ ] 4.2 Add a review-queue component (e.g. `UncategorizedReviewView.vue`) under `frontend/src/views/`: shows the head transaction's import metadata, an editable criteria-list form (field/match-type/pattern per criterion, add/remove criterion), a live match-count preview (debounced call to the preview endpoint on edit), a category picker, and a save-and-apply action that reloads the next head transaction and updated count.
- [ ] 4.3 Add a route and navigation entry point to the new view, following the existing router/bottom-navigation conventions.
- [ ] 4.4 Verify with `npm run type-check` and a manual run through `npm run dev`: import a CSV producing uncategorized transactions, confirm the queue count, save a rule from the first transaction, confirm the count drops by the expected amount and the queue advances.

## 5. Migration and cleanup

- [ ] 5.1 Update any existing `config.toml`/included rules file in this repo (e.g. local dev config, if one exists) from the old flat `matching_rules` shape to the new criteria-list shape pointed at by `rules_file`; verify `nix build ./nix#backend` (or local `cargo build`) starts cleanly against the migrated config.
- [ ] 5.2 Run `nix build ./nix#backend-tests-cargo` and confirm all backend tests pass after the full change; fix any fallout from the `MatchCriteria`/`ImportDeps` shape change in tests not already covered above.
