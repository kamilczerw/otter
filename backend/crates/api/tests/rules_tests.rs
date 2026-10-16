//! Integration tests for the rule-editor capability's HTTP endpoints:
//! `GET/POST /api/v1/rules`, `PUT/DELETE /api/v1/rules/{id}`, and `POST
//! /api/v1/rules/import`. Mirrors `api_tests.rs`'s harness pattern (fresh
//! in-memory SQLite database, hand-assembled `Router`).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, put};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use db::repos::{
    SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
    SqliteTransactionRepository,
};
use db::MatchingRuleRepo;
use domain::services::{CategoryService, EntryService, MonthService, SummaryService, TransactionService};

use otter::handlers::AppState;

/// Builds a test [`Router`] mounting only the `/rules` endpoints, backed by
/// a fresh in-memory SQLite database with `inline_rules` seeded as the
/// config-defined default rules.
async fn setup(inline_rules: Vec<csv_import::Rule>) -> Router {
    let pool = db::create_pool("sqlite::memory:")
        .await
        .expect("failed to create in-memory pool");
    db::run_migrations(&pool).await.expect("failed to run migrations");

    let category_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
    let month_repo = Arc::new(SqliteMonthRepository::new(pool.clone()));
    let entry_repo = Arc::new(SqliteBudgetEntryRepository::new(pool.clone()));
    let transaction_repo = Arc::new(SqliteTransactionRepository::new(pool.clone()));
    let rule_repo: Arc<dyn domain::ports::MatchingRuleRepository> =
        Arc::new(MatchingRuleRepo::new(pool.clone()));

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

    let import_deps = Arc::new(otter::import::ImportDeps {
        month_repo: month_repo.clone(),
        month_service: month_service.clone(),
        category_service: category_service.clone(),
        entry_service: entry_service.clone(),
        transaction_repo: transaction_repo.clone(),
        transaction_service: transaction_service.clone(),
        rule_repo,
        inline_rules,
    });

    let state = AppState {
        category_service,
        month_service,
        entry_service,
        transaction_service,
        summary_service,
        currency_config: otter::config::CurrencyConfig {
            code: "PLN".to_string(),
            minor_unit_name: "grosz".to_string(),
            decimal_places: 2,
        },
        import_deps,
    };

    let api = Router::new()
        .route(
            "/rules",
            get(otter::handlers::rules::list_rules).post(otter::handlers::rules::create_rule),
        )
        .route(
            "/rules/{id}",
            put(otter::handlers::rules::update_rule).delete(otter::handlers::rules::delete_rule),
        )
        .route("/rules/import", axum::routing::post(otter::handlers::rules::import_rules));

    Router::new().nest("/api/v1", api).with_state(state)
}

/// Sends a JSON GET request and returns the response status and parsed body.
async fn do_get(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

/// Sends a JSON request with the given method/body and returns the response
/// status and parsed body.
async fn do_json(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

/// Sends a request with no body (e.g. `DELETE`) and returns the response
/// status and parsed body.
async fn do_no_body(app: &Router, method: &str, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
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

/// Builds a minimal valid create/update request body with one `Contains`
/// criterion, so tests don't repeat the JSON shape.
fn rule_body(category: &str) -> Value {
    json!({
        "criteria": [{ "field": "merchant", "contains": "netto" }],
        "category": category,
    })
}

#[tokio::test]
async fn create_rule_succeeds_and_appears_last_in_listing() {
    let app = setup(Vec::new()).await;

    let (status, body) = do_json(&app, "POST", "/api/v1/rules", rule_body("groceries")).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["category"], "groceries");
    assert_eq!(body["editable"], true);
    assert!(body["id"].is_string());

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    let rules = listing.as_array().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["category"], "groceries");
}

#[tokio::test]
async fn listing_shows_defaults_before_user_rules() {
    let default_rule = csv_import::Rule {
        pattern: csv_import::MatchCriteria(vec![csv_import::MatchCriterion {
            field: csv_import::MatchField::Merchant,
            match_type: csv_import::MatchType::Contains("zabka".to_string()),
        }]),
        category: "snacks".to_string(),
    };
    let app = setup(vec![default_rule]).await;

    do_json(&app, "POST", "/api/v1/rules", rule_body("groceries")).await;

    let (status, listing) = do_get(&app, "/api/v1/rules").await;
    assert_eq!(status, StatusCode::OK);
    let rules = listing.as_array().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0]["category"], "snacks");
    assert_eq!(rules[0]["editable"], false);
    assert!(rules[0]["id"].is_null());
    assert_eq!(rules[1]["category"], "groceries");
    assert_eq!(rules[1]["editable"], true);
}

