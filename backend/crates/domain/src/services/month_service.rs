use std::sync::Arc;
use ulid::Ulid;

use crate::entities::{Month, NewMonth, NewBudgetEntry};
use crate::errors::MonthError;
use crate::ports::{MonthRepository, BudgetEntryRepository};
use crate::types::BudgetMonth;

pub struct MonthService {
    month_repo: Arc<dyn MonthRepository>,
    entry_repo: Arc<dyn BudgetEntryRepository>,
}

impl MonthService {
    pub fn new(
        month_repo: Arc<dyn MonthRepository>,
        entry_repo: Arc<dyn BudgetEntryRepository>,
    ) -> Self {
        Self {
            month_repo,
            entry_repo,
        }
    }

    pub async fn list_all(&self) -> Result<Vec<Month>, MonthError> {
        self.month_repo.list_all().await
    }

    pub async fn find_by_id(&self, id: &Ulid) -> Result<Month, MonthError> {
        self.month_repo
            .find_by_id(id)
            .await?
            .ok_or(MonthError::NotFound)
    }

    pub async fn create(
        &self,
        month: BudgetMonth,
        copy_from: Option<&Ulid>,
        empty: bool,
    ) -> Result<Month, MonthError> {
        let new_month = NewMonth { month };
        let created = self.month_repo.create(new_month).await?;

        // If empty is true, don't copy any entries
        if empty {
            return Ok(created);
        }

        // Determine source month for copying entries
        let source_month_id = if let Some(source_id) = copy_from {
            // Verify the source month exists
            self.month_repo
                .find_by_id(source_id)
                .await?
                .ok_or(MonthError::NotFound)?;
            Some(*source_id)
        } else {
            // Find the latest existing month (excluding the one we just created)
            self.month_repo
                .find_latest()
                .await?
                .filter(|m| m.id != created.id)
                .map(|m| m.id)
        };

        // Copy entries from source month if one exists
        if let Some(source_id) = source_month_id {
            let entries = self.entry_repo.list_by_month(&source_id).await.map_err(|e| {
                MonthError::Repository(format!("Failed to list entries for copy: {}", e))
            })?;

            for entry in entries {
                let new_entry = NewBudgetEntry {
                    month_id: created.id,
                    category_id: entry.category.id,
                    budgeted: entry.budgeted,
                    due_day: entry.due_day,
                };
                self.entry_repo.create(new_entry).await.map_err(|e| {
                    MonthError::Repository(format!("Failed to copy entry: {}", e))
                })?;
            }
        }

        Ok(created)
    }
}
