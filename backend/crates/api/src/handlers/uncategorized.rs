//! HTTP handlers for the "uncategorized" transaction queue (section 3 of
//! the `bulk-categorize-transactions` change):
//!
//! - `GET /api/v1/uncategorized-transactions` reports how many transactions
//!   are currently sitting in the "uncategorized" category and the one at
//!   the head of that queue, plus the starting rule it would learn.
//! - `POST /api/v1/uncategorized-transactions/preview` previews how many of
//!   those transactions a client-drafted rule would match, before it's
//!   saved.
//! - `POST /api/v1/uncategorized-transactions/rules` saves a new rule and
//!   immediately re-categorizes every currently-matching uncategorized
//!   transaction.

use axum::extract::State;
use axum::Json;

use domain::entities::Transaction;
use domain::types::CategoryName;

use csv_import::{match_rule, MatchCriteria, Rule};

use crate::errors::ApiError;
use crate::import::{
    find_or_create_category, find_or_create_entry, find_or_create_month, ImportDeps,
    MatchableImportDetails, UNCATEGORIZED,
};
use crate::requests::{PreviewRuleRequest, SaveRuleRequest};
use crate::responses::{
    MatchCriterionResponse, PreviewRuleResponse, SaveRuleResponse, TransactionResponse,
    UncategorizedHeadResponse, UncategorizedQueueResponse,
};

use super::AppState;

/// Returns the current size of the "uncategorized" transaction queue and the
/// transaction at its head, along with the starting rule that would be
/// learned from it.
///
/// # Errors
///
/// Returns an [`ApiError`] if the underlying transaction lookup fails.
#[utoipa::path(
    get,
    path = "/api/v1/uncategorized-transactions",
    tag = "uncategorized",
    responses(
        (status = 200, description = "Current uncategorized queue state", body = UncategorizedQueueResponse),
    )
)]
pub async fn get_uncategorized_queue(
    State(state): State<AppState>,
) -> Result<Json<UncategorizedQueueResponse>, ApiError> {
    let queue = state
        .import_deps
        .transaction_repo
        .list_by_category_name(UNCATEGORIZED)
        .await?;

    Ok(Json(build_queue_response(queue)))
}

/// Builds the response body from the full list of uncategorized
/// transactions: `remaining` is the list's length, `head` describes the
/// first transaction (if any) plus its derived starting rule.
fn build_queue_response(queue: Vec<Transaction>) -> UncategorizedQueueResponse {
    let remaining = queue.len() as u32;
    let head = queue.into_iter().next().map(build_head_response);

    UncategorizedQueueResponse { remaining, head }
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
/// endpoint).
///
/// # Errors
///
/// Returns an [`ApiError`] if the request's criteria don't convert cleanly
/// (see [`PreviewRuleRequest`]'s `TryFrom`) or the underlying transaction
/// lookup fails.
#[utoipa::path(
    post,
    path = "/api/v1/uncategorized-transactions/preview",
    tag = "uncategorized",
    request_body = PreviewRuleRequest,
    responses(
        (status = 200, description = "Count of uncategorized transactions the draft rule matches", body = PreviewRuleResponse),
        (status = 400, description = "A criterion set neither or both of `contains`/`regex`"),
    )
)]
pub async fn preview_uncategorized_rule(
    State(state): State<AppState>,
    Json(request): Json<PreviewRuleRequest>,
) -> Result<Json<PreviewRuleResponse>, ApiError> {
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

    Ok(Json(PreviewRuleResponse {
        count: count_matches(&queue, &rule),
    }))
}

/// Counts how many transactions in `queue` match `rule`. A transaction with
/// no import metadata never matches (there's nothing for the criteria to
/// check), mirroring [`build_head_response`]'s treatment of such
/// transactions.
fn count_matches(queue: &[Transaction], rule: &Rule) -> u32 {
    queue
        .iter()
        .filter(|transaction| {
            transaction
                .import_details
                .as_ref()
                .is_some_and(|details| match_rule(&MatchableImportDetails(details), rule))
        })
        .count() as u32
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
    path = "/api/v1/uncategorized-transactions/rules",
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

/// Appends `rule` to the current rules file and writes it back, so it's
/// persisted before any transaction gets re-categorized against it.
async fn persist_rule(deps: &ImportDeps, rule: Rule) -> Result<(), ApiError> {
    let mut rules = deps.current_rules()?;
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

    /// An empty queue reports zero remaining and no head.
    #[test]
    fn empty_queue_has_no_head() {
        let response = build_queue_response(vec![]);
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
        };
        let response = build_queue_response(vec![transaction_with_import_details(Some(details))]);

        assert_eq!(response.remaining, 1);
        let head = response.head.expect("head");
        let derived = head.derived_rule.expect("derived rule");
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].contains, Some("NETTO".to_string()));
    }

    /// A head transaction with no import details at all has no derived rule.
    #[test]
    fn head_without_import_details_has_no_derived_rule() {
        let response = build_queue_response(vec![transaction_with_import_details(None)]);

        let head = response.head.expect("head");
        assert!(head.derived_rule.is_none());
    }
}
