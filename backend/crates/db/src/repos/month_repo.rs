use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::str::FromStr;

use domain::entities::{Month, NewMonth};
use domain::errors::MonthError;
use domain::ports::MonthRepository;
use domain::types::BudgetMonth;

pub struct SqliteMonthRepository {
    pool: SqlitePool,
}

impl SqliteMonthRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn map_row_to_month(row: &sqlx::sqlite::SqliteRow) -> Result<Month, MonthError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| MonthError::Repository(format!("invalid ULID: {}", e)))?;

    let month_str: String = row.get("month");
    let month = BudgetMonth::from_str(&month_str)
        .map_err(|e| MonthError::Repository(format!("invalid budget month '{}': {}", month_str, e)))?;

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MonthError::Repository(format!("invalid created_at: {}", e)))?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MonthError::Repository(format!("invalid updated_at: {}", e)))?;

    Ok(Month {
        id,
        month,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl MonthRepository for SqliteMonthRepository {
    async fn list_all(&self) -> Result<Vec<Month>, MonthError> {
        let rows = sqlx::query("SELECT * FROM months ORDER BY month DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MonthError::Repository(e.to_string()))?;

        rows.iter()
            .map(map_row_to_month)
            .collect()
    }

    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Month>, MonthError> {
        let row = sqlx::query("SELECT * FROM months WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MonthError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_month(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_month(&self, month: &BudgetMonth) -> Result<Option<Month>, MonthError> {
        let row = sqlx::query("SELECT * FROM months WHERE month = ?")
            .bind(month.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MonthError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_month(r)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, month: NewMonth) -> Result<Month, MonthError> {
        let id = ulid::Ulid::new();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let result = sqlx::query(
            "INSERT INTO months (id, month, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(month.month.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                return Err(MonthError::AlreadyExists {
                    month: month.month.to_string(),
                });
            }
            Err(e) => return Err(MonthError::Repository(e.to_string())),
        }

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| MonthError::Repository("failed to fetch created month".to_string()))
    }

    async fn find_latest(&self) -> Result<Option<Month>, MonthError> {
        let row = sqlx::query("SELECT * FROM months ORDER BY month DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MonthError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_month(r)?)),
            None => Ok(None),
        }
    }
}
