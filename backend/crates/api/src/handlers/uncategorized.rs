//! HTTP handler for `GET /api/v1/uncategorized-transactions`: reports how
//! many transactions are currently sitting in the "uncategorized" category
//! and the one at the head of that queue, so the bulk-categorize UI (section
//! 3 of the `bulk-categorize-transactions` change) has something to show and
//! act on. This handler only exposes the queue's head; "preview" (3.2) and
//! "save rule" (3.3) are separate, later endpoints.

use axum::extract::State;
use axum::Json;

use domain::entities::Transaction;

use crate::errors::ApiError;
use crate::import::{MatchableImportDetails, UNCATEGORIZED};
use crate::responses::{MatchCriterionResponse, TransactionResponse, UncategorizedHeadResponse, UncategorizedQueueResponse};

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

/// Derives the starting rule's match criteria from `import_details`, via
/// [`csv_import::derive_rule`]. `derive_rule`'s `category` argument is
/// unused by matching, so an empty placeholder is passed and only the
/// derived `pattern`'s criteria are ever exposed to the client.
fn derive_rule_criteria(import_details: &domain::entities::ImportDetails) -> Vec<MatchCriterionResponse> {
    let rule = csv_import::derive_rule(&MatchableImportDetails(import_details), &String::new());
    rule.pattern.0.into_iter().map(MatchCriterionResponse::from).collect()
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
