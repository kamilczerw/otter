use serde::{Deserialize, Serialize};
use crate::types::CategoryName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: ulid::Ulid,
    pub name: CategoryName,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct NewCategory {
    pub name: CategoryName,
}
