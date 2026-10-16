//! Shared test harness for the `api` crate's integration tests.
//!
//! `setup()` in `api_tests.rs` builds a real in-memory SQLite-backed
//! `Router` but discards the services it created, so other test files
//! can't query the database state directly afterward. The import tests
//! need exactly that (persisted rows, not just response JSON), so this
//! module factors the same construction out into a reusable [`TestApp`]
//! that keeps those services around.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use db::repos::{
    SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
    SqliteTransactionRepository,
};
use domain::services::{CategoryService, EntryService, MonthService, SummaryService, TransactionService};

use otter::handlers::AppState;
use otter::import::ImportDeps;

/// A fully wired test application: the router under test, plus the
/// domain services backing it, so a test can both exercise the HTTP API
/// and independently verify what actually landed in the database.
pub struct TestApp {
    pub router: Router,
    pub month_service: Arc<MonthService>,
    pub category_service: Arc<CategoryService>,
    pub entry_service: Arc<EntryService>,
    pub transaction_service: Arc<TransactionService>,
    /// Exposed so rule-saving tests can inspect the on-disk rules file
    /// directly (e.g. to confirm a rejected request left it untouched).
    pub import_deps: Arc<ImportDeps>,
}

/// Builds a [`TestApp`] backed by a fresh in-memory SQLite database with
/// migrations applied, wiring in `rules` as the import pipeline's
/// categorization rules (written to a fresh temp rules file — an empty
/// `rules` list is instead left pointing at a nonexistent path, which
/// `ImportDeps::current_rules` treats as no rules).
///
/// Only `/imports` and the `/transactions-review` routes are mounted
/// — this harness is shared by the import and uncategorized-queue test
/// files, which have no need for the other resource routes that
/// `api_tests.rs`'s own `setup()` mounts.
pub async fn setup(rules: Vec<csv_import::Rule>) -> TestApp {
    let pool = db::create_pool("sqlite::memory:")
        .await
        .expect("failed to create in-memory pool");
    db::run_migrations(&pool).await.expect("failed to run migrations");

    let category_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
    let month_repo = Arc::new(SqliteMonthRepository::new(pool.clone()));
    let entry_repo = Arc::new(SqliteBudgetEntryRepository::new(pool.clone()));
    let transaction_repo = Arc::new(SqliteTransactionRepository::new(pool.clone()));

    let category_service = Arc::new(CategoryService::new(category_repo.clone()));
    let month_service = Arc::new(MonthService::new(month_repo.clone(), entry_repo.clone()));
    let entry_service = Arc::new(EntryService::new(
        entry_repo.clone(),
        category_repo.clone(),
        month_repo.clone(),
    ));
    let transaction_service = Arc::new(TransactionService::new(
        transaction_repo.clone(),
        entry_repo.clone(),
    ));
    let summary_service = Arc::new(SummaryService::new(
        entry_repo.clone(),
        transaction_repo.clone(),
        month_repo.clone(),
    ));

    let rules_path = std::env::temp_dir().join(format!(
        "otter_api_test_rules_{}.toml",
        ulid::Ulid::new()
    ));
    if !rules.is_empty() {
        csv_import::save_rules(&rules_path, &rules).expect("failed to write test rules file");
    }

    let import_deps = Arc::new(ImportDeps {
        month_repo: month_repo.clone(),
        month_service: month_service.clone(),
        category_service: category_service.clone(),
        entry_service: entry_service.clone(),
        transaction_repo: transaction_repo.clone(),
        transaction_service: transaction_service.clone(),
        rules_path,
        inline_rules: Vec::new(),
    });

    let state = AppState {
        category_service: category_service.clone(),
        month_service: month_service.clone(),
        entry_service: entry_service.clone(),
        transaction_service: transaction_service.clone(),
        summary_service,
        currency_config: otter::config::CurrencyConfig {
            code: "PLN".to_string(),
            minor_unit_name: "grosz".to_string(),
            decimal_places: 2,
        },
        import_deps,
    };

    let api = Router::new()
        .route("/imports", post(otter::handlers::imports::import_transactions))
        .route(
            "/transactions-review",
            get(otter::handlers::uncategorized::get_uncategorized_queue),
        )
        .route(
            "/transactions-review/preview",
            post(otter::handlers::uncategorized::preview_uncategorized_rule),
        )
        .route(
            "/transactions-review/rules",
            post(otter::handlers::uncategorized::save_uncategorized_rule),
        );
    let router = Router::new().nest("/api/v1", api).with_state(state.clone());

    TestApp {
        router,
        month_service,
        category_service,
        entry_service,
        transaction_service,
        import_deps: state.import_deps,
    }
}

/// Builds a `multipart/form-data` body with a single part named `"file"`,
/// optionally carrying a `Content-Type` header on that part (e.g. to
/// declare a charset), per RFC 2046's part-header syntax.
///
/// # Returns
///
/// `(boundary, body_bytes)` — the boundary is needed to build the
/// request's own `Content-Type` header.
fn build_multipart_body(filename: &str, part_content_type: Option<&str>, file_bytes: &[u8]) -> (&'static str, Vec<u8>) {
    const BOUNDARY: &str = "----otter-test-boundary";

    let mut body = Vec::new();
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n").as_bytes(),
    );
    if let Some(content_type) = part_content_type {
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
    }
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());

    (BOUNDARY, body)
}

/// POSTs `file_bytes` to `path` as a multipart upload in a part named
/// `"file"`, with filename `filename` and the given part-level
/// `Content-Type` (e.g. `Some("text/csv; charset=windows-1250")`, or
/// `None` to omit the header entirely).
///
/// # Returns
///
/// The response status and its body parsed as JSON (an empty object if
/// the body isn't valid JSON, mirroring `api_tests.rs`'s convention).
pub async fn post_file(
    router: &Router,
    path: &str,
    filename: &str,
    part_content_type: Option<&str>,
    file_bytes: &[u8],
) -> (StatusCode, Value) {
    let (boundary, body) = build_multipart_body(filename, part_content_type, file_bytes);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}
