//! Top-level CSV import orchestration: decode + parse + extract (see
//! `decode.rs`), categorize (see `categorize.rs`), resolve-or-create
//! month/category/entry (see `provisioning.rs`), skip duplicates, and
//! create the transaction — composed here into one pipeline per the
//! design doc's "Auto-creation order and scope" and "Partial success
//! semantics" sections.

use csv_import::ParsedTransaction;
use domain::entities::ImportDetails;
use domain::types::{Money, TransactionDate};

use crate::import::categorize::resolve_category_name;
use crate::import::decode::{decode_and_extract_rows, ImportError, RowFailure, RowOutcome};
use crate::import::provisioning::{find_or_create_category, find_or_create_entry, find_or_create_month, ImportDeps, RowError};

/// The result of a completed (not wholesale-rejected) import request —
/// everything the spec's "Import result reporting" requirement asks for.
/// Plain enough for the HTTP layer (section 5) to map directly into its
/// response DTO.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ImportOutcome {
    /// Number of transactions newly persisted.
    pub imported: u32,
    /// Number of rows skipped because an identical transaction already
    /// existed (same entry, date, amount, and title).
    pub duplicates: u32,
    /// Number of transactions assigned to the `"uncategorized"` category
    /// (whether or not they were ultimately imported or deduplicated).
    pub uncategorized: u32,
    /// `YYYY-MM` names of months created to hold an imported transaction.
    pub created_months: Vec<String>,
    /// Names of categories created to hold an imported transaction.
    pub created_categories: Vec<String>,
    /// Rows that failed extraction or processing, in file order.
    pub failed: Vec<RowFailure>,
}

/// Runs the full import pipeline over an uploaded CSV file's raw bytes.
///
/// # Errors
///
/// Returns [`ImportError`] when the request is rejected wholesale: an
/// unsupported charset, invalid text for the resolved charset, or content
/// that isn't structurally parseable as CSV. Nothing is persisted in that
/// case. A single bad or unprocessable row is never an `Err` here — it is
/// recorded in the returned [`ImportOutcome::failed`] instead, and the rest
/// of the file is still processed.
pub async fn run_import(
    deps: &ImportDeps,
    bytes: &[u8],
    content_type: Option<&str>,
) -> Result<ImportOutcome, ImportError> {
    let outcomes = decode_and_extract_rows(bytes, content_type)?;

    let mut result = ImportOutcome::default();
    for outcome in outcomes {
        apply_row_outcome(deps, outcome, &mut result).await;
    }
    Ok(result)
}

/// Applies one row's outcome to the running `result`: a row that failed
/// extraction is recorded as-is; a successfully extracted transaction is
/// run through [`import_one_transaction`], with any processing failure
/// recorded the same way an extraction failure would be.
async fn apply_row_outcome(deps: &ImportDeps, outcome: RowOutcome, result: &mut ImportOutcome) {
    match outcome {
        RowOutcome::Failed(failure) => result.failed.push(failure),
        RowOutcome::Extracted { row, transaction } => {
            if let Err(err) = import_one_transaction(deps, &transaction, result).await {
                result.failed.push(RowFailure {
                    row,
                    raw: transaction.raw_description.clone(),
                    reason: err.to_string(),
                });
            }
        }
    }
}

/// Carries a transaction's signed bank amount through unchanged: negative
/// values are expenses and zero/positive values are income, per
/// `Transaction::kind()`'s sign convention.
fn transaction_amount(parsed: &ParsedTransaction) -> Money {
    parsed.amount
}

/// Builds the import provenance metadata attached to every transaction
/// created by the CSV import pipeline, carrying the row's bank-reported
/// fields through unchanged.
fn import_details_from_row(parsed: &ParsedTransaction) -> ImportDetails {
    ImportDetails {
        source: "csv_import".to_string(),
        merchant: parsed.merchant.clone(),
        city: parsed.city.clone(),
        country: parsed.country.clone(),
        card_last4: parsed.card_last4.clone(),
        raw_type: Some(parsed.raw_type.clone()),
        currency: Some(parsed.currency.clone()),
        raw_description: Some(parsed.raw_description.clone()),
        original_amount_fragment: parsed.original_amount_fragment.clone(),
    }
}

