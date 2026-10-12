//! Aggregation and output of categorized transactions into a per-category
//! breakdown, in both human-readable (`pretty`) and machine-readable
//! (`json`) formats.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::extract::ParsedTransaction;
use crate::rules::Category;
use domain::types::Money;

/// Per-category summary: total amount and transaction count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CategoryTotal {
    /// The category this total summarizes.
    pub category: Category,
    /// Sum of all transaction amounts in this category, in minor units.
    pub total: Money,
    /// Number of transactions in this category.
    pub count: usize,
}

/// A full per-category breakdown, ordered alphabetically by category name
/// for deterministic, testable output (rather than relying on hash-map
/// iteration order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Breakdown {
    /// Per-category totals, sorted alphabetically by category.
    pub by_category: Vec<CategoryTotal>,
}

/// Pairs a transaction with its assigned category for structured JSON
/// output, since a bare `(ParsedTransaction, Category)` tuple would
/// otherwise serialize as an unreadable 2-element array.
#[derive(Debug, Serialize)]
struct CategorizedTransaction<'a> {
    #[serde(flatten)]
    transaction: &'a ParsedTransaction,
    category: &'a Category,
}

/// Aggregates categorized transactions into per-category totals and counts.
/// Pure function, no I/O.
#[must_use]
pub fn aggregate(transactions: &[(ParsedTransaction, Category)]) -> Breakdown {
    let mut grouped: BTreeMap<&Category, Vec<Money>> = BTreeMap::new();
    for (transaction, category) in transactions {
        grouped.entry(category).or_default().push(transaction.amount);
    }

    let by_category = grouped
        .into_iter()
        .map(|(category, amounts)| CategoryTotal {
            category: category.clone(),
            count: amounts.len(),
            total: amounts.into_iter().sum(),
        })
        .collect();

    Breakdown { by_category }
}

/// Formats minor currency units (e.g. grosz) as a decimal string with 2
/// fractional places, e.g. `-10310` -> `"-103.10"`.
fn format_minor_units(minor_units: i64) -> String {
    let sign = if minor_units < 0 { "-" } else { "" };
    let magnitude = minor_units.abs();
    format!("{sign}{}.{:02}", magnitude / 100, magnitude % 100)
}

/// Renders a `Breakdown` as a simple, aligned-enough plain-text table. Pure
/// function returning a `String`, so it's testable without capturing
/// stdout; callers `println!` the result.
#[must_use]
pub fn render_pretty(breakdown: &Breakdown) -> String {
    let mut lines = vec![format!(
        "{:<24} {:>12} {:>8}",
        "Category", "Total", "Count"
    )];
    for entry in &breakdown.by_category {
        lines.push(format!(
            "{:<24} {:>12} {:>8}",
            entry.category,
            format_minor_units(entry.total.value()),
            entry.count
        ));
    }
    lines.join("\n")
}

