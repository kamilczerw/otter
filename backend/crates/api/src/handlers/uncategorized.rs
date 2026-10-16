//! HTTP handlers for the "uncategorized" transaction queue (section 3 of
//! the `bulk-categorize-transactions` change):
//!
//! - `GET /api/v1/transactions-review` reports how many transactions
//!   are currently sitting in the "uncategorized" category and the one at
//!   the head of that queue, plus the starting rule it would learn.
//! - `POST /api/v1/transactions-review/preview` previews how many of
//!   those transactions a client-drafted rule would match, before it's
//!   saved.
//! - `POST /api/v1/transactions-review/rules` saves a new rule and
//!   immediately re-categorizes every currently-matching uncategorized
//!   transaction.

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use domain::entities::Transaction;
use domain::types::CategoryName;

use csv_import::{match_rule, MatchCriteria, Rule};

use crate::errors::ApiError;
use crate::import::{
    find_or_create_category, find_or_create_entry, find_or_create_month, ImportDeps,
    MatchableImportDetails, UNCATEGORIZED,
};
use crate::requests::{PageRequest, PreviewRuleRequest, SaveRuleRequest};
use crate::responses::{
    MatchCriterionResponse, PreviewRuleResponse, SaveRuleResponse, TransactionResponse,
    UncategorizedHeadResponse, UncategorizedQueueResponse,
};

use super::AppState;

/// Query parameters accepted by [`get_uncategorized_queue`].
#[derive(Debug, Deserialize)]
pub struct UncategorizedQueueQuery {
    /// How many positions to advance past the queue's natural head before
    /// picking `head`, wrapping back to the start once it reaches the
    /// queue's length. Defaults to 0 (the natural head) when absent.
    #[serde(default)]
    pub skip: u32,
}

/// Returns the current size of the "uncategorized" transaction queue and the
/// transaction at its head (offset by `skip`, wrapping around the queue),
/// along with the starting rule that would be learned from it.
///
/// # Errors
///
/// Returns an [`ApiError`] if the underlying transaction lookup fails.
#[utoipa::path(
    get,
    path = "/api/v1/transactions-review",
    tag = "uncategorized",
    params(
        ("skip" = Option<u32>, Query, description = "Positions to advance past the queue's head before picking it, wrapping around the queue (default 0)"),
    ),
    responses(
        (status = 200, description = "Current uncategorized queue state", body = UncategorizedQueueResponse),
    )
)]
pub async fn get_uncategorized_queue(
    State(state): State<AppState>,
    Query(query): Query<UncategorizedQueueQuery>,
) -> Result<Json<UncategorizedQueueResponse>, ApiError> {
    let queue = state
        .import_deps
        .transaction_repo
        .list_by_category_name(UNCATEGORIZED)
        .await?;

    Ok(Json(build_queue_response(queue, query.skip)))
}

/// Builds the response body from the full list of uncategorized
/// transactions: `remaining` is the list's length (unaffected by `skip`),
/// `head` describes the transaction at position `skip modulo remaining`
/// (if any) plus its derived starting rule. An empty queue always yields
/// no head, regardless of `skip`.
fn build_queue_response(queue: Vec<Transaction>, skip: u32) -> UncategorizedQueueResponse {
    let remaining = queue.len() as u32;
    let head_index = head_index(remaining, skip);
    let head = head_index
        .and_then(|index| queue.into_iter().nth(index))
        .map(build_head_response);

    UncategorizedQueueResponse { remaining, head }
}

/// Resolves the queue position `head` should come from: `skip modulo
/// remaining`, wrapping automatically once `skip` reaches the queue's
/// length. `None` when the queue is empty (there is no position to wrap
/// into, and dividing by zero would panic).
fn head_index(remaining: u32, skip: u32) -> Option<usize> {
    if remaining == 0 {
        return None;
    }
    Some((skip % remaining) as usize)
}

/// Builds the head-of-queue response for `transaction`: its own API
/// representation, plus the starting rule [`csv_import::derive_rule`] would
/// learn from its import metadata. `None` when the transaction has no
/// import metadata at all (manually created, so no rule could ever have
/// matched it).
fn build_head_response(transaction: Transaction) -> UncategorizedHeadResponse {
    let derived_rule = transaction
        .import_details
        .as_ref()
        .map(derive_rule_criteria);

    UncategorizedHeadResponse {
        transaction: TransactionResponse::from(transaction),
        derived_rule,
    }
}

