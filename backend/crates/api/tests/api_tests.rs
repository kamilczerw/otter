use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, patch};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use db::repos::{
    SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
    SqliteTransactionRepository,
};
use domain::services::{
    CategoryService, EntryService, MonthService, SummaryService, TransactionService,
};

// Re-use the AppState from the api crate.
use api::handlers::AppState;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

async fn setup() -> Router {
    let pool = db::create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let category_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
    let month_repo = Arc::new(SqliteMonthRepository::new(pool.clone()));
    let entry_repo = Arc::new(SqliteBudgetEntryRepository::new(pool.clone()));
    let transaction_repo = Arc::new(SqliteTransactionRepository::new(pool.clone()));

    let category_service = Arc::new(CategoryService::new(category_repo.clone()));
    let month_service = Arc::new(MonthService::new(
        month_repo.clone(),
        entry_repo.clone(),
    ));
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

    let state = AppState {
        category_service,
        month_service,
        entry_service,
        transaction_service,
        summary_service,
        currency_config: api::config::CurrencyConfig {
            code: "PLN".to_string(),
            minor_unit_name: "grosz".to_string(),
            decimal_places: 2,
        },
    };

    let api = Router::new()
        .route("/health", get(api::handlers::health::health_check))
        .route(
            "/categories",
            get(api::handlers::categories::list_categories)
                .post(api::handlers::categories::create_category),
        )
        .route(
            "/categories/{id}",
            patch(api::handlers::categories::update_category),
        )
        .route(
            "/months",
            get(api::handlers::months::list_months).post(api::handlers::months::create_month),
        )
        .route("/months/{id}", get(api::handlers::months::get_month))
        .route(
            "/months/{id}/entries",
            get(api::handlers::entries::list_entries).post(api::handlers::entries::create_entry),
        )
        .route(
            "/months/{id}/entries/{entry_id}",
            patch(api::handlers::entries::update_entry).delete(api::handlers::entries::delete_entry),
        )
        .route(
            "/transactions",
            get(api::handlers::transactions::list_transactions)
                .post(api::handlers::transactions::create_transaction),
        )
        .route(
            "/transactions/{id}",
            patch(api::handlers::transactions::update_transaction)
                .delete(api::handlers::transactions::delete_transaction),
        )
        .route(
            "/months/{id}/summary",
            get(api::handlers::summary::get_month_summary),
        );

    Router::new().nest("/api/v1", api).with_state(state)
}

async fn do_get(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or_default();
    (status, json)
}

