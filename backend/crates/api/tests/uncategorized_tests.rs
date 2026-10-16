//! Integration tests for the "uncategorized" transaction queue endpoints
//! (tasks 3.1 and 3.2 of the `bulk-categorize-transactions` OpenSpec
//! change), exercised against a real in-memory SQLite database through the
//! actual HTTP router.
//!
//! `GET /api/v1/transactions-review` coverage: an empty queue, a
//! head transaction with a merchant (derives a `merchant` rule), a head
//! transaction with only a `raw_type` (derives a `raw_type` rule), and a
//! head transaction with no import metadata at all (no derived rule).
//!
//! `POST /api/v1/transactions-review/preview` coverage: broader vs.
//! narrower draft rules, the empty-criteria vacuous match, a `regex`
//! criterion end-to-end, rejection of a malformed criterion, and (per the
//! `match-preview-table` change) the optional `page`/`exclude_transaction_id`
//! fields — count-only responses staying byte-for-byte unchanged, a page's
//! `other_count`/`transactions`, exclusion affecting only the page/other
//! count, and paging past the end.
//!
//! `POST /api/v1/transactions-review/rules` coverage (task 3.3):
//! saving a rule persists it and re-categorizes exactly the matching
//! transactions, a later CSV import picks up the saved rule from the same
//! shared rules file, and an invalid regex/category is rejected without
//! touching the rules file or the queue.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use chrono::NaiveDate;
use domain::entities::ImportDetails;
use domain::types::{BudgetMonth, CategoryName, Money, TransactionDate};

use common::{post_file, setup};

/// GETs `/api/v1/transactions-review` and returns the parsed JSON
/// body alongside the status code.
async fn get_queue(router: &axum::Router) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/transactions-review")
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

/// Creates (or reuses) the "uncategorized" category's budget entry for
/// October 2026, and a transaction under it with the given `import_details`,
/// mirroring how the import pipeline provisions its rows.
async fn seed_uncategorized_transaction(app: &common::TestApp, import_details: Option<ImportDetails>) {
    let month = app
        .month_service
        .create(BudgetMonth::new(2026, 10).unwrap(), None, true)
        .await
        .unwrap();
    let category = app
        .category_service
        .create(CategoryName::new("uncategorized").unwrap(), None)
        .await
        .unwrap();
    let entry = app
        .entry_service
        .create(month.id, category.id, Money::new(0), None)
        .await
        .unwrap();

    app.transaction_service
        .create(
            entry.id,
            Money::new(-500),
            TransactionDate::new(NaiveDate::from_ymd_opt(2026, 10, 9).unwrap()),
            None,
            import_details,
            vec![],
        )
        .await
        .unwrap();
}

/// Minimal `ImportDetails` fixture with only the given `merchant`/`raw_type`
/// set; all other optional fields are `None`.
fn import_details(merchant: Option<&str>, raw_type: Option<&str>) -> ImportDetails {
    ImportDetails {
        source: "csv_import".to_string(),
        merchant: merchant.map(str::to_string),
        city: None,
        country: None,
        card_last4: None,
        raw_type: raw_type.map(str::to_string),
        currency: None,
        raw_description: None,
        original_amount_fragment: None,
        raw_fragments: None,
    }
}

/// An empty queue reports zero remaining and a null head.
#[tokio::test]
async fn empty_queue_reports_zero_remaining_and_no_head() {
    let app = setup(vec![]).await;

    let (status, body) = get_queue(&app.router).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["remaining"], 0);
    assert!(body["head"].is_null());
}

/// A head transaction with a merchant in its import details derives a
/// single `merchant` `contains` criterion.
#[tokio::test]
async fn head_with_merchant_derives_merchant_rule() {
    let app = setup(vec![]).await;
    seed_uncategorized_transaction(&app, Some(import_details(Some("NETTO 5567"), None))).await;

    let (status, body) = get_queue(&app.router).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["remaining"], 1);
    let derived_rule = &body["head"]["derived_rule"];
    assert_eq!(derived_rule.as_array().unwrap().len(), 1);
    assert_eq!(derived_rule[0]["field"], "merchant");
    assert_eq!(derived_rule[0]["contains"], "NETTO 5567");
}

