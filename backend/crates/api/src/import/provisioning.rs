//! Find-or-create helpers for the month/category/budget-entry a CSV import
//! row needs but that may not exist yet.
//!
//! Each helper is a single, focused lookup-or-create step; the pipeline
//! (`pipeline.rs`) composes them in order. None of them introduce new
//! domain rules — they just call the existing repository/service methods
//! documented in the design doc's "Auto-creation order and scope" section.

use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use ulid::Ulid;

use domain::entities::{BudgetEntryWithCategory, Category, Month};
use domain::errors::{CategoryError, DomainError, EntryError, MatchingRuleError, MonthError, TransactionError};
use domain::ports::{MatchingRuleRepository, MonthRepository, TransactionRepository};
use domain::services::{CategoryService, EntryService, MonthService, TransactionService};
use domain::types::{BudgetMonth, CategoryName, Money};

use crate::import::rule_conversion::domain_rule_to_csv_rule;

/// Everything the import pipeline needs to resolve/create months,
/// categories, budget entries, and transactions, plus the categorization
/// rules to apply. Bundled into one struct so pipeline functions take a
/// single dependency argument instead of a long parameter list.
pub struct ImportDeps {
    /// Needed directly (not just via `month_service`) for the
    /// find-by-month lookup that has no service-level equivalent.
    pub month_repo: Arc<dyn MonthRepository>,
    pub month_service: Arc<MonthService>,
    pub category_service: Arc<CategoryService>,
    pub entry_service: Arc<EntryService>,
    /// Needed directly (not just via `transaction_service`) for duplicate
    /// detection, which has no service-level equivalent.
    pub transaction_repo: Arc<dyn TransactionRepository>,
    pub transaction_service: Arc<TransactionService>,
    /// DB-backed store of user-stored matching rules (see the
    /// `add-rule-editor-ui` design doc). Queried fresh on every
    /// [`ImportDeps::current_rules`] call rather than cached once at
    /// construction, so a rule saved mid-session (e.g. by the
    /// bulk-categorize API) is picked up without restarting the server.
    pub rule_repo: Arc<dyn MatchingRuleRepository>,
    /// Hand-curated rules from the config's `[transaction.import]
    /// matching_rules]`, checked ahead of `rule_repo`'s rules in
    /// [`ImportDeps::current_rules`]. Unlike `rule_repo`, never written to
    /// by the bulk-categorize API.
    pub inline_rules: Vec<csv_import::Rule>,
}

impl ImportDeps {
    /// Loads the current set of categorization rules: `inline_rules`
    /// followed by every user-stored rule from `rule_repo`, queried fresh on
    /// every call (see the field doc above) so a rule learned mid-session is
    /// picked up without restarting the server.
    ///
    /// # Errors
    ///
    /// Returns a [`MatchingRuleError`] if `rule_repo.list_all()` fails.
    pub async fn current_rules(&self) -> Result<Vec<csv_import::Rule>, MatchingRuleError> {
        let mut rules = self.inline_rules.clone();
        let user_rules = self.rule_repo.list_all().await?;
        rules.extend(user_rules.into_iter().map(domain_rule_to_csv_rule));
        Ok(rules)
    }
}

