//! Interactive learning flow: when a transaction matches no existing rule,
//! prompt the user for a category and learn a new rule from their answer so
//! future, similar transactions categorize automatically.
//!
//! The learned rule's pattern is stored *raw* (as extracted from the
//! transaction), not pre-normalized/uppercased — `match_rule` already
//! normalizes both sides of a comparison at match time (see `rules.rs`), so
//! storing a normalized pattern here would gain nothing while making the
//! hand-editable TOML file uglier to read.

use std::io::{self, Write as _};
use std::path::Path;

use crate::extract::ParsedTransaction;
use crate::rules::{
    categorize, save_rules, Category, Matchable, MatchCriteria, MatchCriterion, MatchField,
    MatchType, Rule,
};

/// Derives a new rule that matches `transaction` and assigns it `category`.
///
/// Generic over [`Matchable`] (not just [`ParsedTransaction`]) so the
/// bulk-categorize API (section 3 of the `bulk-categorize-transactions`
/// change) can derive a starting rule from an already-persisted
/// transaction's stored `ImportDetails` too, via its own `Matchable`
/// wrapper — this function's logic is identical either way, it only reads
/// `merchant`/`raw_type`.
///
/// Mirrors `categorize`'s matching semantics: prefers a `Contains` pattern
/// on `merchant` (substring match) when the transaction has a merchant,
/// since that's the common card-payment case; falls back to a `Contains`
/// match on `raw_type` when there's no merchant (e.g. a transfer row),
/// since that's the only field such rows reliably carry. Pure function, no
/// I/O.
#[must_use]
pub fn derive_rule<T: Matchable>(transaction: &T, category: &Category) -> Rule {
    let criterion = match transaction.merchant() {
        Some(merchant) => MatchCriterion {
            field: MatchField::Merchant,
            match_type: MatchType::Contains(merchant.to_string()),
        },
        None => MatchCriterion {
            field: MatchField::RawType,
            // `raw_type()` is always present on a `ParsedTransaction`, but
            // `Matchable` only guarantees `Option<&str>` in general (e.g. an
            // `ImportDetails` whose `raw_type` was never captured) — fall
            // back to an empty pattern, which matches nothing, rather than
            // panicking on a `None`.
            match_type: MatchType::Contains(transaction.raw_type().unwrap_or_default().to_string()),
        },
    };
    Rule {
        pattern: MatchCriteria(vec![criterion]),
        category: category.clone(),
    }
}

