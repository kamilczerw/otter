//! Integration tests for `POST /api/v1/imports` (section 6 of the
//! `csv-backend-import` OpenSpec change), exercised against a real
//! in-memory SQLite database through the actual HTTP router — not just
//! the pipeline's unit tests, which stop at `import_one_transaction` and
//! don't go through multipart parsing or the HTTP response mapping.
//!
//! Covers every scenario in
//! `openspec/changes/csv-backend-import/specs/transaction-import/spec.md`:
//! successful import + categorization, duplicate re-import, partial
//! per-row failure, wholesale structural rejection, and charset handling.

mod common;

use axum::http::StatusCode;
use csv_import::{MatchCriteria, MatchCriterion, MatchField, MatchType, Rule};

use common::{post_file, setup};

/// Path to the bank CSV fixture shared with the `csv_import` crate's own
/// tests: 12 data rows (3 transfers, 7 card payments, 2 more transfers),
/// Windows-1250 encoded.
const FIXTURE_BYTES: &[u8] = include_bytes!("../../csv_import/tests/fixtures/sample_export.csv");

/// A rule matching the fixture's one NETTO card payment row, categorizing
/// it as `"groceries"` instead of the `"uncategorized"` fallback every
/// other row in the fixture falls to.
fn netto_rule() -> Rule {
    Rule {
        pattern: MatchCriteria(vec![MatchCriterion {
            field: MatchField::Merchant,
            match_type: MatchType::Contains("netto".to_string()),
        }]),
        category: "groceries".to_string(),
    }
}

/// Uploads `FIXTURE_BYTES` as Windows-1250 (its real encoding), the
/// charset every other test in this file uses unless it's specifically
/// testing charset handling.
async fn post_fixture(router: &axum::Router) -> (StatusCode, serde_json::Value) {
    post_file(
        router,
        "/api/v1/imports",
        "sample_export.csv",
        Some("text/csv; charset=windows-1250"),
        FIXTURE_BYTES,
    )
    .await
}

/// 6.1: importing the fixture with a rule matching one of its rows
/// persists all 12 rows, categorizing the one matched row under
/// `"groceries"` and the rest under `"uncategorized"`, and the response
/// counts agree with what's actually in the database.
#[tokio::test]
async fn imports_fixture_and_categorizes_via_rules() {
    let app = setup(vec![netto_rule()]).await;

    let (status, body) = post_fixture(&app.router).await;

    assert_eq!(status, StatusCode::CREATED, "response body: {body}");
    assert_eq!(body["imported"], 12);
    assert_eq!(body["duplicates"], 0);
    assert_eq!(body["uncategorized"], 11);
    assert_eq!(body["failed"].as_array().unwrap().len(), 0);
    assert_eq!(body["created_months"].as_array().unwrap().len(), 1);

    let created_categories: Vec<&str> = body["created_categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(created_categories.len(), 2);
    assert!(created_categories.contains(&"groceries"));
    assert!(created_categories.contains(&"uncategorized"));

    // Verify against the database directly, not just the response JSON.
    let months = app.month_service.list_all().await.unwrap();
    assert_eq!(months.len(), 1, "expected exactly one month created");

    let categories = app.category_service.list_all().await.unwrap();
    let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(categories.len(), 2);
    assert!(category_names.contains(&"groceries"));
    assert!(category_names.contains(&"uncategorized"));

    let entries = app.entry_service.list_by_month(&months[0].id).await.unwrap();
    assert_eq!(entries.len(), 2, "one entry per category in the month");

    let groceries_entry = entries
        .iter()
        .find(|e| e.category.name.as_str() == "groceries")
        .expect("groceries entry should exist");
    let uncategorized_entry = entries
        .iter()
        .find(|e| e.category.name.as_str() == "uncategorized")
        .expect("uncategorized entry should exist");

    let groceries_txns = app
        .transaction_service
        .list_by_entry(&groceries_entry.id, 100, 0)
        .await
        .unwrap();
    assert_eq!(groceries_txns.len(), 1, "only the NETTO row matches the rule");
    assert_eq!(groceries_txns[0].amount.value(), -10310);

    let uncategorized_txns = app
        .transaction_service
        .list_by_entry(&uncategorized_entry.id, 100, 0)
        .await
        .unwrap();
    assert_eq!(uncategorized_txns.len(), 11);
}

/// 6.2: re-importing the exact same file reports every row as a
/// duplicate, imports nothing new, and responds `200` (not `201`, since
/// nothing was newly persisted).
#[tokio::test]
async fn reimporting_same_file_reports_all_duplicates() {
    let app = setup(vec![netto_rule()]).await;

    let (first_status, _) = post_fixture(&app.router).await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (status, body) = post_fixture(&app.router).await;

    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["imported"], 0);
    assert_eq!(body["duplicates"], 12);

    // No new rows should have landed in the database from the re-import.
    let months = app.month_service.list_all().await.unwrap();
    assert_eq!(months.len(), 1);
    let categories = app.category_service.list_all().await.unwrap();
    assert_eq!(categories.len(), 2);
}

