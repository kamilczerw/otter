use async_trait::async_trait;
use crate::entities::{Transaction, NewTransaction};
use crate::errors::TransactionError;
use crate::types::{Money, TransactionDate};

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn list_by_month(&self, month_id: &ulid::Ulid) -> Result<Vec<Transaction>, TransactionError>;
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Transaction>, TransactionError>;
    async fn create(&self, transaction: NewTransaction) -> Result<Transaction, TransactionError>;
    async fn update(&self, id: &ulid::Ulid, entry_id: Option<ulid::Ulid>, amount: Option<Money>, date: Option<TransactionDate>) -> Result<Transaction, TransactionError>;
    async fn delete(&self, id: &ulid::Ulid) -> Result<(), TransactionError>;
    async fn sum_by_entry(&self, entry_id: &ulid::Ulid) -> Result<Money, TransactionError>;
    async fn list_by_entry(&self, entry_id: &ulid::Ulid, limit: u32, offset: u32) -> Result<Vec<Transaction>, TransactionError>;
}
