//! Resolution of a CSV import row's category name, with the
//! `"uncategorized"` fallback the spec requires when rule-matching doesn't
//! produce a usable category.

use csv_import::{ParsedTransaction, Rule};
use domain::types::CategoryName;

/// The fixed category every transaction falls back to when no rule
/// matches, or when the matched rule's category isn't a usable name.
pub const UNCATEGORIZED: &str = "uncategorized";

/// Builds the fallback `("uncategorized", true)` result. A `const` literal,
/// so the `expect` can never actually fail — it exists only because
/// `CategoryName::new` has a fallible signature.
fn uncategorized() -> (CategoryName, bool) {
    (
        CategoryName::new(UNCATEGORIZED).expect("the fixed literal \"uncategorized\" is always a valid category name"),
        true,
    )
}

/// Resolves the category name for `transaction` by running it against
/// `rules` (first-rule-wins, via `csv_import::categorize`), falling back to
/// [`UNCATEGORIZED`] when no rule matches or the matched rule's category
/// string fails [`CategoryName`] validation (e.g. it's empty).
///
/// # Returns
///
/// The resolved category name, and whether it fell back to
/// `"uncategorized"` (used by the pipeline for the `uncategorized` count).
#[must_use]
pub fn resolve_category_name(transaction: &ParsedTransaction, rules: &[Rule]) -> (CategoryName, bool) {
    match csv_import::categorize(transaction, rules) {
        Some(name) => match CategoryName::new(name) {
            Ok(valid) => (valid, false),
            Err(_) => uncategorized(),
        },
        None => uncategorized(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use csv_import::MatchCriteria;
    use domain::types::Money;

    /// Minimal `ParsedTransaction` fixture with just the fields a rule
    /// might match on; the rest default to empty/`None`.
    fn transaction(merchant: Option<&str>) -> ParsedTransaction {
        ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
            amount: Money::new(-100),
            currency: "PLN".to_string(),
            raw_type: "Płatność kartą".to_string(),
            card_last4: None,
            merchant: merchant.map(str::to_string),
            city: None,
            country: None,
            title: None,
            raw_description: String::new(),
        }
    }

    fn rule(merchant_contains: &str, category: &str) -> Rule {
        Rule {
            pattern: MatchCriteria {
                merchant_contains: Some(merchant_contains.to_string()),
                ..Default::default()
            },
            category: category.to_string(),
        }
    }

    /// A transaction matching a configured rule resolves to that rule's
    /// category, not uncategorized.
    #[test]
    fn resolves_to_matched_rule_category() {
        let txn = transaction(Some("NETTO 5567"));
        let rules = vec![rule("netto", "groceries")];

        let (name, fell_back) = resolve_category_name(&txn, &rules);
        assert_eq!(name.as_str(), "groceries");
        assert!(!fell_back);
    }

    /// A transaction matching no rule falls back to uncategorized.
    #[test]
    fn falls_back_to_uncategorized_when_no_rule_matches() {
        let txn = transaction(Some("UNKNOWN SHOP"));
        let rules = vec![rule("netto", "groceries")];

        let (name, fell_back) = resolve_category_name(&txn, &rules);
        assert_eq!(name.as_str(), UNCATEGORIZED);
        assert!(fell_back);
    }

    /// A transaction matching a rule whose category is empty (invalid as a
    /// `CategoryName`) falls back to uncategorized, the same as no match.
    #[test]
    fn falls_back_to_uncategorized_when_matched_category_is_invalid() {
        let txn = transaction(Some("NETTO 5567"));
        let rules = vec![rule("netto", "")];

        let (name, fell_back) = resolve_category_name(&txn, &rules);
        assert_eq!(name.as_str(), UNCATEGORIZED);
        assert!(fell_back);
    }

    /// No rules at all is the same as no match.
    #[test]
    fn falls_back_to_uncategorized_with_no_rules() {
        let txn = transaction(Some("NETTO 5567"));
        let (name, fell_back) = resolve_category_name(&txn, &[]);
        assert_eq!(name.as_str(), UNCATEGORIZED);
        assert!(fell_back);
    }
}
