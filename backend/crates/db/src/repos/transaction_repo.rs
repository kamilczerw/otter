use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::str::FromStr;

use domain::entities::{ImportDetails, NewTransaction, Transaction};
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

/// Serializes `import_details` to a JSON string for storage, or `None` when
/// absent so the column is written as SQL `NULL`.
///
/// # Errors
///
/// Returns `TransactionError::Repository` if serialization fails.
fn serialize_import_details(
    import_details: &Option<ImportDetails>,
) -> Result<Option<String>, TransactionError> {
    import_details
        .as_ref()
        .map(|d| {
            serde_json::to_string(d)
                .map_err(|e| TransactionError::Repository(format!("invalid import_details: {}", e)))
        })
        .transpose()
}

/// Serializes `tags` to a JSON array string for storage (`"[]"` when empty).
///
/// # Errors
///
/// Returns `TransactionError::Repository` if serialization fails.
fn serialize_tags(tags: &[String]) -> Result<String, TransactionError> {
    serde_json::to_string(tags)
        .map_err(|e| TransactionError::Repository(format!("invalid tags: {}", e)))
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

    let import_details_json: Option<String> = row.get("import_details");
    let import_details = import_details_json
        .map(|json| {
            serde_json::from_str::<ImportDetails>(&json)
                .map_err(|e| TransactionError::Repository(format!("invalid import_details: {}", e)))
        })
        .transpose()?;

    let tags_json: String = row.get("tags");
    let tags: Vec<String> = serde_json::from_str(&tags_json)
        .map_err(|e| TransactionError::Repository(format!("invalid tags: {}", e)))?;

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
        import_details,
        tags,
    })
}

#[async_trait]
impl TransactionRepository for SqliteTransactionRepository {
    async fn list_by_month(
        &self,
        month_id: &ulid::Ulid,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let rows = sqlx::query(
            "SELECT t.id, t.entry_id, t.amount, t.date, t.title, t.import_details, t.tags, \
             t.created_at, t.updated_at \
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

        let import_details_json = serialize_import_details(&transaction.import_details)?;
        let tags_json = serialize_tags(&transaction.tags)?;

        let result = sqlx::query(
            "INSERT INTO transactions (id, entry_id, amount, date, title, import_details, tags, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(transaction.entry_id.to_string())
        .bind(transaction.amount.value())
        .bind(transaction.date.to_string())
        .bind(&transaction.title)
        .bind(&import_details_json)
        .bind(&tags_json)
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
            "SELECT id, entry_id, amount, date, title, import_details, tags, created_at, updated_at \
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

    /// Sums the magnitude of all transactions recorded against an entry.
    ///
    /// Amounts are summed as absolute values (`SUM(ABS(amount))`) so that
    /// expense transactions (negative amounts) and income transactions
    /// (positive amounts) both contribute a positive "paid" total. This is
    /// an intentional magnitude sum, not ledger-accurate netting; a refund
    /// recorded in the same category as its original expense will not
    /// offset it.
    async fn sum_by_entry(
        &self,
        entry_id: &ulid::Ulid,
    ) -> Result<Money, TransactionError> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(ABS(amount)), 0) AS total FROM transactions WHERE entry_id = ?",
        )
        .bind(entry_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TransactionError::Repository(e.to_string()))?;

        let total: i64 = row.get("total");
        Ok(Money::new(total))
    }

    async fn find_duplicate(
        &self,
        entry_id: &ulid::Ulid,
        date: &TransactionDate,
        amount: &Money,
        title: Option<&str>,
    ) -> Result<Option<Transaction>, TransactionError> {
        let row = match title {
            Some(t) => {
                sqlx::query(
                    "SELECT id, entry_id, amount, date, title, import_details, tags, created_at, updated_at \
                     FROM transactions \
                     WHERE entry_id = ? AND date = ? AND amount = ? AND title = ? \
                     LIMIT 1",
                )
                .bind(entry_id.to_string())
                .bind(date.to_string())
                .bind(amount.value())
                .bind(t)
                .fetch_optional(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    "SELECT id, entry_id, amount, date, title, import_details, tags, created_at, updated_at \
                     FROM transactions \
                     WHERE entry_id = ? AND date = ? AND amount = ? AND title IS NULL \
                     LIMIT 1",
                )
                .bind(entry_id.to_string())
                .bind(date.to_string())
                .bind(amount.value())
                .fetch_optional(&self.pool)
                .await
            }
        }
        .map_err(|e| TransactionError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_transaction(r)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::entities::NewTransaction;

    /// Spins up an in-memory SQLite pool with migrations applied, mirroring
    /// the pattern used by the `api` crate's integration tests.
    async fn setup_pool() -> SqlitePool {
        let pool = crate::create_pool("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");
        crate::run_migrations(&pool)
            .await
            .expect("failed to run migrations");
        pool
    }

    /// Inserts a category, month, and budget entry, returning the entry's id,
    /// so tests can create transactions against a valid `entry_id`.
    async fn seed_entry(pool: &SqlitePool) -> ulid::Ulid {
        let category_id = ulid::Ulid::new();
        let month_id = ulid::Ulid::new();
        let entry_id = ulid::Ulid::new();
        let now = "2026-01-01T00:00:00.000Z";

        sqlx::query("INSERT INTO categories (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(category_id.to_string())
            .bind("groceries")
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO months (id, month, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(month_id.to_string())
            .bind("2026-01")
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO budget_entries (id, month_id, category_id, budgeted, due_day, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry_id.to_string())
        .bind(month_id.to_string())
        .bind(category_id.to_string())
        .bind(0_i64)
        .bind(Option::<i64>::None)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .unwrap();

        entry_id
    }

    #[tokio::test]
    async fn sum_by_entry_returns_magnitude_for_expense_transaction() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        repo.create(NewTransaction {
            entry_id,
            amount: Money::new(-1500),
            date,
            title: None,
            import_details: None,
            tags: vec![],
        })
        .await
        .unwrap();

        let total = repo.sum_by_entry(&entry_id).await.unwrap();

        assert_eq!(total, Money::new(1500));
    }

    #[tokio::test]
    async fn sum_by_entry_returns_magnitude_for_income_transaction() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        repo.create(NewTransaction {
            entry_id,
            amount: Money::new(1500),
            date,
            title: None,
            import_details: None,
            tags: vec![],
        })
        .await
        .unwrap();

        let total = repo.sum_by_entry(&entry_id).await.unwrap();

        assert_eq!(total, Money::new(1500));
    }

    #[tokio::test]
    async fn find_duplicate_matches_identical_transaction() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        let created = repo
            .create(NewTransaction {
                entry_id,
                amount: Money::new(1500),
                date,
                title: Some("Lidl".to_string()),
                import_details: None,
                tags: vec![],
            })
            .await
            .unwrap();

        let found = repo
            .find_duplicate(&entry_id, &date, &Money::new(1500), Some("Lidl"))
            .await
            .unwrap();

        assert_eq!(found.map(|t| t.id), Some(created.id));
    }

    #[tokio::test]
    async fn find_duplicate_matches_untitled_transaction_only_against_untitled() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        let created = repo
            .create(NewTransaction {
                entry_id,
                amount: Money::new(2000),
                date,
                title: None,
                import_details: None,
                tags: vec![],
            })
            .await
            .unwrap();

        let found = repo
            .find_duplicate(&entry_id, &date, &Money::new(2000), None)
            .await
            .unwrap();

        assert_eq!(found.map(|t| t.id), Some(created.id));
    }

