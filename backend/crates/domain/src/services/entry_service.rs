use std::sync::Arc;
use ulid::Ulid;

use crate::entities::{BudgetEntryWithCategory, NewBudgetEntry};
use crate::errors::EntryError;
use crate::ports::{BudgetEntryRepository, CategoryRepository, MonthRepository};
use crate::types::{DueDay, Money};

pub struct EntryService {
    entry_repo: Arc<dyn BudgetEntryRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    month_repo: Arc<dyn MonthRepository>,
}

impl EntryService {
    pub fn new(
        entry_repo: Arc<dyn BudgetEntryRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        month_repo: Arc<dyn MonthRepository>,
    ) -> Self {
        Self {
            entry_repo,
            category_repo,
            month_repo,
        }
    }

    pub async fn list_by_month(
        &self,
        month_id: &Ulid,
    ) -> Result<Vec<BudgetEntryWithCategory>, EntryError> {
        // Verify month exists
        self.month_repo
            .find_by_id(month_id)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?
            .ok_or(EntryError::MonthNotFound)?;

        self.entry_repo.list_by_month(month_id).await
    }

    pub async fn create(
        &self,
        month_id: Ulid,
        category_id: Ulid,
        budgeted: Money,
        due_day: Option<DueDay>,
    ) -> Result<BudgetEntryWithCategory, EntryError> {
        // Verify month exists
        self.month_repo
            .find_by_id(&month_id)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?
            .ok_or(EntryError::MonthNotFound)?;

        // Verify category exists
        self.category_repo
            .find_by_id(&category_id)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?
            .ok_or(EntryError::CategoryNotFound)?;

        let new_entry = NewBudgetEntry {
            month_id,
            category_id,
            budgeted,
            due_day,
        };

        self.entry_repo.create(new_entry).await
    }

    pub async fn update(
        &self,
        id: &Ulid,
        budgeted: Option<Money>,
        due_day: Option<Option<DueDay>>,
    ) -> Result<BudgetEntryWithCategory, EntryError> {
        self.entry_repo.update(id, budgeted, due_day).await
    }

    pub async fn delete(&self, id: &Ulid) -> Result<(), EntryError> {
        let count = self.entry_repo.transaction_count(id).await?;
        if count > 0 {
            return Err(EntryError::HasTransactions {
                transaction_count: count,
            });
        }
        self.entry_repo.delete(id).await
    }
}