async fn do_post(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn do_patch(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn do_delete(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

// Convenience: create a category and return its id.
async fn create_category(app: &Router, name: &str) -> String {
    let (status, body) = do_post(app, "/api/v1/categories", json!({ "name": name })).await;
    assert_eq!(status, StatusCode::CREATED, "create category failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

// Convenience: create a month and return its id.
async fn create_month(app: &Router, month: &str) -> String {
    let (status, body) = do_post(app, "/api/v1/months", json!({ "month": month })).await;
    assert_eq!(status, StatusCode::CREATED, "create month failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

// Convenience: create an entry and return its id.
async fn create_entry(
    app: &Router,
    month_id: &str,
    category_id: &str,
    budgeted: i64,
    due_day: Option<u8>,
) -> String {
    let mut payload = json!({
        "category_id": category_id,
        "budgeted": budgeted,
    });
    if let Some(d) = due_day {
        payload["due_day"] = json!(d);
    }
    let path = format!("/api/v1/months/{month_id}/entries");
    let (status, body) = do_post(app, &path, payload).await;
    assert_eq!(status, StatusCode::CREATED, "create entry failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

// Convenience: create a transaction and return its id.
async fn create_transaction(
    app: &Router,
    entry_id: &str,
    amount: i64,
    date: &str,
) -> String {
    let payload = json!({
        "entry_id": entry_id,
        "amount": amount,
        "date": date,
    });
    let (status, body) = do_post(app, "/api/v1/transactions", payload).await;
    assert_eq!(status, StatusCode::CREATED, "create transaction failed: {body}");
    body["id"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_check() {
    let app = setup().await;
    let (status, body) = do_get(&app, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_create_and_list_categories() {
    let app = setup().await;

    // Create two categories
    let (status, body) = do_post(&app, "/api/v1/categories", json!({ "name": "food" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "food");

    let (status, body) =
        do_post(&app, "/api/v1/categories", json!({ "name": "utils/electricity" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "utils/electricity");

    // List categories
    let (status, body) = do_get(&app, "/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let categories = body.as_array().expect("expected array");
    assert_eq!(categories.len(), 2);

    // Should be sorted by name: food < utils/electricity
    assert_eq!(categories[0]["name"], "food");
    assert_eq!(categories[1]["name"], "utils/electricity");
}

#[tokio::test]
async fn test_duplicate_category_returns_409() {
    let app = setup().await;

    let (status, _) = do_post(&app, "/api/v1/categories", json!({ "name": "food" })).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = do_post(&app, "/api/v1/categories", json!({ "name": "food" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "CATEGORY_NAME_ALREADY_EXISTS");
}

#[tokio::test]
async fn test_invalid_category_name() {
    let app = setup().await;

    // Empty name
    let (status, body) = do_post(&app, "/api/v1/categories", json!({ "name": "" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "CATEGORY_INVALID_NAME");

    // Double-slash name
    let (status, body) = do_post(&app, "/api/v1/categories", json!({ "name": "a//b" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "CATEGORY_INVALID_NAME");
}

#[tokio::test]
async fn test_rename_category() {
    let app = setup().await;

    let cat_id = create_category(&app, "food").await;

    // Rename
    let path = format!("/api/v1/categories/{cat_id}");
    let (status, body) = do_patch(&app, &path, json!({ "name": "groceries" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "groceries");

    // Verify via list
    let (status, body) = do_get(&app, "/api/v1/categories").await;
    assert_eq!(status, StatusCode::OK);
    let categories = body.as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0]["name"], "groceries");
}

#[tokio::test]
async fn test_create_and_list_months() {
    let app = setup().await;

    let (status, body) = do_post(&app, "/api/v1/months", json!({ "month": "2026-02" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["month"], "2026-02");

    let (status, body) = do_get(&app, "/api/v1/months").await;
    assert_eq!(status, StatusCode::OK);
    let months = body.as_array().unwrap();
    assert_eq!(months.len(), 1);
    assert_eq!(months[0]["month"], "2026-02");
}

#[tokio::test]
async fn test_duplicate_month_returns_409() {
    let app = setup().await;

    let (status, _) = do_post(&app, "/api/v1/months", json!({ "month": "2026-02" })).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = do_post(&app, "/api/v1/months", json!({ "month": "2026-02" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "MONTH_ALREADY_EXISTS");
}

#[tokio::test]
async fn test_create_month_copies_entries() {
    let app = setup().await;

    // Set up category and January month with an entry
    let cat_id = create_category(&app, "food").await;
    let jan_id = create_month(&app, "2026-01").await;
    create_entry(&app, &jan_id, &cat_id, 10000, Some(15)).await;

    // Create February -- should auto-copy entries from January
    let feb_id = create_month(&app, "2026-02").await;

    // Verify February entries
    let path = format!("/api/v1/months/{feb_id}/entries");
    let (status, body) = do_get(&app, &path).await;
    assert_eq!(status, StatusCode::OK);
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["category"]["name"], "food");
    assert_eq!(entries[0]["budgeted"], 10000);
    assert_eq!(entries[0]["due_day"], 15);
}

#[tokio::test]
async fn test_budget_entry_crud() {
    let app = setup().await;

    let cat_id = create_category(&app, "rent").await;
    let month_id = create_month(&app, "2026-03").await;

    // Create entry
    let entry_id = create_entry(&app, &month_id, &cat_id, 5000, None).await;

    // List entries -> 1
    let entries_path = format!("/api/v1/months/{month_id}/entries");
    let (status, body) = do_get(&app, &entries_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Update entry budgeted amount
    let entry_path = format!("/api/v1/months/{month_id}/entries/{entry_id}");
    let (status, body) = do_patch(&app, &entry_path, json!({ "budgeted": 20000 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["budgeted"], 20000);

    // Delete entry
    let (status, _) = do_delete(&app, &entry_path).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // List entries -> 0
    let (status, body) = do_get(&app, &entries_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_cannot_delete_entry_with_transactions() {
    let app = setup().await;

    let cat_id = create_category(&app, "food").await;
    let month_id = create_month(&app, "2026-04").await;
    let entry_id = create_entry(&app, &month_id, &cat_id, 10000, None).await;

    // Create a transaction on this entry
    create_transaction(&app, &entry_id, 3000, "2026-04-10").await;

    // Attempt to delete the entry -- should fail
    let entry_path = format!("/api/v1/months/{month_id}/entries/{entry_id}");
    let (status, body) = do_delete(&app, &entry_path).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "ENTRY_HAS_TRANSACTIONS");
}

#[tokio::test]
async fn test_transaction_crud() {
    let app = setup().await;

    let cat_id = create_category(&app, "food").await;
    let month_id = create_month(&app, "2026-05").await;
    let entry_id = create_entry(&app, &month_id, &cat_id, 10000, None).await;

    // Create transaction
    let txn_id = create_transaction(&app, &entry_id, 2500, "2026-05-01").await;

    // List transactions for month
    let list_path = format!("/api/v1/transactions?month={month_id}");
    let (status, body) = do_get(&app, &list_path).await;
    assert_eq!(status, StatusCode::OK);
    let transactions = body.as_array().unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0]["amount"], 2500);

    // Update transaction amount
    let txn_path = format!("/api/v1/transactions/{txn_id}");
    let (status, body) = do_patch(&app, &txn_path, json!({ "amount": 4000 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["amount"], 4000);

    // Delete transaction
    let (status, _) = do_delete(&app, &txn_path).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // List transactions -> 0
    let (status, body) = do_get(&app, &list_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_month_summary() {
    let app = setup().await;

    let cat_id = create_category(&app, "food").await;
    let month_id = create_month(&app, "2026-02").await;

    // Add entry with budgeted = 10000
    let entry_id = create_entry(&app, &month_id, &cat_id, 10000, None).await;

    // Create transaction for 5000
    create_transaction(&app, &entry_id, 5000, "2026-02-15").await;

    // Get summary
    let summary_path = format!("/api/v1/months/{month_id}/summary");
    let (status, body) = do_get(&app, &summary_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_budgeted"], 10000);
    assert_eq!(body["total_paid"], 5000);
    assert_eq!(body["remaining"], 5000);

    // Category status
    let categories = body["categories"].as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0]["category"]["name"], "food");
    assert_eq!(categories[0]["budgeted"], 10000);
    assert_eq!(categories[0]["paid"], 5000);
    assert_eq!(categories[0]["remaining"], 5000);
    assert_eq!(categories[0]["status"], "underspent");
}

#[tokio::test]
async fn test_transactions_require_month_param() {
    let app = setup().await;

    let (status, body) = do_get(&app, "/api/v1/transactions").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "TRANSACTIONS_MONTH_REQUIRED");
}