/// 6.3: one deliberately corrupted row (unparseable amount) mixed into
/// otherwise-valid fixture rows still allows the request to succeed as a
/// whole: the bad row is reported in `failed` with its position, raw
/// content, and reason, while the remaining 11 rows are imported, both
/// per the response counts and in the database.
#[tokio::test]
async fn partial_failure_reports_bad_row_and_imports_the_rest() {
    let app = setup(vec![]).await;

    // Corrupt the first data row's amount, mirroring csv_import's own
    // "collects_per_row_extraction_failures" unit test.
    let text = csv_import::decode_windows1250(FIXTURE_BYTES);
    let corrupted = text.replacen("-2.86", "not-a-number", 1);

    let (status, body) = post_file(
        &app.router,
        "/api/v1/imports",
        "corrupted.csv",
        None, // UTF-8: `corrupted` is already decoded to a Rust String.
        corrupted.as_bytes(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "response body: {body}");
    assert_eq!(body["imported"], 11);
    assert_eq!(body["duplicates"], 0);

    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["row"], 1);
    assert!(
        failed[0]["raw"].as_str().unwrap().contains("not-a-number"),
        "raw content should preserve the corrupted value: {failed:?}"
    );
    assert!(
        failed[0]["reason"].as_str().unwrap().contains("amount"),
        "reason should name the amount field: {failed:?}"
    );

    // The 11 good rows actually landed in the database.
    let categories = app.category_service.list_all().await.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].name.as_str(), "uncategorized");

    let months = app.month_service.list_all().await.unwrap();
    let entries = app.entry_service.list_by_month(&months[0].id).await.unwrap();
    assert_eq!(entries.len(), 1);
    let txns = app
        .transaction_service
        .list_by_entry(&entries[0].id, 100, 0)
        .await
        .unwrap();
    assert_eq!(txns.len(), 11);
}

/// 6.4: content that isn't structurally shaped as CSV at all (wrong
/// header column count — a `CsvParseError::UnexpectedHeader`, not a
/// per-row failure) is rejected wholesale as `400`, with nothing
/// persisted at all.
#[tokio::test]
async fn structurally_invalid_csv_is_rejected_with_nothing_persisted() {
    let app = setup(vec![]).await;

    let (status, body) = post_file(
        &app.router,
        "/api/v1/imports",
        "garbage.csv",
        None,
        b"\"a\",\"b\",\"c\"\n",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "response body: {body}");
    assert_eq!(body["error"]["code"], "BAD_REQUEST");

    assert!(app.month_service.list_all().await.unwrap().is_empty());
    assert!(app.category_service.list_all().await.unwrap().is_empty());
}

/// 6.5a: a file declared `charset=windows-1250` is decoded as such and
/// imports successfully — the fixture is genuinely Windows-1250 encoded,
/// so a wrong decode would corrupt its Polish diacritics and, for rows
/// whose required fields happen to depend on them, could surface as
/// extraction failures.
#[tokio::test]
async fn charset_windows_1250_decodes_and_imports_successfully() {
    let app = setup(vec![]).await;

    let (status, body) = post_fixture(&app.router).await;

    assert_eq!(status, StatusCode::CREATED, "response body: {body}");
    assert_eq!(body["imported"], 12);
    assert_eq!(body["failed"].as_array().unwrap().len(), 0);
}

/// 6.5b: a file uploaded with no charset declared at all is decoded as
/// UTF-8 by default, per the spec's "upload with no charset declared"
/// scenario.
#[tokio::test]
async fn no_charset_decodes_as_utf8_by_default() {
    let app = setup(vec![]).await;

    let csv = "\"Data operacji\",\"Data waluty\",\"Typ transakcji\",\"Kwota\",\"Waluta\",\
        \"Saldo po transakcji\",\"Opis transakcji\",\"\",\"\",\"\",\"\",\"\",\"\"\n\
        \"2026-10-11\",\"2026-10-09\",\"Autooszczędzanie\",\"-2.86\",\"PLN\",\"\",\
        \"AUTOOSZCZĘDZANIE-PRZELEW\",\"\",\"\",\"\",\"\",\"\",\"\"\n";

    let (status, body) = post_file(
        &app.router,
        "/api/v1/imports",
        "utf8.csv",
        None, // no Content-Type at all on the part -> no declared charset
        csv.as_bytes(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "response body: {body}");
    assert_eq!(body["imported"], 1);
    assert_eq!(body["failed"].as_array().unwrap().len(), 0);
}

/// 6.5c: a declared charset this module has no decoder for is rejected as
/// `400`, per the spec's "upload with unsupported charset" scenario.
#[tokio::test]
async fn unsupported_charset_is_rejected() {
    let app = setup(vec![]).await;

    let (status, body) = post_file(
        &app.router,
        "/api/v1/imports",
        "sample_export.csv",
        Some("text/csv; charset=shift-jis"),
        FIXTURE_BYTES,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "response body: {body}");
    assert_eq!(body["error"]["code"], "BAD_REQUEST");

    assert!(app.month_service.list_all().await.unwrap().is_empty());
}