/// Renders a `Breakdown` alongside the full list of categorized
/// transactions as a JSON document with two top-level keys: `breakdown`
/// (per-category totals) and `transactions` (every transaction with its
/// extracted fields and assigned category).
///
/// # Errors
///
/// Returns a `serde_json::Error` if serialization fails (not expected in
/// practice: these types contain no maps with non-string keys or cycles).
pub fn render_json(
    breakdown: &Breakdown,
    transactions: &[(ParsedTransaction, Category)],
) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct Document<'a> {
        breakdown: &'a [CategoryTotal],
        transactions: Vec<CategorizedTransaction<'a>>,
    }

    let document = Document {
        breakdown: &breakdown.by_category,
        transactions: transactions
            .iter()
            .map(|(transaction, category)| CategorizedTransaction {
                transaction,
                category,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&document)
}

/// Prints the categorized breakdown to stdout in the requested format.
///
/// # Errors
///
/// Returns a `serde_json::Error` (via [`crate::CsvImportError`]) if
/// `format` is [`OutputFormat::Json`] and serialization fails.
pub fn print_breakdown(
    breakdown: &Breakdown,
    transactions: &[(ParsedTransaction, Category)],
    format: OutputFormat,
) -> Result<(), crate::CsvImportError> {
    match format {
        OutputFormat::Pretty => println!("{}", render_pretty(breakdown)),
        OutputFormat::Json => println!("{}", render_json(breakdown, transactions)?),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::check;
    use chrono::NaiveDate;

    /// Builds a minimal `ParsedTransaction` fixture with a given amount; the
    /// rest of the fields are irrelevant to aggregation/rendering tests.
    fn transaction(amount: i64) -> ParsedTransaction {
        ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
            amount: Money::new(amount),
            currency: "PLN".to_string(),
            raw_type: "Płatność kartą".to_string(),
            card_last4: None,
            merchant: None,
            city: None,
            country: None,
            title: None,
            raw_description: String::new(),
        }
    }

    /// Multiple transactions across 2+ categories, including negative
    /// amounts, should sum correctly per category and count correctly.
    #[test]
    fn aggregates_multiple_categories_with_negative_amounts() {
        let transactions = vec![
            (transaction(-100), "groceries".to_string()),
            (transaction(-50), "groceries".to_string()),
            (transaction(2000), "salary".to_string()),
        ];

        let breakdown = aggregate(&transactions);

        check!(breakdown.by_category.len() == 2);
        check!(breakdown.by_category[0].category == "groceries");
        check!(breakdown.by_category[0].total == Money::new(-150));
        check!(breakdown.by_category[0].count == 2);
        check!(breakdown.by_category[1].category == "salary");
        check!(breakdown.by_category[1].total == Money::new(2000));
        check!(breakdown.by_category[1].count == 1);
    }

    /// A single category with multiple transactions aggregates into one
    /// entry with the summed total and correct count.
    #[test]
    fn aggregates_single_category() {
        let transactions = vec![
            (transaction(-10), "fuel".to_string()),
            (transaction(-20), "fuel".to_string()),
        ];

        let breakdown = aggregate(&transactions);

        check!(breakdown.by_category.len() == 1);
        check!(breakdown.by_category[0].total == Money::new(-30));
        check!(breakdown.by_category[0].count == 2);
    }

    /// Aggregating an empty slice yields an empty breakdown, not a panic.
    #[test]
    fn aggregates_empty_input() {
        let breakdown = aggregate(&[]);
        check!(breakdown.by_category.is_empty());
    }

    /// Negative, positive and whole-number minor-unit values all format as
    /// the expected 2-decimal-place string.
    #[test]
    fn formats_minor_units_as_decimal_string() {
        check!(format_minor_units(-10310) == "-103.10");
        check!(format_minor_units(10310) == "103.10");
        check!(format_minor_units(2000) == "20.00");
        check!(format_minor_units(0) == "0.00");
    }

    /// The pretty table contains each category's name and formatted total.
    #[test]
    fn renders_pretty_table_with_category_and_total() {
        let transactions = vec![
            (transaction(-10310), "groceries".to_string()),
            (transaction(500000), "salary".to_string()),
        ];
        let breakdown = aggregate(&transactions);

        let output = render_pretty(&breakdown);

        check!(output.contains("groceries"));
        check!(output.contains("-103.10"));
        check!(output.contains("salary"));
        check!(output.contains("5000.00"));
    }

    /// The JSON output is valid, parseable JSON exposing both the
    /// `breakdown` summary and the full `transactions` list.
    #[test]
    fn renders_valid_json_with_breakdown_and_transactions() {
        let transactions = vec![(transaction(-10310), "groceries".to_string())];
        let breakdown = aggregate(&transactions);

        let json = render_json(&breakdown, &transactions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        check!(parsed["breakdown"][0]["category"] == "groceries");
        check!(parsed["breakdown"][0]["total"] == -10310);
        check!(parsed["breakdown"][0]["count"] == 1);
        check!(parsed["transactions"][0]["category"] == "groceries");
        check!(parsed["transactions"][0]["amount"] == -10310);
    }
}