/// Prints `transaction`'s extracted fields and reads a category from stdin.
///
/// Only present fields are printed. Empty input is accepted as a literal
/// empty-string category rather than re-prompting — this is a prototype CLI
/// pausing mid-run for one person's own bank statement, not a public-facing
/// form, so a retry loop isn't worth building.
///
/// # Errors
///
/// Returns an `io::Error` if reading the line from stdin fails.
pub fn prompt_for_category(transaction: &ParsedTransaction) -> io::Result<Category> {
    print_transaction_fields(transaction);
    print!("Category: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

/// Prints the extracted fields of `transaction` in a human-readable form,
/// skipping fields that are absent.
fn print_transaction_fields(transaction: &ParsedTransaction) {
    println!("No rule matched this transaction:");
    println!("  date:     {}", transaction.date);
    println!("  amount:   {}", transaction.amount.value());
    if let Some(merchant) = &transaction.merchant {
        println!("  merchant: {merchant}");
    }
    if let Some(city) = &transaction.city {
        println!("  city:     {city}");
    }
    if let Some(country) = &transaction.country {
        println!("  country:  {country}");
    }
    if let Some(card_last4) = &transaction.card_last4 {
        println!("  card:     ****{card_last4}");
    }
    if let Some(title) = &transaction.title {
        println!("  title:    {title}");
    }
    println!("  raw_type: {}", transaction.raw_type);
}

/// Categorizes `transaction`, learning a new rule interactively if no
/// existing rule matches.
///
/// Tries `categorize` first. On a miss: if `interactive` is `false`, returns
/// `"Uncategorized"` without touching `rules` or `rules_file` at all. If
/// `interactive` is `true`, prompts for a category, derives a rule via
/// [`derive_rule`], appends it to `rules` (so later transactions in the same
/// run benefit immediately without re-prompting for the same pattern), and
/// persists the full updated rule set back to `rules_file`.
///
/// # Errors
///
/// Returns an `io::Error` if prompting stdin fails, or if persisting the
/// updated rules file fails.
pub fn categorize_or_learn(
    transaction: &ParsedTransaction,
    rules: &mut Vec<Rule>,
    rules_file: &Path,
    interactive: bool,
) -> io::Result<Category> {
    if let Some(category) = categorize(transaction, rules) {
        return Ok(category);
    }

    if !interactive {
        return Ok("Uncategorized".to_string());
    }

    let category = prompt_for_category(transaction)?;
    let rule = derive_rule(transaction, &category);
    rules.push(rule);
    save_rules(rules_file, rules).map_err(io::Error::other)?;
    Ok(category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};
    use chrono::NaiveDate;
    use domain::types::Money;

    /// Builds a minimal `ParsedTransaction` fixture with just the fields a
    /// test needs set; the rest default to `None`/empty.
    fn transaction(merchant: Option<&str>, raw_type: &str) -> ParsedTransaction {
        ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
            amount: Money::new(-100),
            currency: "PLN".to_string(),
            raw_type: raw_type.to_string(),
            card_last4: None,
            merchant: merchant.map(str::to_string),
            city: None,
            country: None,
            title: None,
            raw_description: String::new(),
            original_amount_fragment: None,
        }
    }

    /// With a merchant present, `derive_rule` builds a `merchant_contains`
    /// rule, and that rule actually matches the same transaction via
    /// `categorize` (not just field equality).
    #[test]
    fn derives_merchant_contains_rule_that_matches() {
        let txn = transaction(Some("NETTO 5567 K.2"), "Płatność kartą");
        let category = "groceries".to_string();

        let rule = derive_rule(&txn, &category);
        check!(
            rule.pattern.0
                == vec![MatchCriterion {
                    field: MatchField::Merchant,
                    match_type: MatchType::Contains("NETTO 5567 K.2".to_string()),
                }]
        );

        check!(categorize(&txn, &[rule]) == Some(category));
    }

    /// With no merchant (the transfer case), `derive_rule` falls back to an
    /// exact `raw_type` rule, which also actually matches via `categorize`.
    #[test]
    fn derives_raw_type_fallback_rule_that_matches() {
        let txn = transaction(None, "Autooszczędzanie");
        let category = "savings".to_string();

        let rule = derive_rule(&txn, &category);
        check!(
            rule.pattern.0
                == vec![MatchCriterion {
                    field: MatchField::RawType,
                    match_type: MatchType::Contains("Autooszczędzanie".to_string()),
                }]
        );

        check!(categorize(&txn, &[rule]) == Some(category));
    }

    /// Writing rules to disk and reloading them yields an equal rule set.
    #[test]
    fn save_rules_round_trips_through_load_rules() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "csv_import_learn_test_roundtrip_{}.toml",
            std::process::id()
        ));

        let rules = vec![
            derive_rule(&transaction(Some("NETTO"), "Płatność kartą"), &"groceries".to_string()),
            derive_rule(&transaction(None, "Autooszczędzanie"), &"savings".to_string()),
        ];

        save_rules(&path, &rules).unwrap();
        let_assert!(Ok(reloaded) = crate::rules::load_rules(&path));
        std::fs::remove_file(&path).unwrap();

        check!(reloaded == rules);
    }

    /// When `interactive` is `false`, an unmatched transaction is assigned
    /// `Uncategorized` without appending to `rules` or touching the rules
    /// file (we simply never call `save_rules` on this path, so there's
    /// nothing to assert on disk beyond the in-memory rules being
    /// unmodified).
    #[test]
    fn no_interactive_assigns_uncategorized_without_learning() {
        let txn = transaction(Some("UNKNOWN SHOP"), "Płatność kartą");
        let mut rules: Vec<Rule> = vec![];
        let path = Path::new("/nonexistent/should-not-be-touched.toml");

        let_assert!(Ok(category) = categorize_or_learn(&txn, &mut rules, path, false));
        check!(category == "Uncategorized".to_string());
        check!(rules.is_empty());
    }

    /// When a rule already matches, `categorize_or_learn` short-circuits:
    /// no prompting, no learning, existing rules untouched.
    #[test]
    fn existing_match_short_circuits_without_learning() {
        let txn = transaction(Some("NETTO 5567 K.2"), "Płatność kartą");
        let mut rules = vec![derive_rule(&txn, &"groceries".to_string())];
        let rules_before = rules.clone();
        let path = Path::new("/nonexistent/should-not-be-touched.toml");

        let_assert!(Ok(category) = categorize_or_learn(&txn, &mut rules, path, true));
        check!(category == "groceries".to_string());
        check!(rules == rules_before);
    }
}
