//! SQLite adapter for `domain::ports::MatchingRuleRepository`: persists
//! user-stored matching rules, serializing their criteria list as JSON and
//! tracking evaluation order via the `position` column (see the
//! `add-rule-editor-ui` design doc's "Decisions" section).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use domain::entities::{MatchCriterion, MatchingRule, NewMatchingRule};
use domain::errors::MatchingRuleError;
use domain::ports::MatchingRuleRepository;

/// SQLite-backed implementation of [`MatchingRuleRepository`].
pub struct MatchingRuleRepo {
    pool: SqlitePool,
}

impl MatchingRuleRepo {
    /// Builds a repository over an already-migrated connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Fetches a single rule by id, used to re-read a row after an
    /// insert/update so the returned `MatchingRule` reflects DB-assigned
    /// fields (position, timestamps).
    ///
    /// # Errors
    ///
    /// Returns [`MatchingRuleError::Repository`] if the query or row mapping
    /// fails.
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<MatchingRule>, MatchingRuleError> {
        let row = sqlx::query("SELECT * FROM matching_rules WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MatchingRuleError::Repository(e.to_string()))?;

        row.as_ref().map(map_row_to_rule).transpose()
    }
}

/// Serializes a rule's criteria list to a JSON string for storage.
///
/// # Errors
///
/// Returns [`MatchingRuleError::Repository`] if serialization fails.
fn serialize_criteria(criteria: &[MatchCriterion]) -> Result<String, MatchingRuleError> {
    serde_json::to_string(criteria)
        .map_err(|e| MatchingRuleError::Repository(format!("invalid criteria: {e}")))
}

/// Parses a stored RFC3339 timestamp column, wrapping any parse failure as
/// a [`MatchingRuleError::Repository`] tagged with the offending column name.
fn parse_timestamp(value: &str, column: &str) -> Result<DateTime<Utc>, MatchingRuleError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MatchingRuleError::Repository(format!("invalid {column}: {e}")))
}

/// Maps a `matching_rules` row into a [`MatchingRule`], deserializing the
/// JSON-stored criteria list and parsing the ULID/timestamp columns.
fn map_row_to_rule(row: &sqlx::sqlite::SqliteRow) -> Result<MatchingRule, MatchingRuleError> {
    let id_str: String = row.get("id");
    let id = ulid::Ulid::from_string(&id_str)
        .map_err(|e| MatchingRuleError::Repository(format!("invalid ULID: {e}")))?;

    let criteria_json: String = row.get("criteria");
    let criteria: Vec<MatchCriterion> = serde_json::from_str(&criteria_json)
        .map_err(|e| MatchingRuleError::Repository(format!("invalid criteria: {e}")))?;

    let category: String = row.get("category");

    let created_at_str: String = row.get("created_at");
    let created_at = parse_timestamp(&created_at_str, "created_at")?;

    let updated_at_str: String = row.get("updated_at");
    let updated_at = parse_timestamp(&updated_at_str, "updated_at")?;

    Ok(MatchingRule {
        id,
        criteria,
        category,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl MatchingRuleRepository for MatchingRuleRepo {
    async fn list_all(&self) -> Result<Vec<MatchingRule>, MatchingRuleError> {
        let rows = sqlx::query("SELECT * FROM matching_rules ORDER BY position ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MatchingRuleError::Repository(e.to_string()))?;

        rows.iter().map(map_row_to_rule).collect()
    }

    async fn create(&self, rule: NewMatchingRule) -> Result<MatchingRule, MatchingRuleError> {
        let id = ulid::Ulid::new();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let criteria_json = serialize_criteria(&rule.criteria)?;

        sqlx::query(
            "INSERT INTO matching_rules (id, criteria, category, position, created_at, updated_at) \
             VALUES (?, ?, ?, (SELECT COALESCE(MAX(position), 0) + 1 FROM matching_rules), ?, ?)",
        )
        .bind(id.to_string())
        .bind(&criteria_json)
        .bind(&rule.category)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| MatchingRuleError::Repository(e.to_string()))?;

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| MatchingRuleError::Repository("failed to fetch created rule".to_string()))
    }

    async fn update(
        &self,
        id: &ulid::Ulid,
        rule: NewMatchingRule,
    ) -> Result<MatchingRule, MatchingRuleError> {
        let criteria_json = serialize_criteria(&rule.criteria)?;

        let result = sqlx::query("UPDATE matching_rules SET criteria = ?, category = ? WHERE id = ?")
            .bind(&criteria_json)
            .bind(&rule.category)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| MatchingRuleError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(MatchingRuleError::NotFound);
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| MatchingRuleError::Repository("failed to fetch updated rule".to_string()))
    }

    async fn delete(&self, id: &ulid::Ulid) -> Result<(), MatchingRuleError> {
        let result = sqlx::query("DELETE FROM matching_rules WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| MatchingRuleError::Repository(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(MatchingRuleError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::entities::{MatchField, MatchType};

    /// Spins up an in-memory SQLite pool with migrations applied, mirroring
    /// the pattern used elsewhere in this crate's tests.
    async fn setup_pool() -> SqlitePool {
        let pool = crate::create_pool("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");
        crate::run_migrations(&pool)
            .await
            .expect("failed to run migrations");
        pool
    }

    /// Builds a minimal valid `NewMatchingRule` with one `Contains` criterion
    /// so tests don't repeat criteria construction boilerplate.
    fn new_rule(category: &str) -> NewMatchingRule {
        NewMatchingRule {
            criteria: vec![MatchCriterion::new(
                MatchField::Merchant,
                MatchType::Contains("netto".to_string()),
            )
            .unwrap()],
            category: category.to_string(),
        }
    }

    #[tokio::test]
    async fn create_and_list_all_orders_by_position() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let first = repo.create(new_rule("groceries")).await.unwrap();
        let second = repo.create(new_rule("transport")).await.unwrap();

        let rules = repo.list_all().await.unwrap();

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, first.id);
        assert_eq!(rules[1].id, second.id);
        assert_eq!(rules[0].category, "groceries");
        assert_eq!(rules[1].category, "transport");
    }

    #[tokio::test]
    async fn list_all_is_empty_when_no_rules_exist() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let rules = repo.list_all().await.unwrap();

        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn update_changes_criteria_and_category_without_moving_position() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let first = repo.create(new_rule("groceries")).await.unwrap();
        let second = repo.create(new_rule("transport")).await.unwrap();

        let updated = repo
            .update(&first.id, new_rule("utilities"))
            .await
            .unwrap();

        assert_eq!(updated.category, "utilities");

        // Position is unchanged: `first` (now "utilities") still sorts
        // before `second` ("transport") in `list_all`.
        let rules = repo.list_all().await.unwrap();
        assert_eq!(rules[0].id, first.id);
        assert_eq!(rules[0].category, "utilities");
        assert_eq!(rules[1].id, second.id);
    }

    #[tokio::test]
    async fn update_of_nonexistent_id_returns_not_found() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let result = repo.update(&ulid::Ulid::new(), new_rule("groceries")).await;

        assert!(matches!(result, Err(MatchingRuleError::NotFound)));
    }

    #[tokio::test]
    async fn delete_removes_rule() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let rule = repo.create(new_rule("groceries")).await.unwrap();
        repo.delete(&rule.id).await.unwrap();

        let rules = repo.list_all().await.unwrap();
        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn delete_of_nonexistent_id_returns_not_found() {
        let pool = setup_pool().await;
        let repo = MatchingRuleRepo::new(pool);

        let result = repo.delete(&ulid::Ulid::new()).await;

        assert!(matches!(result, Err(MatchingRuleError::NotFound)));
    }
}