/// Previews a draft rule: reports how many transactions currently in the
/// "uncategorized" category its match criteria would match, so the UI can
/// show a live count before the rule is saved (saving is a separate, later
/// endpoint). When the request includes a `page`, the response also
/// includes that page of the actual matching transactions (in queue order)
/// and `other_count`, their total with `exclude_transaction_id` (if given)
/// left out — `count` always reports every match regardless of exclusion.
///
/// # Errors
///
/// Returns an [`ApiError`] if the request's criteria don't convert cleanly
/// (see [`PreviewRuleRequest`]'s `TryFrom`) or the underlying transaction
/// lookup fails.
#[utoipa::path(
    post,
    path = "/api/v1/transactions-review/preview",
    tag = "uncategorized",
    request_body = PreviewRuleRequest,
    responses(
        (status = 200, description = "Count of uncategorized transactions the draft rule matches, plus an optional page of them", body = PreviewRuleResponse),
        (status = 400, description = "A criterion set neither or both of `contains`/`regex`"),
    )
)]
pub async fn preview_uncategorized_rule(
    State(state): State<AppState>,
    Json(request): Json<PreviewRuleRequest>,
) -> Result<Json<PreviewRuleResponse>, ApiError> {
    let exclude_transaction_id = request.exclude_transaction_id.clone();
    let page_request = request.page.as_ref().map(|page| PageRequest {
        offset: page.offset,
        limit: page.limit,
    });
    let criteria = MatchCriteria::try_from(request)
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    let rule = Rule {
        pattern: criteria,
        category: String::new(),
    };

    let queue = state
        .import_deps
        .transaction_repo
        .list_by_category_name(UNCATEGORIZED)
        .await?;

    let count = count_matches(&queue, &rule);
    let (other_count, transactions) = match &page_request {
        Some(page) => {
            let page = build_match_page(&queue, &rule, exclude_transaction_id.as_deref(), page);
            (Some(page.other_count), Some(page.transactions))
        }
        None => (None, None),
    };

    Ok(Json(PreviewRuleResponse {
        count,
        other_count,
        transactions,
    }))
}

/// Filters `queue` down to the transactions `rule` matches, preserving the
/// queue's existing order. A transaction with no import metadata never
/// matches (there's nothing for the criteria to check), mirroring
/// [`build_head_response`]'s treatment of such transactions. Shared by
/// [`count_matches`] and [`build_match_page`] so both read the same filter
/// logic.
fn matching_transactions<'a>(queue: &'a [Transaction], rule: &Rule) -> Vec<&'a Transaction> {
    queue
        .iter()
        .filter(|transaction| {
            transaction
                .import_details
                .as_ref()
                .is_some_and(|details| match_rule(&MatchableImportDetails(details), rule))
        })
        .collect()
}

/// Counts how many transactions in `queue` match `rule`. Always reports the
/// overall total, regardless of any `exclude_transaction_id` a paged
/// request might also carry — exclusion only ever affects
/// [`build_match_page`]'s `other_count`/page.
fn count_matches(queue: &[Transaction], rule: &Rule) -> u32 {
    matching_transactions(queue, rule).len() as u32
}

/// Server-side cap on how many transactions a single previewed page can
/// return, regardless of the `limit` a client requests — keeps a
/// pathological request from forcing a huge response.
const MAX_PREVIEW_PAGE_SIZE: u32 = 50;

/// One page of a draft rule's actual matches, plus the match total it was
/// sliced from (`other_count`) — both excluding `exclude_transaction_id`,
/// if one was given.
struct MatchPage {
    other_count: u32,
    transactions: Vec<TransactionResponse>,
}

/// Builds a page of `rule`'s matches in `queue`: `exclude_transaction_id`
/// (if any) is dropped from the matches first, then `[offset, offset +
/// limit)` is sliced out of what's left, with `limit` clamped to
/// [`MAX_PREVIEW_PAGE_SIZE`]. An `offset` at or beyond the number of
/// (post-exclusion) matches yields an empty page rather than an error.
fn build_match_page(
    queue: &[Transaction],
    rule: &Rule,
    exclude_transaction_id: Option<&str>,
    page: &PageRequest,
) -> MatchPage {
    let matches = matches_excluding(queue, rule, exclude_transaction_id);
    let limit = page.limit.min(MAX_PREVIEW_PAGE_SIZE) as usize;

    let transactions = matches
        .iter()
        .skip(page.offset as usize)
        .take(limit)
        .map(|transaction| TransactionResponse::from((*transaction).clone()))
        .collect();

    MatchPage {
        other_count: matches.len() as u32,
        transactions,
    }
}

