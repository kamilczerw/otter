use serde::Serialize;
use utoipa::ToSchema;

use domain::entities::{
    BudgetEntryWithCategory, Category, CategorySummary as DomainCategorySummary, Month,
    Transaction, TransactionKind,
};

/// Maps a transaction's derived direction to its API string representation.
fn transaction_kind_str(kind: TransactionKind) -> &'static str {
    match kind {
        TransactionKind::Expense => "expense",
        TransactionKind::Income => "income",
    }
}
use domain::services::{BudgetStatus, CategoryBudgetSummary, MonthSummary};

use crate::import::{ImportOutcome, RowFailure};

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
    pub created_at: String, // RFC 3339
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthResponse {
    pub id: String,
    pub month: String, // "YYYY-MM"
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategorySummaryResponse {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntryResponse {
    pub id: String,
    pub category: CategorySummaryResponse,
    pub budgeted: i64,
    pub due_day: Option<u8>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub id: String,
    pub entry_id: String,
    pub amount: i64,
    /// Direction derived from `amount`'s sign: `"expense"` or `"income"`.
    pub kind: String,
    pub date: String, // "YYYY-MM-DD"
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthSummaryResponse {
    pub month: String,
    pub total_budgeted: i64,
    pub total_paid: i64,
    pub remaining: i64,
    pub categories: Vec<CategoryBudgetSummaryResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryBudgetSummaryResponse {
    pub entry_id: String,
    pub category: CategorySummaryResponse,
    pub budgeted: i64,
    pub paid: i64,
    pub remaining: i64,
    pub status: String, // "unpaid", "underspent", "on_budget", "overspent"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedTransactionsResponse {
    pub items: Vec<TransactionResponse>,
    pub has_more: bool,
}

/// One CSV import row that failed extraction or processing, as reported to
/// API clients: its position in the file, its best-effort raw content, and
/// why it failed.
#[derive(Debug, Serialize, ToSchema)]
pub struct FailedRowResponse {
    pub row: usize,
    pub raw: String,
    pub reason: String,
}

/// Result of a CSV import request that wasn't rejected wholesale: counts of
/// imported/duplicate/uncategorized transactions, names of any newly
/// created months/categories, and the itemized list of rows that failed.
#[derive(Debug, Serialize, ToSchema)]
pub struct ImportResponse {
    pub imported: u32,
    pub duplicates: u32,
    pub uncategorized: u32,
    pub created_months: Vec<String>,
    pub created_categories: Vec<String>,
    pub failed: Vec<FailedRowResponse>,
}

// --- From impls ---

impl From<RowFailure> for FailedRowResponse {
    fn from(f: RowFailure) -> Self {
        Self {
            row: f.row,
            raw: f.raw,
            reason: f.reason,
        }
    }
}

impl From<ImportOutcome> for ImportResponse {
    fn from(outcome: ImportOutcome) -> Self {
        Self {
            imported: outcome.imported,
            duplicates: outcome.duplicates,
            uncategorized: outcome.uncategorized,
            created_months: outcome.created_months,
            created_categories: outcome.created_categories,
            failed: outcome.failed.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name.as_str().to_string(),
            label: c.label,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

impl From<Month> for MonthResponse {
    fn from(m: Month) -> Self {
        Self {
            id: m.id.to_string(),
            month: m.month.to_string(),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

impl From<DomainCategorySummary> for CategorySummaryResponse {
    fn from(cs: DomainCategorySummary) -> Self {
        Self {
            id: cs.id.to_string(),
            name: cs.name.as_str().to_string(),
            label: cs.label,
        }
    }
}

impl From<BudgetEntryWithCategory> for EntryResponse {
    fn from(e: BudgetEntryWithCategory) -> Self {
        Self {
            id: e.id.to_string(),
            category: CategorySummaryResponse::from(e.category),
            budgeted: e.budgeted.value(),
            due_day: e.due_day.map(|d| d.value()),
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id.to_string(),
            entry_id: t.entry_id.to_string(),
            amount: t.amount.value(),
            kind: transaction_kind_str(t.kind()).to_string(),
            date: t.date.to_string(),
            title: t.title,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

impl From<MonthSummary> for MonthSummaryResponse {
    fn from(s: MonthSummary) -> Self {
        Self {
            month: s.month.to_string(),
            total_budgeted: s.total_budgeted.value(),
            total_paid: s.total_paid.value(),
            remaining: s.remaining.value(),
            categories: s.categories.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<CategoryBudgetSummary> for CategoryBudgetSummaryResponse {
    fn from(c: CategoryBudgetSummary) -> Self {
        let status = match c.status {
            BudgetStatus::Unpaid => "unpaid",
            BudgetStatus::Underspent => "underspent",
            BudgetStatus::OnBudget => "on_budget",
            BudgetStatus::Overspent => "overspent",
        };
        Self {
            entry_id: c.entry_id.to_string(),
            category: CategorySummaryResponse::from(c.category),
            budgeted: c.budgeted.value(),
            paid: c.paid.value(),
            remaining: c.remaining.value(),
            status: status.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::types::{Money, TransactionDate};
    use chrono::NaiveDate;

    /// Builds a `Transaction` with the given amount, leaving all other
    /// fields as arbitrary fixed values irrelevant to `kind` serialization.
    fn transaction_with_amount(amount: i64) -> Transaction {
        let now = chrono::Utc::now();
        Transaction {
            id: ulid::Ulid::new(),
            entry_id: ulid::Ulid::new(),
            amount: Money::new(amount),
            date: TransactionDate::new(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            title: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn negative_amount_serializes_kind_as_expense() {
        let response = TransactionResponse::from(transaction_with_amount(-500));
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["kind"], "expense");
    }

    #[test]
    fn positive_amount_serializes_kind_as_income() {
        let response = TransactionResponse::from(transaction_with_amount(500));
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["kind"], "income");
    }
}