#[tokio::test]
async fn edit_updates_criteria_and_category_in_place() {
    let app = setup(Vec::new()).await;

    let (_, created) = do_json(&app, "POST", "/api/v1/rules", rule_body("groceries")).await;
    let id = created["id"].as_str().unwrap();

    let (status, updated) = do_json(
        &app,
        "PUT",
        &format!("/api/v1/rules/{id}"),
        rule_body("utilities"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["category"], "utilities");
    assert_eq!(updated["id"], id);
}

#[tokio::test]
async fn edit_of_unknown_id_returns_404() {
    let app = setup(Vec::new()).await;

    let unknown_id = ulid::Ulid::new().to_string();
    let (status, body) = do_json(
        &app,
        "PUT",
        &format!("/api/v1/rules/{unknown_id}"),
        rule_body("groceries"),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "MATCHING_RULE_NOT_FOUND");
}

#[tokio::test]
async fn delete_removes_rule_from_listing() {
    let app = setup(Vec::new()).await;

    let (_, created) = do_json(&app, "POST", "/api/v1/rules", rule_body("groceries")).await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = do_no_body(&app, "DELETE", &format!("/api/v1/rules/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    assert!(listing.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn delete_of_unknown_id_returns_404() {
    let app = setup(Vec::new()).await;

    let unknown_id = ulid::Ulid::new().to_string();
    let (status, body) = do_no_body(&app, "DELETE", &format!("/api/v1/rules/{unknown_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "MATCHING_RULE_NOT_FOUND");
}

#[tokio::test]
async fn create_with_invalid_regex_returns_400() {
    let app = setup(Vec::new()).await;

    let body = json!({
        "criteria": [{ "field": "merchant", "regex": "(unclosed" }],
        "category": "groceries",
    });
    let (status, response) = do_json(&app, "POST", "/api/v1/rules", body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["error"]["code"], "MATCHING_RULE_INVALID_REGEX");

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    assert!(listing.as_array().unwrap().is_empty());
}

/// Builds a `multipart/form-data` body with a single part named `"file"`
/// carrying `toml_contents`, mirroring the shared harness in
/// `tests/common/mod.rs` (not reused directly since it's scoped to CSV
/// uploads' filename/content-type conventions, not a `.toml` rules file).
fn build_multipart_toml_body(toml_contents: &str) -> (&'static str, Vec<u8>) {
    const BOUNDARY: &str = "----otter-rules-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"rules.toml\"\r\n\r\n",
    );
    body.extend_from_slice(toml_contents.as_bytes());
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    (BOUNDARY, body)
}

/// POSTs `toml_contents` to `/api/v1/rules/import` as a multipart upload.
async fn post_rules_file(app: &Router, toml_contents: &str) -> (StatusCode, Value) {
    let (boundary, body) = build_multipart_toml_body(toml_contents);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rules/import")
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

const VALID_TOML: &str = "[[matching_rules]]\ncategory = \"groceries\"\n[[matching_rules.match]]\nfield = \"merchant\"\ncontains = \"netto\"\n";

#[tokio::test]
async fn import_all_new_rules_are_stored() {
    let app = setup(Vec::new()).await;

    let (status, body) = post_rules_file(&app, VALID_TOML).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["imported"], 1);
    assert_eq!(body["skipped"], 0);
    assert!(body["failed"].as_array().unwrap().is_empty());

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    assert_eq!(listing.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn import_skips_exact_duplicate_of_existing_user_rule() {
    let app = setup(Vec::new()).await;

    do_json(&app, "POST", "/api/v1/rules", rule_body("groceries")).await;

    let (status, body) = post_rules_file(&app, VALID_TOML).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["imported"], 0);
    assert_eq!(body["skipped"], 1);

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    assert_eq!(listing.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn import_reports_invalid_regex_rule_and_still_imports_rest() {
    let app = setup(Vec::new()).await;

    let toml = format!(
        "{VALID_TOML}\n[[matching_rules]]\ncategory = \"bad\"\n[[matching_rules.match]]\nfield = \"merchant\"\nregex = \"(unclosed\"\n"
    );

    let (status, body) = post_rules_file(&app, &toml).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["imported"], 1);
    assert_eq!(body["skipped"], 0);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["category"], "bad");

    let (_, listing) = do_get(&app, "/api/v1/rules").await;
    assert_eq!(listing.as_array().unwrap().len(), 1);
}