/// `rule`'s matches in `queue`, with `exclude_transaction_id` (if any)
/// dropped from the result — the shared filtering step behind
/// [`build_match_page`]'s page and its `other_count`.
fn matches_excluding<'a>(
    queue: &'a [Transaction],
    rule: &Rule,
    exclude_transaction_id: Option<&str>,
) -> Vec<&'a Transaction> {
    matching_transactions(queue, rule)
        .into_iter()
        .filter(|transaction| match exclude_transaction_id {
            Some(excluded_id) => transaction.id.to_string() != excluded_id,
            None => true,
        })
        .collect()
}

/// Derives the starting rule's match criteria from `import_details`, via
/// [`csv_import::derive_rule`]. `derive_rule`'s `category` argument is
/// unused by matching, so an empty placeholder is passed and only the
/// derived `pattern`'s criteria are ever exposed to the client.
fn derive_rule_criteria(import_details: &domain::entities::ImportDetails) -> Vec<MatchCriterionResponse> {
    let rule = csv_import::derive_rule(&MatchableImportDetails(import_details), &String::new());
    rule.pattern.0.into_iter().map(MatchCriterionResponse::from).collect()
}

/// Saves a new categorization rule and immediately re-categorizes every
/// transaction currently sitting in the "uncategorized" queue that it
/// matches, moving each one onto its own (auto-provisioned) budget entry
/// under the target category.
///
/// # Errors
///
/// Returns an [`ApiError`]:
/// - `400` if a criterion is malformed (see [`SaveRuleRequest`]'s
///   `TryFrom`), any `regex` criterion fails to compile, or `category`
///   isn't a valid [`CategoryName`].
/// - Otherwise, any error from loading/persisting the rules file,
///   provisioning a month/category/entry, or updating a transaction.
#[utoipa::path(
    post,
    path = "/api/v1/transactions-review/rules",
    tag = "uncategorized",
    request_body = SaveRuleRequest,
    responses(
        (status = 200, description = "Remaining uncategorized count after applying the new rule", body = SaveRuleResponse),
        (status = 400, description = "A criterion was malformed, a `regex` failed to compile, or `category` is invalid"),
    )
)]
pub async fn save_uncategorized_rule(
    State(state): State<AppState>,
    Json(request): Json<SaveRuleRequest>,
) -> Result<Json<SaveRuleResponse>, ApiError> {
    let category_name = CategoryName::new(request.category.clone())
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;

    let criteria = MatchCriteria::try_from(request)
        .map_err(|e| ApiError::bad_request(&e.to_string()))?;
    reject_uncompilable_regexes(&criteria)?;

    let rule = Rule {
        pattern: criteria,
        category: category_name.as_str().to_string(),
    };
    persist_rule(&state.import_deps, rule.clone()).await?;

    apply_rule_to_queue(&state.import_deps, &rule, &category_name).await?;

    let remaining = state
        .import_deps
        .transaction_repo
        .list_by_category_name(UNCATEGORIZED)
        .await?
        .len() as u32;
    Ok(Json(SaveRuleResponse { remaining }))
}

/// Checks that every `regex` criterion in `criteria` compiles, rejecting
/// the whole rule with a `400` naming the offending pattern otherwise. Run
/// before any persistence so an uncompilable pattern never reaches the
/// rules file (it would silently never match, per [`csv_import`]'s
/// treatment of a bad pattern at match time).
fn reject_uncompilable_regexes(criteria: &MatchCriteria) -> Result<(), ApiError> {
    for criterion in &criteria.0 {
        if let csv_import::MatchType::Regex(pattern) = &criterion.match_type {
            if regex::Regex::new(pattern).is_err() {
                return Err(ApiError::bad_request(&format!(
                    "invalid regex pattern: {pattern}"
                )));
            }
        }
    }
    Ok(())
}

/// Appends `rule` to the rules file at `deps.rules_path` and writes it
/// back, so it's persisted before any transaction gets re-categorized
/// against it. Deliberately reads only the file-based rules (not
/// `deps.inline_rules`) — the config's inline rules aren't this function's
/// to rewrite.
async fn persist_rule(deps: &ImportDeps, rule: Rule) -> Result<(), ApiError> {
    let mut rules = csv_import::load_rules_or_empty(&deps.rules_path)?;
    rules.push(rule);
    csv_import::save_rules(&deps.rules_path, &rules)?;
    Ok(())
}

