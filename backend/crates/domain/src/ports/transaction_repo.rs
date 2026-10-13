use async_trait::async_trait;
use crate::entities::{Transaction, NewTransaction};
use crate::errors::TransactionError;
use crate::types::{Money, TransactionDate};

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn list_by_month(&self, month_id: &ulid::Ulid) -> Result<Vec<Transaction>, TransactionError>;
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Transaction>, TransactionError>;
    async fn create(&self, transaction: NewTransaction) -> Result<Transaction, TransactionError>;
    async fn update(&self, id: &ulid::Ulid, entry_id: Option<ulid::Ulid>, amount: Option<Money>, date: Option<TransactionDate>, title: Option<Option<String>>) -> Result<Transaction, TransactionError>;
    async fn delete(&self, id: &ulid::Ulid) -> Result<(), TransactionError>;
    async fn sum_by_entry(&self, entry_id: &ulid::Ulid) -> Result<Money, TransactionError>;
    async fn list_by_entry(&self, entry_id: &ulid::Ulid, limit: u32, offset: u32) -> Result<Vec<Transaction>, TransactionError>;

    /// Looks up an existing transaction matching `(entry_id, date, amount, title)`.
    ///
    /// Used by CSV import to skip rows that were already imported: a transaction
    /// is considered a duplicate of another if all four fields match exactly
    /// (title included, so an untitled transaction only matches other untitled
    /// transactions on the same entry/date/amount).
    ///
    /// # Errors
    ///
    /// Returns `TransactionError::Repository` if the underlying query fails.
    async fn find_duplicate(
        &self,
        entry_id: &ulid::Ulid,
        date: &TransactionDate,
        amount: &Money,
        title: Option<&str>,
    ) -> Result<Option<Transaction>, TransactionError>;

    /// Lists every transaction currently assigned (via its budget entry) to
    /// the category named `category_name`, across all months.
    ///
    /// Used by the bulk-categorize feature to find every transaction still
    /// sitting in `"uncategorized"`, regardless of which month it landed
    /// in — a single query, rather than the API layer looping over
    /// `list_by_month` for every month.
    ///
    /// # Errors
    ///
    /// Returns `TransactionError::Repository` if the underlying query fails.
    async fn list_by_category_name(
        &self,
        category_name: &str,
    ) -> Result<Vec<Transaction>, TransactionError>;
}
