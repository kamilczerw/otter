use serde::{Deserialize, Serialize};
use crate::types::{Money, TransactionDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: ulid::Ulid,
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
}