/// Errors that can occur while resolving or auto-creating a row's month,
/// category, or budget entry, or while creating/looking up its transaction.
///
/// These are per-row failures: the pipeline catches them and records the
/// row in the response's `failed` list rather than aborting the import.
#[derive(Debug, thiserror::Error)]
pub enum RowError {
    /// The transaction's date doesn't fall in a representable budget month
    /// (e.g. a `BudgetMonth` year outside the supported 2000-2100 range).
    #[error("transaction date {date} doesn't fall in a valid budget month: {source}")]
    InvalidMonth { date: NaiveDate, source: DomainError },
    #[error(transparent)]
    Month(#[from] MonthError),
    #[error(transparent)]
    Category(#[from] CategoryError),
    #[error(transparent)]
    Entry(#[from] EntryError),
    #[error(transparent)]
    Transaction(#[from] TransactionError),
    /// Loading the current categorization rules (config defaults plus
    /// user-stored rules) failed.
    #[error(transparent)]
    MatchingRule(#[from] MatchingRuleError),
}

/// Finds the `Month` covering `date`'s year/month, creating an empty month
/// (no entries copied in) if none exists yet.
///
/// # Returns
///
/// The resolved month, and whether it was newly created.
///
/// # Errors
///
/// Returns [`RowError::InvalidMonth`] if `date`'s year/month isn't a valid
/// [`BudgetMonth`], or a wrapped [`MonthError`] if the lookup or creation
/// fails.
pub async fn find_or_create_month(
    deps: &ImportDeps,
    date: NaiveDate,
) -> Result<(Month, bool), RowError> {
    let budget_month =
        BudgetMonth::new(date.year(), date.month() as u8).map_err(|source| RowError::InvalidMonth {
            date,
            source,
        })?;

    if let Some(existing) = deps.month_repo.find_by_month(&budget_month).await? {
        return Ok((existing, false));
    }

    let created = deps.month_service.create(budget_month, None, true).await?;
    Ok((created, true))
}

/// Finds a `Category` by exact name (linear scan over `list_all`, per the
/// design doc — category counts are small), creating it if none exists yet.
///
/// # Returns
///
/// The resolved category, and whether it was newly created.
///
/// # Errors
///
/// Returns a wrapped [`CategoryError`] if the listing or creation fails.
pub async fn find_or_create_category(
    deps: &ImportDeps,
    name: &CategoryName,
) -> Result<(Category, bool), RowError> {
    let existing = deps
        .category_service
        .list_all()
        .await?
        .into_iter()
        .find(|category| category.name.as_str() == name.as_str());

    if let Some(category) = existing {
        return Ok((category, false));
    }

    let created = deps.category_service.create(name.clone(), None).await?;
    Ok((created, true))
}

/// Finds the budget entry linking `month_id` and `category_id` (linear scan
/// over `list_by_month`, per the design doc), creating one with a zero
/// budgeted amount and no due day if none exists yet.
///
/// # Errors
///
/// Returns a wrapped [`EntryError`] if the listing or creation fails.
pub async fn find_or_create_entry(
    deps: &ImportDeps,
    month_id: Ulid,
    category_id: Ulid,
) -> Result<BudgetEntryWithCategory, RowError> {
    let existing = deps
        .entry_service
        .list_by_month(&month_id)
        .await?
        .into_iter()
        .find(|entry| entry.category.id == category_id);

    if let Some(entry) = existing {
        return Ok(entry);
    }

    let created = deps
        .entry_service
        .create(month_id, category_id, Money::new(0), None)
        .await?;
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use db::repos::{
        SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
        SqliteTransactionRepository,
    };
    use db::MatchingRuleRepo;

    /// Builds `ImportDeps` backed by a fresh in-memory SQLite database,
    /// following the same setup pattern as the `api` crate's integration
    /// tests (there's no lighter-weight repository fake in this codebase).
    /// `rule_repo` is backed by that same empty database, so
    /// `current_rules()` resolves to an empty rule set (these tests don't
    /// exercise categorization).
    async fn test_deps() -> ImportDeps {
        let pool = db::create_pool("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");
        db::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        let month_repo = Arc::new(SqliteMonthRepository::new(pool.clone()));
        let category_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
        let entry_repo = Arc::new(SqliteBudgetEntryRepository::new(pool.clone()));
        let transaction_repo = Arc::new(SqliteTransactionRepository::new(pool.clone()));
        let rule_repo = Arc::new(MatchingRuleRepo::new(pool.clone()));

        ImportDeps {
            month_repo: month_repo.clone(),
            month_service: Arc::new(MonthService::new(month_repo.clone(), entry_repo.clone())),
            category_service: Arc::new(CategoryService::new(category_repo.clone())),
            entry_service: Arc::new(EntryService::new(
                entry_repo.clone(),
                category_repo.clone(),
                month_repo.clone(),
            )),
            transaction_repo: transaction_repo.clone(),
            transaction_service: Arc::new(TransactionService::new(
                transaction_repo.clone(),
                entry_repo.clone(),
            )),
            rule_repo,
            inline_rules: Vec::new(),
        }
    }

    /// The first call for a given date creates a new, empty month; a
    /// second call for the same date finds it instead of creating another.
    #[tokio::test]
    async fn find_or_create_month_creates_then_finds() {
        let deps = test_deps().await;
        let date = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();

        let (created, was_created) = find_or_create_month(&deps, date).await.unwrap();
        assert!(was_created);
        assert_eq!(created.month.year(), 2026);
        assert_eq!(created.month.month(), 10);

        let (found, was_created) = find_or_create_month(&deps, date).await.unwrap();
        assert!(!was_created);
        assert_eq!(found.id, created.id);
    }

    /// The first call for a given name creates a new category; a second
    /// call for the same name finds it instead of creating another.
    #[tokio::test]
    async fn find_or_create_category_creates_then_finds() {
        let deps = test_deps().await;
        let name = CategoryName::new("groceries").unwrap();

        let (created, was_created) = find_or_create_category(&deps, &name).await.unwrap();
        assert!(was_created);
        assert_eq!(created.name.as_str(), "groceries");

        let (found, was_created) = find_or_create_category(&deps, &name).await.unwrap();
        assert!(!was_created);
        assert_eq!(found.id, created.id);
    }

    /// The first call for a given (month, category) pair creates a new
    /// zero-budgeted entry; a second call finds the same entry instead.
    #[tokio::test]
    async fn find_or_create_entry_creates_then_finds() {
        let deps = test_deps().await;
        let date = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        let (month, _) = find_or_create_month(&deps, date).await.unwrap();
        let name = CategoryName::new("groceries").unwrap();
        let (category, _) = find_or_create_category(&deps, &name).await.unwrap();

        let created = find_or_create_entry(&deps, month.id, category.id)
            .await
            .unwrap();
        assert_eq!(created.budgeted, Money::new(0));

        let found = find_or_create_entry(&deps, month.id, category.id)
            .await
            .unwrap();
        assert_eq!(found.id, created.id);
    }
}
