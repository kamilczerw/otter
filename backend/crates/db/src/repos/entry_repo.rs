use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use domain::entities::{BudgetEntry, BudgetEntryWithCategory, CategorySummary, NewBudgetEntry};
use domain::errors::EntryError;
use domain::ports::BudgetEntryRepository;
use domain::types::{CategoryName, DueDay, Money};

pub struct SqliteBudgetEntryRepository {
    pool: SqlitePool,
}

impl SqliteBudgetEntryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn map_row_to_entry(row: &sqlx::sqlite::SqliteRow) -> Result<BudgetEntry, EntryError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| EntryError::Repository(format!("invalid ULID: {}", e)))?;

    let month_id_str: String = row.get("month_id");
    let month_id = ulid::Ulid::from_string(&month_id_str)
        .map_err(|e| EntryError::Repository(format!("invalid month_id ULID: {}", e)))?;

    let category_id_str: String = row.get("category_id");
    let category_id = ulid::Ulid::from_string(&category_id_str)
        .map_err(|e| EntryError::Repository(format!("invalid category_id ULID: {}", e)))?;

    let budgeted: i64 = row.get("budgeted");

    let due_day_raw: Option<i32> = row.get("due_day");
    let due_day = match due_day_raw {
        Some(d) => Some(
            DueDay::new(d as u8)
                .map_err(|e| EntryError::Repository(format!("invalid due_day: {}", e)))?,
        ),
        None => None,
    };

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| EntryError::Repository(format!("invalid created_at: {}", e)))?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| EntryError::Repository(format!("invalid updated_at: {}", e)))?;

    Ok(BudgetEntry {
        id,
        month_id,
        category_id,
        budgeted: Money::new(budgeted),
        due_day,
        created_at,
        updated_at,
    })
}

fn map_row_to_entry_with_category(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<BudgetEntryWithCategory, EntryError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| EntryError::Repository(format!("invalid ULID: {}", e)))?;

    let category_id_str: String = row.get("category_id");
    let category_id = ulid::Ulid::from_string(&category_id_str)
        .map_err(|e| EntryError::Repository(format!("invalid category_id ULID: {}", e)))?;

    let category_name_str: String = row.get("category_name");
    let category_name = CategoryName::new(category_name_str.clone())
        .map_err(|e| EntryError::Repository(format!("invalid category name: {}", e)))?;

    let budgeted: i64 = row.get("budgeted");

    let due_day_raw: Option<i32> = row.get("due_day");
    let due_day = match due_day_raw {
        Some(d) => Some(
            DueDay::new(d as u8)
                .map_err(|e| EntryError::Repository(format!("invalid due_day: {}", e)))?,
        ),
        None => None,
    };

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| EntryError::Repository(format!("invalid created_at: {}", e)))?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| EntryError::Repository(format!("invalid updated_at: {}", e)))?;

    Ok(BudgetEntryWithCategory {
        id,
        category: CategorySummary {
            id: category_id,
            name: category_name,
        },
        budgeted: Money::new(budgeted),
        due_day,
        created_at,
        updated_at,
    })
}

async fn fetch_entry_with_category(
    pool: &SqlitePool,
    entry_id: &ulid::Ulid,
) -> Result<BudgetEntryWithCategory, EntryError> {
    let row = sqlx::query(
        "SELECT e.id, e.category_id, c.name AS category_name, e.budgeted, e.due_day, \
         e.created_at, e.updated_at \
         FROM budget_entries e \
         JOIN categories c ON e.category_id = c.id \
         WHERE e.id = ?",
    )
    .bind(entry_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| EntryError::Repository(e.to_string()))?;

    match row {
        Some(ref r) => map_row_to_entry_with_category(r),
        None => Err(EntryError::NotFound),
    }
}

#[async_trait]
impl BudgetEntryRepository for SqliteBudgetEntryRepository {
    async fn list_by_month(
        &self,
        month_id: &ulid::Ulid,
    ) -> Result<Vec<BudgetEntryWithCategory>, EntryError> {
        let rows = sqlx::query(
            "SELECT e.id, e.category_id, c.name AS category_name, e.budgeted, e.due_day, \
             e.created_at, e.updated_at \
             FROM budget_entries e \
             JOIN categories c ON e.category_id = c.id \
             WHERE e.month_id = ? \
             ORDER BY e.due_day ASC NULLS LAST, c.name ASC",
        )
        .bind(month_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EntryError::Repository(e.to_string()))?;

        rows.iter()
            .map(map_row_to_entry_with_category)
            .collect()
    }

    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<BudgetEntry>, EntryError> {
        let row = sqlx::query("SELECT * FROM budget_entries WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_entry(r)?)),
            None => Ok(None),
        }
    }

    async fn create(
        &self,
        entry: NewBudgetEntry,
    ) -> Result<BudgetEntryWithCategory, EntryError> {
        let id = ulid::Ulid::new();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let due_day_val = entry.due_day.map(|d| d.value() as i32);

        let result = sqlx::query(
            "INSERT INTO budget_entries (id, month_id, category_id, budgeted, due_day, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(entry.month_id.to_string())
        .bind(entry.category_id.to_string())
        .bind(entry.budgeted.value())
        .bind(due_day_val)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                return Err(EntryError::CategoryAlreadyInMonth {
                    category_id: entry.category_id.to_string(),
                    month: entry.month_id.to_string(),
                });
            }
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("FOREIGN KEY constraint failed") =>
            {
                let month_exists = sqlx::query("SELECT 1 FROM months WHERE id = ?")
                    .bind(entry.month_id.to_string())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| EntryError::Repository(e.to_string()))?;
                if month_exists.is_none() {
                    return Err(EntryError::MonthNotFound);
                }
                return Err(EntryError::CategoryNotFound);
            }
            Err(e) => return Err(EntryError::Repository(e.to_string())),
        }

        fetch_entry_with_category(&self.pool, &id).await
    }

    async fn update(
        &self,
        id: &ulid::Ulid,
        budgeted: Option<Money>,
        due_day: Option<Option<DueDay>>,
    ) -> Result<BudgetEntryWithCategory, EntryError> {
        // Build dynamic UPDATE query
        let mut set_clauses: Vec<String> = Vec::new();

        if budgeted.is_some() {
            set_clauses.push("budgeted = ?".to_string());
        }

        if due_day.is_some() {
            set_clauses.push("due_day = ?".to_string());
        }

        if set_clauses.is_empty() {
            return fetch_entry_with_category(&self.pool, id).await;
        }

        let sql = format!(
            "UPDATE budget_entries SET {} WHERE id = ?",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&sql);

        // Bind values in order
        if let Some(b) = budgeted {
            query = query.bind(b.value());
        }
        if let Some(dd) = &due_day {
            query = query.bind(dd.as_ref().map(|d| d.value() as i32));
        }

        query = query.bind(id.to_string());

        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(EntryError::NotFound);
        }

        fetch_entry_with_category(&self.pool, id).await
    }

    async fn delete(&self, id: &ulid::Ulid) -> Result<(), EntryError> {
        let result = sqlx::query("DELETE FROM budget_entries WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(EntryError::NotFound);
        }

        Ok(())
    }

    async fn transaction_count(&self, entry_id: &ulid::Ulid) -> Result<i64, EntryError> {
        let row = sqlx::query("SELECT COUNT(*) AS cnt FROM transactions WHERE entry_id = ?")
            .bind(entry_id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EntryError::Repository(e.to_string()))?;

        let count: i64 = row.get("cnt");
        Ok(count)
    }
}
