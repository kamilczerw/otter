use async_trait::async_trait;
use crate::entities::{BudgetEntry, BudgetEntryWithCategory, NewBudgetEntry};
use crate::errors::EntryError;
use crate::types::{Money, DueDay};

#[async_trait]
pub trait BudgetEntryRepository: Send + Sync {
    async fn list_by_month(&self, month_id: &ulid::Ulid) -> Result<Vec<BudgetEntryWithCategory>, EntryError>;
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<BudgetEntry>, EntryError>;
    async fn create(&self, entry: NewBudgetEntry) -> Result<BudgetEntryWithCategory, EntryError>;
    async fn update(&self, id: &ulid::Ulid, budgeted: Option<Money>, due_day: Option<Option<DueDay>>) -> Result<BudgetEntryWithCategory, EntryError>;
    async fn delete(&self, id: &ulid::Ulid) -> Result<(), EntryError>;
    async fn transaction_count(&self, entry_id: &ulid::Ulid) -> Result<i64, EntryError>;
}