/// Runs the per-row pipeline for one extracted transaction: resolve
/// category -> resolve-or-create month -> resolve-or-create category ->
/// resolve-or-create entry -> duplicate check -> create transaction,
/// updating `result`'s counts and created-month/category lists as it goes.
///
/// # Errors
///
/// Returns [`RowError`] if any resolution/creation step fails. The caller
/// records this as a row failure rather than aborting the whole import.
async fn import_one_transaction(
    deps: &ImportDeps,
    transaction: &ParsedTransaction,
    result: &mut ImportOutcome,
) -> Result<(), RowError> {
    let (category_name, is_uncategorized) = resolve_category_name(transaction, &deps.rules);

    let (month, month_created) = find_or_create_month(deps, transaction.date).await?;
    if month_created {
        result.created_months.push(month.month.to_string());
    }

    let (category, category_created) = find_or_create_category(deps, &category_name).await?;
    if category_created {
        result.created_categories.push(category.name.as_str().to_string());
    }

    let entry = find_or_create_entry(deps, month.id, category.id).await?;

    if is_uncategorized {
        result.uncategorized += 1;
    }

    let amount = transaction_amount(transaction);
    let date = TransactionDate::new(transaction.date);

    let duplicate = deps
        .transaction_repo
        .find_duplicate(&entry.id, &date, &amount, transaction.title.as_deref())
        .await?;

    if duplicate.is_some() {
        result.duplicates += 1;
        return Ok(());
    }

    deps.transaction_service
        .create(
            entry.id,
            amount,
            date,
            transaction.title.clone(),
            Some(import_details_from_row(transaction)),
            vec!["imported/csv".to_string()],
        )
        .await?;
    result.imported += 1;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use chrono::NaiveDate;
    use csv_import::MatchCriteria;
    use db::repos::{
        SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
        SqliteTransactionRepository,
    };
    use domain::services::{CategoryService, EntryService, MonthService, TransactionService};

    /// Builds `ImportDeps` backed by a fresh in-memory SQLite database,
    /// following the same setup pattern as the `api` crate's integration
    /// tests (there's no lighter-weight repository fake in this codebase).
    async fn test_deps(rules: Vec<csv_import::Rule>) -> ImportDeps {
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
            rules,
        }
    }

    /// Minimal `ParsedTransaction` fixture; callers override just the
    /// fields their scenario needs.
    fn transaction(merchant: Option<&str>, amount: i64, title: Option<&str>) -> ParsedTransaction {
        ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
            amount: Money::new(amount),
            currency: "PLN".to_string(),
            raw_type: "Płatność kartą".to_string(),
            card_last4: None,
            merchant: merchant.map(str::to_string),
            city: None,
            country: None,
            title: title.map(str::to_string),
            raw_description: "test row".to_string(),
            original_amount_fragment: None,
        }
    }

    fn netto_rule() -> csv_import::Rule {
        csv_import::Rule {
            pattern: MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                ..Default::default()
            },
            category: "groceries".to_string(),
        }
    }

    /// A transaction matching a configured rule is persisted under that
    /// rule's category, and the import/uncategorized counts reflect it.
    #[tokio::test]
    async fn imports_transaction_matching_a_rule() {
        let deps = test_deps(vec![netto_rule()]).await;
        let txn = transaction(Some("NETTO 5567"), -1050, Some("groceries run"));

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        assert_eq!(result.imported, 1);
        assert_eq!(result.duplicates, 0);
        assert_eq!(result.uncategorized, 0);
        assert_eq!(result.created_categories, vec!["groceries".to_string()]);
    }

    /// A transaction matching no rule is persisted under "uncategorized",
    /// and counted as such.
    #[tokio::test]
    async fn falls_back_to_uncategorized_when_no_rule_matches() {
        let deps = test_deps(vec![netto_rule()]).await;
        let txn = transaction(Some("UNKNOWN SHOP"), -500, None);

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        assert_eq!(result.imported, 1);
        assert_eq!(result.uncategorized, 1);
        assert_eq!(result.created_categories, vec!["uncategorized".to_string()]);
    }

    /// A transaction matching a rule whose category is empty falls back to
    /// uncategorized the same as a non-matching transaction would.
    #[tokio::test]
    async fn falls_back_to_uncategorized_when_rule_category_is_invalid() {
        let invalid_rule = csv_import::Rule {
            pattern: MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                ..Default::default()
            },
            category: String::new(),
        };
        let deps = test_deps(vec![invalid_rule]).await;
        let txn = transaction(Some("NETTO 5567"), -500, None);

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        assert_eq!(result.uncategorized, 1);
        assert_eq!(result.created_categories, vec!["uncategorized".to_string()]);
    }

    /// Importing the same transaction a second time is skipped as a
    /// duplicate, not double-counted as imported.
    #[tokio::test]
    async fn skips_duplicate_transaction_on_second_import() {
        let deps = test_deps(vec![netto_rule()]).await;
        let txn = transaction(Some("NETTO 5567"), -1050, Some("groceries run"));

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        assert_eq!(result.imported, 1);
        assert_eq!(result.duplicates, 1);
    }

    /// A transaction needing a brand-new month, category, and entry gets
    /// all three auto-created, and the month/category are reported as
    /// newly created exactly once.
    #[tokio::test]
    async fn auto_creates_missing_month_category_and_entry() {
        let deps = test_deps(vec![netto_rule()]).await;
        let txn = transaction(Some("NETTO 5567"), -1050, Some("groceries run"));

        assert!(deps.month_service.list_all().await.unwrap().is_empty());
        assert!(deps.category_service.list_all().await.unwrap().is_empty());

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        let months = deps.month_service.list_all().await.unwrap();
        assert_eq!(months.len(), 1);
        assert_eq!(result.created_months, vec![months[0].month.to_string()]);

        let categories = deps.category_service.list_all().await.unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name.as_str(), "groceries");

        let entries = deps.entry_service.list_by_month(&months[0].id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].category.id, categories[0].id);
    }

    /// A transaction with a positive (credit) amount is persisted as-is —
    /// not negated or rejected — and is classified `Income` by
    /// `Transaction::kind()`.
    #[tokio::test]
    async fn imports_credit_amount_as_income() {
        let deps = test_deps(vec![]).await;
        let txn = transaction(Some("REFUND SHOP"), 1050, Some("refund"));

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        assert_eq!(result.imported, 1);
        assert!(result.failed.is_empty());

        let months = deps.month_service.list_all().await.unwrap();
        let transactions = deps.transaction_repo.list_by_month(&months[0].id).await.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, Money::new(1050));
        assert_eq!(transactions[0].kind(), domain::entities::TransactionKind::Income);
    }

    /// An imported transaction is tagged `"imported/csv"` and carries its
    /// source row's merchant/source metadata in `import_details`, so later
    /// UI/features can distinguish imported from manually entered rows.
    #[tokio::test]
    async fn imported_transaction_carries_tags_and_import_details() {
        let deps = test_deps(vec![]).await;
        let txn = transaction(Some("NETTO 5567"), -1050, Some("groceries run"));

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        let months = deps.month_service.list_all().await.unwrap();
        let transactions = deps.transaction_repo.list_by_month(&months[0].id).await.unwrap();
        assert_eq!(transactions.len(), 1);

        let created = &transactions[0];
        assert_eq!(created.tags, vec!["imported/csv".to_string()]);

        let import_details = created.import_details.as_ref().expect("import_details should be set");
        assert_eq!(import_details.source, "csv_import");
        assert_eq!(import_details.merchant, Some("NETTO 5567".to_string()));
    }

    /// A transaction created via CSV import maps to a `TransactionResponse`
    /// with non-null `metadata` and the `"imported/csv"` tag, while a
    /// manually created transaction (no import details, no tags) maps to a
    /// response with `metadata: None` and an empty `tags` list.
    #[tokio::test]
    async fn transaction_response_reflects_import_metadata_and_tags() {
        use crate::responses::TransactionResponse;

        let deps = test_deps(vec![]).await;
        let txn = transaction(Some("NETTO 5567"), -1050, Some("groceries run"));

        let mut result = ImportOutcome::default();
        import_one_transaction(&deps, &txn, &mut result).await.unwrap();

        let months = deps.month_service.list_all().await.unwrap();
        let imported = deps.transaction_repo.list_by_month(&months[0].id).await.unwrap();
        assert_eq!(imported.len(), 1);

        let imported_response = TransactionResponse::from(imported[0].clone());
        let metadata = imported_response.metadata.expect("imported transaction should carry metadata");
        assert_eq!(metadata.source, "csv_import");
        assert_eq!(metadata.merchant, Some("NETTO 5567".to_string()));
        assert_eq!(imported_response.tags, vec!["imported/csv".to_string()]);

        let entry_id = imported[0].entry_id;
        let manual = deps
            .transaction_service
            .create(
                entry_id,
                Money::new(-500),
                TransactionDate::new(NaiveDate::from_ymd_opt(2026, 10, 9).unwrap()),
                Some("manual entry".to_string()),
                None,
                vec![],
            )
            .await
            .unwrap();

        let manual_response = TransactionResponse::from(manual);
        assert!(manual_response.metadata.is_none());
        assert!(manual_response.tags.is_empty());
    }

    /// `run_import` end-to-end: decodes, extracts, and imports a tiny
    /// well-formed CSV, confirming the public entry point wires everything
    /// together (not just the inner per-transaction step).
    #[tokio::test]
    async fn run_import_processes_a_minimal_csv() {
        let deps = test_deps(vec![]).await;
        let csv = "\"Data operacji\",\"Data waluty\",\"Typ transakcji\",\"Kwota\",\"Waluta\",\"Saldo po transakcji\",\"Opis transakcji\",\"\",\"\",\"\",\"\",\"\",\"\"\n\
             \"2026-10-09\",\"2026-10-09\",\"Autooszczedzanie\",\"-2.86\",\"PLN\",\"\",\"TEST\",\"\",\"\",\"\",\"\",\"\",\"\"\n";

        let outcome = run_import(&deps, csv.as_bytes(), None).await.unwrap();
        assert_eq!(outcome.imported, 1);
        assert_eq!(outcome.uncategorized, 1);
        assert!(outcome.failed.is_empty());
    }
}