/// A head transaction with only a `raw_type` (no merchant) derives a
/// single `raw_type` `contains` criterion.
#[tokio::test]
async fn head_with_only_raw_type_derives_raw_type_rule() {
    let app = setup(vec![]).await;
    seed_uncategorized_transaction(&app, Some(import_details(None, Some("Autooszczędzanie")))).await;

    let (status, body) = get_queue(&app.router).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["remaining"], 1);
    let derived_rule = &body["head"]["derived_rule"];
    assert_eq!(derived_rule.as_array().unwrap().len(), 1);
    assert_eq!(derived_rule[0]["field"], "raw_type");
    assert_eq!(derived_rule[0]["contains"], "Autooszczędzanie");
}

/// A head transaction with no import details at all (manually created) has
/// a null `derived_rule`.
#[tokio::test]
async fn head_without_import_details_has_no_derived_rule() {
    let app = setup(vec![]).await;
    seed_uncategorized_transaction(&app, None).await;

    let (status, body) = get_queue(&app.router).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["remaining"], 1);
    assert!(body["head"]["derived_rule"].is_null());
}

/// Creates (or reuses) the "uncategorized" category's budget entry for
/// October 2026 and returns its id, so a test can seed several transactions
/// under the same entry without re-creating the month/category/entry (which
/// would error on the second call).
async fn uncategorized_entry_id(app: &common::TestApp) -> ulid::Ulid {
    let month = app
        .month_service
        .create(BudgetMonth::new(2026, 10).unwrap(), None, true)
        .await
        .unwrap();
    let category = app
        .category_service
        .create(CategoryName::new("uncategorized").unwrap(), None)
        .await
        .unwrap();
    let entry = app
        .entry_service
        .create(month.id, category.id, Money::new(0), None)
        .await
        .unwrap();
    entry.id
}

/// Creates a transaction with the given `import_details` under the
/// "uncategorized" entry identified by `entry_id`, for seeding multiple
/// transactions into the same preview-test queue.
async fn seed_transaction_under(app: &common::TestApp, entry_id: ulid::Ulid, import_details: Option<ImportDetails>) {
    app.transaction_service
        .create(
            entry_id,
            Money::new(-500),
            TransactionDate::new(NaiveDate::from_ymd_opt(2026, 10, 9).unwrap()),
            None,
            import_details,
            vec![],
        )
        .await
        .unwrap();
}

/// POSTs `/api/v1/transactions-review/preview` with the given raw
/// JSON `body` and returns the parsed response alongside the status code.
async fn preview_rule(router: &axum::Router, body: Value) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/transactions-review/preview")
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

/// Builds a `{ "criteria": [...] }` preview request body from
/// `(field, contains)` pairs.
fn criteria_body(criteria: Vec<(&str, &str)>) -> Value {
    serde_json::json!({
        "criteria": criteria
            .into_iter()
            .map(|(field, contains)| serde_json::json!({ "field": field, "contains": contains }))
            .collect::<Vec<_>>(),
    })
}

