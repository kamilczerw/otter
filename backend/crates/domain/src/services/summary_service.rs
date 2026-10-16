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

            let has_transactions = self
                .entry_repo
                .transaction_count(&entry.id)
                .await
                .map_err(|e| {
                    MonthError::Repository(format!("Failed to count transactions: {}", e))
                })?
                > 0;

            // Income-only entries have transactions but no expense-side
            // "paid" total (sum_by_entry excludes positive amounts); exclude
            // them from the budget view entirely.
            if has_transactions && paid.value() == 0 {
                continue;
            }

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
    use async_trait::async_trait;
    use chrono::Utc;

    use crate::entities::{
        BudgetEntry, BudgetEntryWithCategory, CategorySummary, Month, NewBudgetEntry, NewMonth,
    };
    use crate::errors::EntryError;
    use crate::types::{BudgetMonth, CategoryName, DueDay};

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

    /// Fake `BudgetEntryRepository` returning a single, fixed entry so the
    /// summary pipeline can be exercised without a database.
    struct FakeEntryRepo {
        entry: BudgetEntryWithCategory,
        /// Transaction count reported for the fixture entry, controlling
        /// whether `get_month_summary` treats it as having transactions.
        transaction_count: i64,
    }

    #[async_trait]
    impl BudgetEntryRepository for FakeEntryRepo {
        async fn list_by_month(
            &self,
            _month_id: &Ulid,
        ) -> Result<Vec<BudgetEntryWithCategory>, EntryError> {
            Ok(vec![self.entry.clone()])
        }

        async fn find_by_id(&self, _id: &Ulid) -> Result<Option<BudgetEntry>, EntryError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn create(
            &self,
            _entry: NewBudgetEntry,
        ) -> Result<BudgetEntryWithCategory, EntryError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn update(
            &self,
            _id: &Ulid,
            _budgeted: Option<Money>,
            _due_day: Option<Option<DueDay>>,
        ) -> Result<BudgetEntryWithCategory, EntryError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn delete(&self, _id: &Ulid) -> Result<(), EntryError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn transaction_count(&self, _entry_id: &Ulid) -> Result<i64, EntryError> {
            Ok(self.transaction_count)
        }
    }

    /// Fake `TransactionRepository` whose `sum_by_entry` returns a fixed
    /// magnitude, standing in for the real `SUM(ABS(amount))` aggregation.
    struct FakeTransactionRepo {
        sum: Money,
    }

    #[async_trait]
    impl crate::ports::TransactionRepository for FakeTransactionRepo {
        async fn list_by_month(
            &self,
            _month_id: &Ulid,
        ) -> Result<Vec<crate::entities::Transaction>, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn find_by_id(
            &self,
            _id: &Ulid,
        ) -> Result<Option<crate::entities::Transaction>, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn create(
            &self,
            _transaction: crate::entities::NewTransaction,
        ) -> Result<crate::entities::Transaction, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn update(
            &self,
            _id: &Ulid,
            _entry_id: Option<Ulid>,
            _amount: Option<Money>,
            _date: Option<crate::types::TransactionDate>,
            _title: Option<Option<String>>,
        ) -> Result<crate::entities::Transaction, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn delete(&self, _id: &Ulid) -> Result<(), crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn sum_by_entry(
            &self,
            _entry_id: &Ulid,
        ) -> Result<Money, crate::errors::TransactionError> {
            Ok(self.sum)
        }

        async fn list_by_entry(
            &self,
            _entry_id: &Ulid,
            _limit: u32,
            _offset: u32,
            _kind: Option<crate::entities::TransactionKind>,
        ) -> Result<Vec<crate::entities::Transaction>, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn find_duplicate(
            &self,
            _entry_id: &Ulid,
            _date: &crate::types::TransactionDate,
            _amount: &Money,
            _title: Option<&str>,
        ) -> Result<Option<crate::entities::Transaction>, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn list_by_category_name(
            &self,
            _category_name: &str,
        ) -> Result<Vec<crate::entities::Transaction>, crate::errors::TransactionError> {
            unimplemented!("not needed for summary pipeline tests")
        }
    }

    /// Fake `MonthRepository` returning a single, fixed month.
    struct FakeMonthRepo {
        month: Month,
    }

    #[async_trait]
    impl MonthRepository for FakeMonthRepo {
        async fn list_all(&self) -> Result<Vec<Month>, MonthError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn find_by_id(&self, _id: &Ulid) -> Result<Option<Month>, MonthError> {
            Ok(Some(self.month.clone()))
        }

        async fn find_by_month(
            &self,
            _month: &BudgetMonth,
        ) -> Result<Option<Month>, MonthError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn create(&self, _month: NewMonth) -> Result<Month, MonthError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn find_latest(&self) -> Result<Option<Month>, MonthError> {
            unimplemented!("not needed for summary pipeline tests")
        }

        async fn find_latest_excluding(
            &self,
            _exclude_id: &Ulid,
        ) -> Result<Option<Month>, MonthError> {
            unimplemented!("not needed for summary pipeline tests")
        }
    }

    /// Builds a `BudgetEntryWithCategory` with the given budgeted amount,
    /// for use as a fixed fixture entry in pipeline tests.
    fn make_entry(budgeted: Money) -> BudgetEntryWithCategory {
        BudgetEntryWithCategory {
            id: Ulid::new(),
            category: CategorySummary {
                id: Ulid::new(),
                name: CategoryName::new("income/salary").unwrap(),
                label: None,
            },
            budgeted,
            due_day: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Builds a fixture `SummaryService` with a single fake entry, exercising
    /// the real `get_month_summary` pipeline without a database.
    fn make_service(
        entry: BudgetEntryWithCategory,
        transaction_count: i64,
        paid: Money,
    ) -> SummaryService {
        let month = Month {
            id: Ulid::new(),
            month: BudgetMonth::new(2026, 1).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        SummaryService::new(
            Arc::new(FakeEntryRepo {
                entry,
                transaction_count,
            }),
            Arc::new(FakeTransactionRepo { sum: paid }),
            Arc::new(FakeMonthRepo { month }),
        )
    }

    #[tokio::test]
    async fn income_only_entry_is_excluded_from_summary() {
        // Income category: transactions exist but `sum_by_entry` now
        // excludes positive amounts, so `paid` is zero. Such entries must be
        // dropped from the summary entirely, not shown as "Unpaid".
        let budgeted = Money::new(500_00);
        let entry = make_entry(budgeted);

        let service = make_service(entry, 1, Money::new(0));

        let summary = service.get_month_summary(&Ulid::new()).await.unwrap();

        assert!(summary.categories.is_empty());
        assert_eq!(summary.total_budgeted, Money::new(0));
        assert_eq!(summary.total_paid, Money::new(0));
    }

    #[tokio::test]
    async fn entry_with_zero_transactions_still_appears() {
        // No transactions recorded at all: paid is zero, but this is a
        // genuinely unpaid expense entry, not an excluded income entry.
        let budgeted = Money::new(500_00);
        let entry = make_entry(budgeted);

        let service = make_service(entry, 0, Money::new(0));

        let summary = service.get_month_summary(&Ulid::new()).await.unwrap();

        assert_eq!(summary.categories.len(), 1);
        assert_eq!(summary.categories[0].status, BudgetStatus::Unpaid);
        assert_eq!(summary.total_budgeted, Money::new(500_00));
    }

    #[tokio::test]
    async fn entry_with_mixed_expense_and_income_appears_with_expense_only_paid() {
        // Mixed entry: has transactions, and `paid` (expense-only, per
        // sum_by_entry) is non-zero, so it must appear normally.
        let budgeted = Money::new(500_00);
        let entry = make_entry(budgeted);
        let paid = Money::new(300_00);

        let service = make_service(entry, 2, paid);

        let summary = service.get_month_summary(&Ulid::new()).await.unwrap();

        assert_eq!(summary.categories.len(), 1);
        assert_eq!(summary.categories[0].paid, paid);
        assert_eq!(summary.total_paid, paid);
    }
}
