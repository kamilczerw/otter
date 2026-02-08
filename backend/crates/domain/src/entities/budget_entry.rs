use serde::{Deserialize, Serialize};
use crate::types::{Money, DueDay, CategoryName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEntry {
    pub id: ulid::Ulid,
    pub month_id: ulid::Ulid,
    pub category_id: ulid::Ulid,
    pub budgeted: Money,
    pub due_day: Option<DueDay>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Budget entry with inlined category info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEntryWithCategory {
    pub id: ulid::Ulid,
    pub category: CategorySummary,
    pub budgeted: Money,
    pub due_day: Option<DueDay>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Minimal category info for embedding in entries/summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub id: ulid::Ulid,
    pub name: CategoryName,
}

#[derive(Debug, Clone)]
pub struct NewBudgetEntry {
    pub month_id: ulid::Ulid,
    pub category_id: ulid::Ulid,
    pub budgeted: Money,
    pub due_day: Option<DueDay>,
}