/// A broader draft rule (matching several seeded merchants) returns a
/// higher count than a narrower one (matching only a subset), and an empty
/// criteria list matches every transaction that has import metadata
/// (vacuous AND).
#[tokio::test]
async fn preview_counts_matches_by_specificity() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567 K.2"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 1234 K.9"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("ZABKA 42"), None))).await;
    seed_transaction_under(&app, entry_id, None).await;

    let (status, body) = preview_rule(&app.router, criteria_body(vec![("merchant", "netto")])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 2);

    let (status, body) = preview_rule(&app.router, criteria_body(vec![("merchant", "NETTO 5567")])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);

    let (status, body) = preview_rule(&app.router, serde_json::json!({ "criteria": [] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 3); // the 3 with import_details, not the one without
}

/// With no `page` in the request, the response body is byte-for-byte the
/// count-only shape from before pagination existed: exactly `{"count":N}`,
/// with no `other_count`/`transactions` keys at all.
#[tokio::test]
async fn preview_without_page_is_byte_for_byte_count_only() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567"), None))).await;

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/transactions-review/preview")
                .header("content-type", "application/json")
                .body(Body::from(criteria_body(vec![("merchant", "netto")]).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(bytes.as_ref(), br#"{"count":1}"#);
}

/// With a `page` in the request, the response additionally includes
/// `other_count` and `transactions`, while `count` still reports the
/// overall total unaffected by `exclude_transaction_id`.
#[tokio::test]
async fn preview_with_page_returns_other_count_and_transactions() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567 K.2"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 1234 K.9"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 42 ZZ"), None))).await;

    let mut body = criteria_body(vec![("merchant", "netto")]);
    body["page"] = serde_json::json!({ "offset": 0, "limit": 20 });
    let (status, body) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 3);
    assert_eq!(body["other_count"], 3);
    assert_eq!(body["transactions"].as_array().unwrap().len(), 3);
}

/// `exclude_transaction_id` drops that transaction from `other_count` and
/// the page, but leaves the overall `count` untouched.
#[tokio::test]
async fn preview_excludes_transaction_from_other_count_and_page_only() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567 K.2"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 1234 K.9"), None))).await;

    let (_, queue_body) = get_queue(&app.router).await;
    let excluded_id = queue_body["head"]["transaction"]["id"].as_str().unwrap().to_string();

    let mut body = criteria_body(vec![("merchant", "netto")]);
    body["page"] = serde_json::json!({ "offset": 0, "limit": 20 });
    body["exclude_transaction_id"] = serde_json::json!(excluded_id);
    let (status, body) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 2, "overall count is unaffected by exclusion");
    assert_eq!(body["other_count"], 1);
    let transactions = body["transactions"].as_array().unwrap();
    assert_eq!(transactions.len(), 1);
    assert_ne!(transactions[0]["id"], excluded_id);
}

/// Requesting a `page` with an `offset` past the number of matches returns
/// an empty `transactions` list, not an error.
#[tokio::test]
async fn preview_page_past_the_end_is_empty() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567"), None))).await;

    let mut body = criteria_body(vec![("merchant", "netto")]);
    body["page"] = serde_json::json!({ "offset": 10, "limit": 20 });
    let (status, body) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["other_count"], 1);
    assert!(body["transactions"].as_array().unwrap().is_empty());
}

/// A `regex` criterion is honored end-to-end through the API, not just
/// `contains`.
#[tokio::test]
async fn preview_supports_regex_criterion() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567 K.2"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("ZABKA 42"), None))).await;

    let body = serde_json::json!({
        "criteria": [{ "field": "merchant", "regex": r"^NETTO \d+" }],
    });
    let (status, body) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);
}

/// A `raw_fragments` criterion is honored end-to-end through the API: a
/// transaction whose `ImportDetails.raw_fragments` contains the pattern is
/// matched, and one without a `raw_fragments` value is not.
#[tokio::test]
async fn preview_supports_raw_fragments_criterion() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    let mut with_fragments = import_details(Some("NETTO 5567 K.2"), None);
    with_fragments.raw_fragments = Some("REF123 | EXTRA NOTE".to_string());
    seed_transaction_under(&app, entry_id, Some(with_fragments)).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("ZABKA 42"), None))).await;

    let body = serde_json::json!({
        "criteria": [{ "field": "raw_fragments", "contains": "extra note" }],
    });
    let (status, body) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);
}

