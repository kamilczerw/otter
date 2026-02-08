use serde::{Deserialize, Serialize};
use crate::types::BudgetMonth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month {
    pub id: ulid::Ulid,
    pub month: BudgetMonth,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct NewMonth {
    pub month: BudgetMonth,
}
