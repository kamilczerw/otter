use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use domain::entities::{Category, NewCategory};
use domain::errors::CategoryError;
use domain::ports::CategoryRepository;
use domain::types::CategoryName;

pub struct SqliteCategoryRepository {
    pool: SqlitePool,
}

impl SqliteCategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn map_row_to_category(row: &sqlx::sqlite::SqliteRow) -> Result<Category, CategoryError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| CategoryError::Repository(format!("invalid ULID: {}", e)))?;

    let name_str: String = row.get("name");
    let name = CategoryName::new(name_str.clone())
        .map_err(|e| CategoryError::Repository(format!("invalid category name '{}': {}", name_str, e)))?;

    let label: Option<String> = row.get("label");

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CategoryError::Repository(format!("invalid created_at: {}", e)))?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CategoryError::Repository(format!("invalid updated_at: {}", e)))?;

    Ok(Category {
        id,
        name,
        label,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn list_all(&self) -> Result<Vec<Category>, CategoryError> {
        let rows = sqlx::query("SELECT * FROM categories ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CategoryError::Repository(e.to_string()))?;

        rows.iter()
            .map(map_row_to_category)
            .collect()
    }

    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Category>, CategoryError> {
        let row = sqlx::query("SELECT * FROM categories WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CategoryError::Repository(e.to_string()))?;

        match row {
            Some(ref r) => Ok(Some(map_row_to_category(r)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, category: NewCategory) -> Result<Category, CategoryError> {
        let id = ulid::Ulid::new();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let result = sqlx::query(
            "INSERT INTO categories (id, name, label, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(category.name.as_str())
        .bind(&category.label)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                return Err(CategoryError::NameAlreadyExists {
                    name: category.name.as_str().to_string(),
                });
            }
            Err(e) => return Err(CategoryError::Repository(e.to_string())),
        }

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| CategoryError::Repository("failed to fetch created category".to_string()))
    }

    async fn update_name(
        &self,
        id: &ulid::Ulid,
        name: CategoryName,
    ) -> Result<Category, CategoryError> {
        let result = sqlx::query("UPDATE categories SET name = ? WHERE id = ?")
            .bind(name.as_str())
            .bind(id.to_string())
            .execute(&self.pool)
            .await;

        match result {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    return Err(CategoryError::NotFound);
                }
            }
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                return Err(CategoryError::NameAlreadyExists {
                    name: name.as_str().to_string(),
                });
            }
            Err(e) => return Err(CategoryError::Repository(e.to_string())),
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| CategoryError::Repository("failed to fetch updated category".to_string()))
    }

    async fn update(
        &self,
        id: &ulid::Ulid,
        name: Option<CategoryName>,
        label: Option<Option<String>>,
    ) -> Result<Category, CategoryError> {
        // Build dynamic UPDATE query based on provided fields
        let mut updates = Vec::new();
        let mut query = String::from("UPDATE categories SET ");

        if name.is_some() {
            updates.push("name = ?");
        }
        if label.is_some() {
            updates.push("label = ?");
        }

        if updates.is_empty() {
            // Nothing to update, just fetch and return
            return self.find_by_id(id)
                .await?
                .ok_or(CategoryError::NotFound);
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ?");

        let mut q = sqlx::query(&query);

        if let Some(ref n) = name {
            q = q.bind(n.as_str());
        }
        if let Some(ref l) = label {
            q = q.bind(l);
        }
        q = q.bind(id.to_string());

        let result = q.execute(&self.pool).await;

        match result {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    return Err(CategoryError::NotFound);
                }
            }
            Err(sqlx::Error::Database(ref db_err))
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                return Err(CategoryError::NameAlreadyExists {
                    name: name.map(|n| n.as_str().to_string()).unwrap_or_default(),
                });
            }
            Err(e) => return Err(CategoryError::Repository(e.to_string())),
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| CategoryError::Repository("failed to fetch updated category".to_string()))
    }
}