/// Re-categorizes every uncategorized transaction that `rule` matches:
/// moves it onto the budget entry for `category_name` in its own
/// transaction month, auto-provisioning that month/category/entry if
/// they don't exist yet (same provisioning helpers the import pipeline
/// uses).
async fn apply_rule_to_queue(
    deps: &ImportDeps,
    rule: &Rule,
    category_name: &CategoryName,
) -> Result<(), ApiError> {
    let queue = deps.transaction_repo.list_by_category_name(UNCATEGORIZED).await?;

    for transaction in queue {
        if transaction_matches_rule(&transaction, rule) {
            recategorize_transaction(deps, &transaction, category_name).await?;
        }
    }
    Ok(())
}

/// Whether `transaction` matches `rule`, via its import metadata. A
/// transaction with no import metadata never matches, mirroring
/// [`count_matches`].
fn transaction_matches_rule(transaction: &Transaction, rule: &Rule) -> bool {
    transaction
        .import_details
        .as_ref()
        .is_some_and(|details| match_rule(&MatchableImportDetails(details), rule))
}

/// Moves one matching transaction onto the budget entry for
/// `category_name`, in the budget month its own `date` falls in.
async fn recategorize_transaction(
    deps: &ImportDeps,
    transaction: &Transaction,
    category_name: &CategoryName,
) -> Result<(), ApiError> {
    let (category, _) = find_or_create_category(deps, category_name).await?;
    let (month, _) = find_or_create_month(deps, transaction.date.value()).await?;
    let entry = find_or_create_entry(deps, month.id, category.id).await?;

    deps.transaction_service
        .update(&transaction.id, Some(entry.id), None, None, None)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::entities::ImportDetails;
    use domain::types::{Money, TransactionDate};

    /// Builds a `Transaction` with the given `import_details`, leaving all
    /// other fields at arbitrary fixed values irrelevant to this handler's
    /// logic.
    fn transaction_with_import_details(import_details: Option<ImportDetails>) -> Transaction {
        let now = chrono::Utc::now();
        Transaction {
            id: ulid::Ulid::new(),
            entry_id: ulid::Ulid::new(),
            amount: Money::new(-100),
            date: TransactionDate::new(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            title: None,
            created_at: now,
            updated_at: now,
            import_details,
            tags: vec![],
        }
    }

    /// Builds a bare transaction distinguishable only by `id`, for tests
    /// that only care about which queue position ends up as `head`.
    fn transaction_with_id(id: ulid::Ulid) -> Transaction {
        let mut transaction = transaction_with_import_details(None);
        transaction.id = id;
        transaction
    }

    /// An empty queue reports zero remaining and no head.
    #[test]
    fn empty_queue_has_no_head() {
        let response = build_queue_response(vec![], 0);
        assert_eq!(response.remaining, 0);
        assert!(response.head.is_none());
    }

    /// An empty queue has no head regardless of a nonzero skip.
    #[test]
    fn empty_queue_has_no_head_regardless_of_skip() {
        let response = build_queue_response(vec![], 5);
        assert_eq!(response.remaining, 0);
        assert!(response.head.is_none());
    }

    /// A head transaction with a merchant derives a merchant-contains rule.
    #[test]
    fn head_with_merchant_derives_merchant_rule() {
        let details = ImportDetails {
            source: "csv_import".to_string(),
            merchant: Some("NETTO".to_string()),
            city: None,
            country: None,
            card_last4: None,
            raw_type: None,
            currency: None,
            raw_description: None,
            original_amount_fragment: None,
            raw_fragments: None,
        };
        let response = build_queue_response(vec![transaction_with_import_details(Some(details))], 0);

        assert_eq!(response.remaining, 1);
        let head = response.head.expect("head");
        let derived = head.derived_rule.expect("derived rule");
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].contains, Some("NETTO".to_string()));
    }

    /// A head transaction with no import details at all has no derived rule.
    #[test]
    fn head_without_import_details_has_no_derived_rule() {
        let response = build_queue_response(vec![transaction_with_import_details(None)], 0);

        let head = response.head.expect("head");
        assert!(head.derived_rule.is_none());
    }

    /// `skip=1` returns the second transaction in queue order as head.
    #[test]
    fn skip_one_returns_second_transaction() {
        let first = ulid::Ulid::new();
        let second = ulid::Ulid::new();
        let response =
            build_queue_response(vec![transaction_with_id(first), transaction_with_id(second)], 1);

        assert_eq!(response.remaining, 2);
        assert_eq!(response.head.expect("head").transaction.id, second.to_string());
    }

    /// Skip wraps back to the start once it reaches (or exceeds) the queue
    /// length, and `remaining` stays the total count regardless of skip.
    #[test]
    fn skip_wraps_around_queue_without_affecting_remaining() {
        let ids: Vec<ulid::Ulid> = (0..3).map(|_| ulid::Ulid::new()).collect();
        let queue = || ids.iter().map(|id| transaction_with_id(*id)).collect::<Vec<_>>();
        let head_id = |skip| build_queue_response(queue(), skip).head.expect("head").transaction.id;

        assert_eq!(build_queue_response(queue(), 0).remaining, 3);
        assert_eq!(build_queue_response(queue(), 3).remaining, 3);
        assert_eq!(build_queue_response(queue(), 4).remaining, 3);
        assert_eq!(head_id(0), head_id(3), "skip == remaining wraps to the head");
        assert_eq!(head_id(1), head_id(4), "skip > remaining wraps consistently");
    }

    /// Builds a queue of `n` transactions that all match any rule with no
    /// criteria (vacuous AND), each carrying distinct merchant import
    /// details so they're individually identifiable by id.
    fn matching_queue(n: usize) -> Vec<Transaction> {
        (0..n)
            .map(|i| {
                let details = ImportDetails {
                    source: "csv_import".to_string(),
                    merchant: Some(format!("MERCHANT {i}")),
                    city: None,
                    country: None,
                    card_last4: None,
                    raw_type: None,
                    currency: None,
                    raw_description: None,
                    original_amount_fragment: None,
                    raw_fragments: None,
                };
                transaction_with_import_details(Some(details))
            })
            .collect()
    }

    /// A vacuous rule (no criteria) matches every transaction with import
    /// details.
    fn vacuous_rule() -> Rule {
        Rule {
            pattern: MatchCriteria(vec![]),
            category: String::new(),
        }
    }

    /// With no page requested, `build_match_page` is never called by the
    /// handler — `count_matches` alone still reports the full total,
    /// unaffected by any later pagination logic.
    #[test]
    fn count_matches_reports_full_total_without_paging() {
        let queue = matching_queue(5);
        assert_eq!(count_matches(&queue, &vacuous_rule()), 5);
    }

    /// Excluding a transaction id removes it from both the page and the
    /// page's own total (`other_count`), but never from the overall
    /// `count_matches` total.
    #[test]
    fn excluded_transaction_id_removed_from_page_and_other_count_only() {
        let queue = matching_queue(3);
        let excluded_id = queue[0].id.to_string();
        let rule = vacuous_rule();
        let page_request = PageRequest { offset: 0, limit: 10 };

        let page = build_match_page(&queue, &rule, Some(excluded_id.as_str()), &page_request);

        assert_eq!(page.other_count, 2);
        assert_eq!(page.transactions.len(), 2);
        assert!(page.transactions.iter().all(|t| t.id != excluded_id));
        assert_eq!(count_matches(&queue, &rule), 3, "overall count is unaffected");
    }

    /// Requesting an offset at or beyond the number of matches returns an
    /// empty page (not an error).
    #[test]
    fn offset_past_the_end_returns_empty_page() {
        let queue = matching_queue(3);
        let rule = vacuous_rule();
        let page_request = PageRequest { offset: 10, limit: 10 };

        let page = build_match_page(&queue, &rule, None, &page_request);

        assert_eq!(page.other_count, 3, "total is unaffected by paging past the end");
        assert!(page.transactions.is_empty());
    }

    /// A requested `limit` above the server's fixed cap is silently
    /// clamped, never returning more than `MAX_PREVIEW_PAGE_SIZE`
    /// transactions in one page.
    #[test]
    fn limit_above_cap_is_clamped() {
        let queue = matching_queue((MAX_PREVIEW_PAGE_SIZE + 10) as usize);
        let rule = vacuous_rule();
        let page_request = PageRequest {
            offset: 0,
            limit: MAX_PREVIEW_PAGE_SIZE + 10,
        };

        let page = build_match_page(&queue, &rule, None, &page_request);

        assert_eq!(page.transactions.len(), MAX_PREVIEW_PAGE_SIZE as usize);
    }
}
