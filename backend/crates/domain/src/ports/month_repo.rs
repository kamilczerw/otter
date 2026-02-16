use async_trait::async_trait;
use crate::entities::{Month, NewMonth};
use crate::errors::MonthError;
use crate::types::BudgetMonth;

#[async_trait]
pub trait MonthRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<Month>, MonthError>;
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Month>, MonthError>;
    async fn find_by_month(&self, month: &BudgetMonth) -> Result<Option<Month>, MonthError>;
    async fn create(&self, month: NewMonth) -> Result<Month, MonthError>;
    async fn find_latest(&self) -> Result<Option<Month>, MonthError>;
    async fn find_latest_excluding(&self, exclude_id: &ulid::Ulid) -> Result<Option<Month>, MonthError>;
}
