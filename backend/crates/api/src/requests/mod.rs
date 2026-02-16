use serde::Deserialize;
use utoipa::ToSchema;

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
