use std::sync::Arc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::entities::CategorySummary;
use crate::errors::MonthError;
use crate::ports::{BudgetEntryRepository, MonthRepository, TransactionRepository};
use crate::types::{BudgetMonth, Money};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthSummary {
    pub month: BudgetMonth,
    pub total_budgeted: Money,
    pub total_paid: Money,
    pub remaining: Money,
    pub categories: Vec<CategoryBudgetSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBudgetSummary {
    pub entry_id: ulid::Ulid,
    pub category: CategorySummary,
    pub budgeted: Money,
    pub paid: Money,
    pub remaining: Money,
    pub status: BudgetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BudgetStatus {
    Unpaid,
    Underspent,
    OnBudget,
    Overspent,
}

fn derive_status(budgeted: Money, paid: Money) -> BudgetStatus {
    let b = budgeted.value();
    let p = paid.value();

    if b == 0 && p == 0 {
        BudgetStatus::OnBudget
    } else if p == 0 && b > 0 {
        BudgetStatus::Unpaid
    } else if p > 0 && p < b {
        BudgetStatus::Underspent
    } else if p == b {
        BudgetStatus::OnBudget
    } else {
        // p > b, includes zero-budget categories with any payment
        BudgetStatus::Overspent
    }
}

pub struct SummaryService {
    entry_repo: Arc<dyn BudgetEntryRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    month_repo: Arc<dyn MonthRepository>,
}

impl SummaryService {
    pub fn new(
        entry_repo: Arc<dyn BudgetEntryRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        month_repo: Arc<dyn MonthRepository>,
    ) -> Self {
        Self {
            entry_repo,
            transaction_repo,
            month_repo,
        }
    }

    pub async fn get_month_summary(&self, month_id: &Ulid) -> Result<MonthSummary, MonthError> {
        let month = self
            .month_repo
            .find_by_id(month_id)
            .await?
            .ok_or(MonthError::NotFound)?;

        let entries = self
            .entry_repo
            .list_by_month(month_id)
            .await
            .map_err(|e| MonthError::Repository(format!("Failed to list entries: {}", e)))?;

        let mut categories = Vec::with_capacity(entries.len());
        let mut total_budgeted = Money::new(0);
        let mut total_paid = Money::new(0);

        for entry in entries {
            let paid = self
                .transaction_repo
                .sum_by_entry(&entry.id)
                .await
                .map_err(|e| {
                    MonthError::Repository(format!("Failed to sum transactions: {}", e))
                })?;

            let remaining = entry.budgeted - paid;
            let status = derive_status(entry.budgeted, paid);

            total_budgeted = total_budgeted + entry.budgeted;
            total_paid = total_paid + paid;

            categories.push(CategoryBudgetSummary {
                entry_id: entry.id,
                category: entry.category,
                budgeted: entry.budgeted,
                paid,
                remaining,
                status,
            });
        }

        let remaining = total_budgeted - total_paid;

        Ok(MonthSummary {
            month: month.month,
            total_budgeted,
            total_paid,
            remaining,
            categories,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(budgeted: i64, paid: i64) -> BudgetStatus {
        derive_status(Money::new(budgeted), Money::new(paid))
    }

    #[test]
    fn test_unpaid() {
        // paid == 0, budgeted > 0 -> Unpaid
        assert_eq!(status(1000, 0), BudgetStatus::Unpaid);
    }

    #[test]
    fn test_underspent() {
        // paid > 0, paid < budgeted -> Underspent
        assert_eq!(status(1000, 500), BudgetStatus::Underspent);
    }

    #[test]
    fn test_on_budget() {
        // paid == budgeted -> OnBudget
        assert_eq!(status(1000, 1000), BudgetStatus::OnBudget);
    }

    #[test]
    fn test_overspent() {
        // paid > budgeted -> Overspent
        assert_eq!(status(1000, 1500), BudgetStatus::Overspent);
    }

    #[test]
    fn test_zero_budget_zero_paid() {
        // budgeted == 0, paid == 0 -> OnBudget
        assert_eq!(status(0, 0), BudgetStatus::OnBudget);
    }

    #[test]
    fn test_zero_budget_with_payment() {
        // budgeted == 0, paid > 0 -> Overspent
        assert_eq!(status(0, 500), BudgetStatus::Overspent);
    }
}