    #[tokio::test]
    async fn find_duplicate_returns_none_when_no_match() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        repo.create(NewTransaction {
            entry_id,
            amount: Money::new(1500),
            date,
            title: Some("Lidl".to_string()),
            import_details: None,
            tags: vec![],
        })
        .await
        .unwrap();

        // Different amount.
        assert!(repo
            .find_duplicate(&entry_id, &date, &Money::new(999), Some("Lidl"))
            .await
            .unwrap()
            .is_none());

        // Different date.
        let other_date: TransactionDate = "2026-01-16".parse().unwrap();
        assert!(repo
            .find_duplicate(&entry_id, &other_date, &Money::new(1500), Some("Lidl"))
            .await
            .unwrap()
            .is_none());

        // Different title.
        assert!(repo
            .find_duplicate(&entry_id, &date, &Money::new(1500), Some("Biedronka"))
            .await
            .unwrap()
            .is_none());

        // Untitled probe against a titled row.
        assert!(repo
            .find_duplicate(&entry_id, &date, &Money::new(1500), None)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn create_and_find_by_id_round_trip_populated_import_details_and_tags() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        let import_details = ImportDetails {
            source: "mbank_csv".to_string(),
            merchant: Some("Lidl".to_string()),
            city: Some("Warsaw".to_string()),
            country: Some("PL".to_string()),
            card_last4: Some("1234".to_string()),
            raw_type: Some("Płatność kartą".to_string()),
            currency: Some("PLN".to_string()),
            raw_description: Some("PLATNOSC KARTA LIDL WARSZAWA".to_string()),
            original_amount_fragment: Some("-123,45".to_string()),
        };
        let tags = vec!["groceries".to_string(), "recurring".to_string()];

        let created = repo
            .create(NewTransaction {
                entry_id,
                amount: Money::new(-12345),
                date,
                title: Some("Lidl".to_string()),
                import_details: Some(import_details.clone()),
                tags: tags.clone(),
            })
            .await
            .unwrap();

        let found = repo
            .find_by_id(&created.id)
            .await
            .unwrap()
            .expect("transaction should exist");

        assert_eq!(found.id, created.id);
        assert_eq!(found.entry_id, entry_id);
        assert_eq!(found.amount, Money::new(-12345));
        assert_eq!(found.title, Some("Lidl".to_string()));
        assert_eq!(found.import_details, Some(import_details));
        assert_eq!(found.tags, tags);
    }

    #[tokio::test]
    async fn create_and_find_by_id_round_trip_absent_import_details_and_tags() {
        let pool = setup_pool().await;
        let repo = SqliteTransactionRepository::new(pool.clone());
        let entry_id = seed_entry(&pool).await;
        let date: TransactionDate = "2026-01-15".parse().unwrap();

        let created = repo
            .create(NewTransaction {
                entry_id,
                amount: Money::new(1000),
                date,
                title: None,
                import_details: None,
                tags: vec![],
            })
            .await
            .unwrap();

        // Verify the raw column values directly: import_details is SQL
        // NULL and tags is the literal empty-array JSON string.
        let row = sqlx::query("SELECT import_details, tags FROM transactions WHERE id = ?")
            .bind(created.id.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
        let raw_import_details: Option<String> = row.get("import_details");
        let raw_tags: String = row.get("tags");
        assert_eq!(raw_import_details, None);
        assert_eq!(raw_tags, "[]");

        let found = repo
            .find_by_id(&created.id)
            .await
            .unwrap()
            .expect("transaction should exist");

        assert_eq!(found.import_details, None);
        assert_eq!(found.tags, Vec::<String>::new());
    }
}
