//! Integration tests for `GET /api/v1/uncategorized-transactions` (task 3.1
//! of the `bulk-categorize-transactions` OpenSpec change), exercised against
//! a real in-memory SQLite database through the actual HTTP router.
//!
//! Covers: an empty queue, a head transaction with a merchant (derives a
//! `merchant` rule), a head transaction with only a `raw_type` (derives a
//! `raw_type` rule), and a head transaction with no import metadata at all
//! (no derived rule).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use chrono::NaiveDate;
use domain::entities::ImportDetails;
use domain::types::{BudgetMonth, CategoryName, Money, TransactionDate};

use common::setup;

/// GETs `/api/v1/uncategorized-transactions` and returns the parsed JSON
/// body alongside the status code.
async fn get_queue(router: &axum::Router) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/uncategorized-transactions")
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
