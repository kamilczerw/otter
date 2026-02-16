use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::str::FromStr;

use domain::entities::{NewTransaction, Transaction};
use domain::errors::TransactionError;
use domain::ports::TransactionRepository;
use domain::types::{Money, TransactionDate};

pub struct SqliteTransactionRepository {
    pool: SqlitePool,
}

impl SqliteTransactionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn map_row_to_transaction(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<Transaction, TransactionError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| TransactionError::Repository(format!("invalid ULID: {}", e)))?;

    let entry_id_str: String = row.get("entry_id");
    let entry_id = ulid::Ulid::from_string(&entry_id_str)
        .map_err(|e| TransactionError::Repository(format!("invalid entry_id ULID: {}", e)))?;

    let amount: i64 = row.get("amount");

    let date_str: String = row.get("date");
    let date = TransactionDate::from_str(&date_str)
        .map_err(|e| TransactionError::Repository(format!("invalid date: {}", e)))?;

    let title: Option<String> = row.get("title");

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| TransactionError::Repository(format!("invalid created_at: {}", e)))?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| TransactionError::Repository(format!("invalid updated_at: {}", e)))?;

    Ok(Transaction {
        id,
        entry_id,
        amount: Money::new(amount),
        date,
        title,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl TransactionRepository for SqliteTransactionRepository {
    async fn list_by_month(
        &self,
        month_id: &ulid::Ulid,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let rows = sqlx::query(
            "SELECT t.id, t.entry_id, t.amount, t.date, t.title, t.created_at, t.updated_at \
             FROM transactions t \
             JOIN budget_entries e ON t.entry_id = e.id \
             WHERE e.month_id = ? \
             ORDER BY t.date DESC, t.created_at DESC",
        )
        .bind(month_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TransactionError::Repository(e.to_string()))?;

        rows.iter()
            .map(map_row_to_transaction)
            .collect()
    }

    async fn find_by_id(
        &self,
        id: &ulid::Ulid,
    ) -> Result<Option<Transaction>, TransactionError> {
        let row = sqlx::query("SELECT * FROM transactions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TransactionError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_transaction(r)?)),
            None => Ok(None),
        }
    }

    async fn create(
        &self,
        transaction: NewTransaction,
    ) -> Result<Transaction, TransactionError> {
        let id = ulid::Ulid::new();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let result = sqlx::query(
            "INSERT INTO transactions (id, entry_id, amount, date, title, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(transaction.entry_id.to_string())
        .bind(transaction.amount.value())
        .bind(transaction.date.to_string())
        .bind(&transaction.title)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("FOREIGN KEY constraint failed") =>
            {
                return Err(TransactionError::EntryNotFound);
            }
            Err(e) => return Err(TransactionError::Repository(e.to_string())),
        }

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| {
                TransactionError::Repository("failed to fetch created transaction".to_string())
            })
    }

    async fn update(
        &self,
        id: &ulid::Ulid,
        entry_id: Option<ulid::Ulid>,
        amount: Option<Money>,
        date: Option<TransactionDate>,
        title: Option<Option<String>>,
    ) -> Result<Transaction, TransactionError> {
        let mut set_clauses: Vec<String> = Vec::new();

        if entry_id.is_some() {
            set_clauses.push("entry_id = ?".to_string());
        }
        if amount.is_some() {
            set_clauses.push("amount = ?".to_string());
        }
        if date.is_some() {
            set_clauses.push("date = ?".to_string());
        }
        if title.is_some() {
            set_clauses.push("title = ?".to_string());
        }

        if set_clauses.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or(TransactionError::NotFound);
        }

        let sql = format!(
            "UPDATE transactions SET {} WHERE id = ?",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&sql);

        if let Some(ref eid) = entry_id {
            query = query.bind(eid.to_string());
        }
        if let Some(ref a) = amount {
            query = query.bind(a.value());
        }
        if let Some(ref d) = date {
            query = query.bind(d.to_string());
        }
        if let Some(ref t) = title {
            query = query.bind(t);
        }

        query = query.bind(id.to_string());

        let result = query.execute(&self.pool).await;

        match result {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    return Err(TransactionError::NotFound);
                }
            }
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("FOREIGN KEY constraint failed") =>
            {
                return Err(TransactionError::EntryNotFound);
            }
            Err(e) => return Err(TransactionError::Repository(e.to_string())),
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| {
                TransactionError::Repository("failed to fetch updated transaction".to_string())
            })
    }

    async fn delete(&self, id: &ulid::Ulid) -> Result<(), TransactionError> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| TransactionError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TransactionError::NotFound);
        }

        Ok(())
    }

    async fn list_by_entry(
        &self,
        entry_id: &ulid::Ulid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let rows = sqlx::query(
            "SELECT id, entry_id, amount, date, title, created_at, updated_at \
             FROM transactions \
             WHERE entry_id = ? \
             ORDER BY date DESC, created_at DESC \
             LIMIT ? OFFSET ?",
        )
        .bind(entry_id.to_string())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TransactionError::Repository(e.to_string()))?;

        rows.iter()
            .map(map_row_to_transaction)
            .collect()
    }

    async fn sum_by_entry(
        &self,
        entry_id: &ulid::Ulid,
    ) -> Result<Money, TransactionError> {
        let row =
            sqlx::query("SELECT COALESCE(SUM(amount), 0) AS total FROM transactions WHERE entry_id = ?")
                .bind(entry_id.to_string())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| TransactionError::Repository(e.to_string()))?;

        let total: i64 = row.get("total");
        Ok(Money::new(total))
    }
}
