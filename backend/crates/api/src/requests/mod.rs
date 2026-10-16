use serde::Deserialize;
use utoipa::ToSchema;

use csv_import::{MatchCriteria, MatchCriterion, MatchField, MatchType};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCategoryRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, with = "double_option")]
    pub label: Option<Option<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMonthRequest {
    pub month: String, // "YYYY-MM"
    #[serde(default)]
    pub copy_from: Option<String>, // ULID of source month to copy entries from
    #[serde(default)]
    pub empty: Option<bool>, // Create month with no entries
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEntryRequest {
    pub category_id: String,
    pub budgeted: i64,
    pub due_day: Option<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEntryRequest {
    #[serde(default)]
    pub budgeted: Option<i64>,
    #[serde(default, with = "double_option")]
    pub due_day: Option<Option<u8>>, // None = don't change, Some(None) = clear, Some(Some(v)) = set
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub entry_id: String,
    pub amount: i64,
    pub date: String, // "YYYY-MM-DD"
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTransactionRequest {
    #[serde(default)]
    pub entry_id: Option<String>,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub date: Option<String>, // "YYYY-MM-DD"
    #[serde(default, with = "double_option")]
    pub title: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionListQuery {
    pub month: Option<String>, // ULID of month
    pub entry_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    /// Optional direction filter: `"expense"` or `"income"`. Any other
    /// non-empty value is a `400 TRANSACTION_INVALID_KIND`.
    pub kind: Option<String>,
}

/// Wire-format mirror of [`MatchField`] for request bodies (the reverse
/// direction of `responses::MatchFieldResponse`): deserializes the same
/// snake_case strings so a draft rule built client-side round-trips through
/// the same field names the API already reports.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchFieldRequest {
    Merchant,
    RawType,
    City,
    Country,
    Description,
    /// The transaction's joined, `" | "`-separated unmapped trailing CSV
    /// columns, if any.
    RawFragments,
}

impl From<MatchFieldRequest> for MatchField {
    fn from(field: MatchFieldRequest) -> Self {
        match field {
            MatchFieldRequest::Merchant => Self::Merchant,
            MatchFieldRequest::RawType => Self::RawType,
            MatchFieldRequest::City => Self::City,
            MatchFieldRequest::Country => Self::Country,
            MatchFieldRequest::Description => Self::Description,
            MatchFieldRequest::RawFragments => Self::RawFragments,
        }
    }
}

/// Wire-format mirror of a [`MatchCriterion`] for request bodies: a flat
/// `{ field, contains, regex }` shape, structurally the reverse of
/// `responses::MatchCriterionResponse`. Exactly one of `contains`/`regex`
/// must be set — this is a client-supplied draft rule crossing a trust
/// boundary, so an invalid combination (both or neither set) is rejected via
/// [`TryFrom`] rather than silently defaulting to one side.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MatchCriterionRequest {
    pub field: MatchFieldRequest,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
}

/// Error converting a [`MatchCriterionRequest`] into a [`MatchCriterion`]:
/// the client didn't set exactly one of `contains`/`regex`.
#[derive(Debug, thiserror::Error)]
#[error("exactly one of `contains` or `regex` must be set")]
pub struct InvalidMatchCriterionRequest;

impl TryFrom<MatchCriterionRequest> for MatchCriterion {
    type Error = InvalidMatchCriterionRequest;

    fn try_from(request: MatchCriterionRequest) -> Result<Self, Self::Error> {
        let match_type = match (request.contains, request.regex) {
            (Some(pattern), None) => MatchType::Contains(pattern),
            (None, Some(pattern)) => MatchType::Regex(pattern),
            _ => return Err(InvalidMatchCriterionRequest),
        };
        Ok(MatchCriterion {
            field: request.field.into(),
            match_type,
        })
    }
}

/// A requested slice of a larger result set: skip `offset` items, then take
/// at most `limit`. Used by [`PreviewRuleRequest`] to page through a draft
/// rule's actual matches instead of returning the whole queue at once.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PageRequest {
    pub offset: u32,
    pub limit: u32,
}

/// Request body for `POST /api/v1/transactions-review/preview`: a
/// draft rule's match criteria, ANDed together exactly like a saved
/// [`csv_import::Rule`]'s `match` list, so the UI can preview how many
/// uncategorized transactions it would match before saving it.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewRuleRequest {
    pub criteria: Vec<MatchCriterionRequest>,
    /// A transaction id to omit from the paged "other matches" view and its
    /// count — e.g. the head transaction, since it's already shown
    /// elsewhere in the review UI and showing it again in the match table
    /// would be redundant. Never affects the overall `count`. Absent (or
    /// `None`) excludes nothing.
    #[serde(default)]
    pub exclude_transaction_id: Option<String>,
    /// When present, requests a page of the rule's actual matches (not just
    /// their count) alongside the response's existing `count`. Absent means
    /// the caller only wants the count, preserving today's response shape
    /// exactly.
    #[serde(default)]
    pub page: Option<PageRequest>,
}

impl TryFrom<PreviewRuleRequest> for MatchCriteria {
    type Error = InvalidMatchCriterionRequest;

    fn try_from(request: PreviewRuleRequest) -> Result<Self, Self::Error> {
        criteria_from_requests(request.criteria)
    }
}

/// Request body for `POST /api/v1/transactions-review/rules`: a new
/// rule to persist, built from the same draft-rule criteria shape as
/// [`PreviewRuleRequest`] plus the category to assign on a match.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveRuleRequest {
    pub criteria: Vec<MatchCriterionRequest>,
    pub category: String,
}

impl TryFrom<SaveRuleRequest> for MatchCriteria {
    type Error = InvalidMatchCriterionRequest;

    fn try_from(request: SaveRuleRequest) -> Result<Self, Self::Error> {
        criteria_from_requests(request.criteria)
    }
}

/// Shared conversion step behind both [`PreviewRuleRequest`] and
/// [`SaveRuleRequest`]'s `TryFrom<_> for MatchCriteria` impls, so the
/// per-criterion conversion logic lives in exactly one place.
fn criteria_from_requests(
    criteria: Vec<MatchCriterionRequest>,
) -> Result<MatchCriteria, InvalidMatchCriterionRequest> {
    let criteria = criteria
        .into_iter()
        .map(MatchCriterion::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MatchCriteria(criteria))
}

/// Custom serde module for handling `Option<Option<T>>` fields correctly.
///
/// This allows distinguishing between three states in JSON:
/// - Field missing: `None` (don't update)
/// - Field is `null`: `Some(None)` (clear the field)
/// - Field has value: `Some(Some(value))` (set to value)
mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A preview request with only `criteria` (no `exclude_transaction_id`
    /// or `page`) deserializes with both new fields defaulting to `None`,
    /// so existing clients that predate pagination keep working unchanged.
    #[test]
    fn preview_request_without_paging_fields_deserializes() {
        let json = r#"{ "criteria": [] }"#;
        let request: PreviewRuleRequest = serde_json::from_str(json).unwrap();

        assert!(request.exclude_transaction_id.is_none());
        assert!(request.page.is_none());
    }

    /// A preview request with `exclude_transaction_id` and a `page`
    /// deserializes both into their respective fields.
    #[test]
    fn preview_request_with_paging_fields_deserializes() {
        let json = r#"{
            "criteria": [],
            "exclude_transaction_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "page": { "offset": 10, "limit": 20 }
        }"#;
        let request: PreviewRuleRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            request.exclude_transaction_id,
            Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string())
        );
        let page = request.page.unwrap();
        assert_eq!(page.offset, 10);
        assert_eq!(page.limit, 20);
    }
}