/// A criterion setting neither `contains` nor `regex` is rejected as a bad
/// request rather than silently matching nothing or everything.
#[tokio::test]
async fn preview_rejects_criterion_missing_match_type() {
    let app = setup(vec![]).await;

    let body = serde_json::json!({ "criteria": [{ "field": "merchant" }] });
    let (status, _) = preview_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// POSTs `/api/v1/transactions-review/rules` with the given raw JSON
/// `body` and returns the parsed response alongside the status code.
async fn save_rule(router: &axum::Router, body: Value) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/transactions-review/rules")
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

/// Builds a `{ "criteria": [...], "category": ... }` save-rule request body
/// from `(field, contains)` pairs.
fn save_rule_body(criteria: Vec<(&str, &str)>, category: &str) -> Value {
    serde_json::json!({
        "criteria": criteria
            .into_iter()
            .map(|(field, contains)| serde_json::json!({ "field": field, "contains": contains }))
            .collect::<Vec<_>>(),
        "category": category,
    })
}

/// Saving a rule persists it to the rules file, re-categorizes exactly the
/// transactions it matches onto a budget entry under the new category, and
/// leaves non-matching transactions untouched.
#[tokio::test]
async fn saving_rule_persists_and_recategorizes_matches() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567 K.2"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 1234 K.9"), None))).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("ZABKA 42"), None))).await;

    let (status, body) = save_rule(
        &app.router,
        save_rule_body(vec![("merchant", "netto")], "groceries"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["remaining"], 1); // only the ZABKA one is left uncategorized

    let rules = csv_import::load_rules(&app.import_deps.rules_path).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].category, "groceries");

    let remaining = app
        .import_deps
        .transaction_repo
        .list_by_category_name("uncategorized")
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(
        remaining[0].import_details.as_ref().unwrap().merchant,
        Some("ZABKA 42".to_string())
    );

    app.category_service
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.name.as_str() == "groceries")
        .expect("groceries category was auto-created by the recategorization");
}

/// A rule saved via the API is picked up by a later CSV import through the
/// same shared rules file, without reconstructing `ImportDeps`.
#[tokio::test]
async fn saved_rule_applies_to_later_import() {
    const FIXTURE_BYTES: &[u8] =
        include_bytes!("../../csv_import/tests/fixtures/sample_export.csv");

    let app = setup(vec![]).await;

    let (status, _) = save_rule(
        &app.router,
        save_rule_body(vec![("merchant", "netto")], "groceries"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = post_file(
        &app.router,
        "/api/v1/imports",
        "sample_export.csv",
        Some("text/csv; charset=windows-1250"),
        FIXTURE_BYTES,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["created_categories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c == "groceries"));
    assert_eq!(body["uncategorized"], 11); // 12 rows minus the 1 NETTO row
}

/// A `regex` criterion that fails to compile is rejected with a `400`,
/// leaving the rules file and the queue untouched.
#[tokio::test]
async fn saving_rule_rejects_uncompilable_regex() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567"), None))).await;

    let body = serde_json::json!({
        "criteria": [{ "field": "merchant", "regex": "(unterminated" }],
        "category": "groceries",
    });
    let (status, _) = save_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!app.import_deps.rules_path.exists());

    let (_, queue_body) = get_queue(&app.router).await;
    assert_eq!(queue_body["remaining"], 1);
}

/// An invalid (empty) category name is rejected with a `400`, leaving the
/// rules file and the queue untouched.
#[tokio::test]
async fn saving_rule_rejects_invalid_category() {
    let app = setup(vec![]).await;
    let entry_id = uncategorized_entry_id(&app).await;
    seed_transaction_under(&app, entry_id, Some(import_details(Some("NETTO 5567"), None))).await;

    let body = save_rule_body(vec![("merchant", "netto")], "");
    let (status, _) = save_rule(&app.router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!app.import_deps.rules_path.exists());

    let (_, queue_body) = get_queue(&app.router).await;
    assert_eq!(queue_body["remaining"], 1);
}
