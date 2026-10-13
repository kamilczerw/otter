//! Rule-based categorization of [`ParsedTransaction`]s against a
//! user-maintained, hand-editable TOML rules file.
//!
//! The rules file is a TOML document with a top-level `matching_rules` array
//! of tables (see [`RulesFile`]) — TOML has no bare top-level array, so this
//! wrapper key is the simplest shape that's still pleasant to hand-edit, per
//! the "hand-editable" goal in the design doc. Matching is first-rule-wins:
//! rules are tried in file order and the first one whose specified `match`
//! fields all match wins.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::extract::ParsedTransaction;

/// A transaction category. A plain string alias rather than an enum/newtype:
/// per the design doc, any string is a valid category, there's no fixed
/// registry to enforce.
pub type Category = String;

/// A single categorization rule: if all specified `match` fields match a
/// transaction, it is assigned `category`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// The match criteria for this rule. Renamed from `pattern` in Rust
    /// because `match` is a reserved keyword.
    #[serde(rename = "match")]
    pub pattern: MatchCriteria,
    /// The category assigned to transactions matching this rule.
    pub category: Category,
}

/// The `match:` block of a [`Rule`]. All fields are optional; a rule matches
/// a transaction when every field it specifies matches (AND semantics).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchCriteria {
    /// Case/diacritic-insensitive *substring* match against the
    /// transaction's `merchant` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merchant_contains: Option<String>,
    /// Case/diacritic-insensitive *exact* match against the transaction's
    /// `city` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Case/diacritic-insensitive *exact* match against the transaction's
    /// `country` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Case/diacritic-insensitive *exact* match against the transaction's
    /// `raw_type` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_type: Option<String>,
}

/// On-disk shape of the rules file: a TOML document with a single
/// `matching_rules` array of tables.
///
/// TOML has no syntax for a bare top-level array (unlike YAML's `- item`
/// list), so every TOML rules document needs a named top-level key; this
/// wrapper exists purely to give `Vec<Rule>` that key. Also shared with the
/// `[transaction.import]` section of the backend's `config.toml`, so the
/// standalone rules file and the inline config block parse via the exact
/// same type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesFile {
    /// The rules themselves, in first-match-wins file order.
    pub matching_rules: Vec<Rule>,
}

/// Errors from loading a rules file.
#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    /// The rules file couldn't be read (e.g. it doesn't exist, or
    /// permissions deny access).
    ///
    /// Section 5 (interactive learning) will decide whether a missing file
    /// on first run should instead start from an empty rule set; this error
    /// variant implements loading an *existing* file only, as scoped.
    #[error("failed to read rules file {path:?}: {source}")]
    Io {
        /// Path that failed to read.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The rules file's contents didn't parse as valid TOML matching the
    /// [`RulesFile`] schema.
    #[error("failed to parse rules file as TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),

    /// The parsed rules couldn't be serialized back to TOML (practically
    /// never hit — see the `ponytail` note on [`save_rules`]).
    #[error("failed to serialize rules as TOML: {0}")]
    SerializeToml(#[from] toml::ser::Error),

    /// The rules file couldn't be written back to disk (e.g. permissions
    /// deny access, or the parent directory doesn't exist).
    #[error("failed to write rules file {path:?}: {source}")]
    Write {
        /// Path that failed to write.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

/// Strips the 9 Polish diacritics (both cases) and uppercases the result, so
/// rule patterns and transaction field values can be compared
/// case/diacritic-insensitively.
///
/// Pure function; no locale/Unicode-normalization crate needed since the
/// known diacritic set is small and fixed.
pub fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ą' | 'Ą' => 'A',
            'ć' | 'Ć' => 'C',
            'ę' | 'Ę' => 'E',
            'ł' | 'Ł' => 'L',
            'ń' | 'Ń' => 'N',
            'ó' | 'Ó' => 'O',
            'ś' | 'Ś' => 'S',
            'ź' | 'Ź' => 'Z',
            'ż' | 'Ż' => 'Z',
            other => other,
        })
        .collect::<String>()
        .to_uppercase()
}

/// Checks whether `pattern` is a normalized substring of `value`.
fn normalized_contains(value: &str, pattern: &str) -> bool {
    normalize(value).contains(&normalize(pattern))
}

/// Checks whether `pattern` normalized-equals `value`.
fn normalized_eq(value: &str, pattern: &str) -> bool {
    normalize(value) == normalize(pattern)
}

/// Checks a single optional match criterion against an optional transaction
/// field, using the given comparison function. Matches trivially (`true`)
/// when the rule doesn't specify this field; fails when the rule specifies
/// it but the transaction field is absent.
fn field_matches(
    criterion: Option<&String>,
    transaction_field: Option<&String>,
    compare: impl Fn(&str, &str) -> bool,
) -> bool {
    match criterion {
        None => true,
        Some(pattern) => transaction_field.is_some_and(|value| compare(value, pattern)),
    }
}

/// Checks whether every `match` field specified on `rule` matches
/// `transaction`. Pure function, no I/O.
#[must_use]
pub fn match_rule(transaction: &ParsedTransaction, rule: &Rule) -> bool {
    let criteria = &rule.pattern;
    field_matches(
        criteria.merchant_contains.as_ref(),
        transaction.merchant.as_ref(),
        normalized_contains,
    ) && field_matches(
        criteria.city.as_ref(),
        transaction.city.as_ref(),
        normalized_eq,
    ) && field_matches(
        criteria.country.as_ref(),
        transaction.country.as_ref(),
        normalized_eq,
    ) && criteria
        .raw_type
        .as_ref()
        .is_none_or(|pattern| normalized_eq(&transaction.raw_type, pattern))
}

/// Categorizes `transaction` against `rules`, first-rule-wins: returns the
/// `category` of the first rule in file order whose `match` fields all
/// match, or `None` if no rule matches. Pure function, no I/O.
#[must_use]
pub fn categorize(transaction: &ParsedTransaction, rules: &[Rule]) -> Option<Category> {
    rules
        .iter()
        .find(|rule| match_rule(transaction, rule))
        .map(|rule| rule.category.clone())
}

/// Loads and parses an existing rules file into a list of [`Rule`]s.
///
/// The TOML shape is a `matching_rules` array of tables (see [`RulesFile`]),
/// e.g.:
///
/// ```toml
/// [[matching_rules]]
/// category = "groceries"
/// [matching_rules.match]
/// merchant_contains = "netto"
/// ```
///
/// # Errors
///
/// Returns [`RulesError::Io`] if the file can't be read (including if it
/// doesn't exist), or [`RulesError::InvalidToml`] if its contents don't
/// parse as a [`RulesFile`].
pub fn load_rules(path: &Path) -> Result<Vec<Rule>, RulesError> {
    let contents = std::fs::read_to_string(path).map_err(|source| RulesError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let rules_file: RulesFile = toml::from_str(&contents)?;
    Ok(rules_file.matching_rules)
}

/// Loads the rules file at `path`, or starts from an empty rule set if it
/// doesn't exist yet (e.g. the very first run of a fresh import, before any
/// rule has been learned). Delegates to [`load_rules`] for an existing file,
/// so parse errors on a malformed existing file are still reported.
///
/// # Errors
///
/// Returns [`RulesError::InvalidYaml`] if an existing file's contents don't
/// parse as a list of [`Rule`]s, or [`RulesError::Io`] if the file exists
/// but can't be read.
pub fn load_rules_or_empty(path: &Path) -> Result<Vec<Rule>, RulesError> {
    if path.exists() {
        load_rules(path)
    } else {
        Ok(Vec::new())
    }
}

/// Serializes `rules` as TOML (wrapped in a [`RulesFile`]) and writes them
/// back to `path`, overwriting any existing contents. The write-side
/// counterpart to [`load_rules`], used to persist a newly learned rule.
///
/// # Errors
///
/// Returns [`RulesError::SerializeToml`] if the rules can't be serialized
/// (practically never — a `Vec<Rule>` has no maps with non-string keys and
/// no cycles), or [`RulesError::Write`] if the file can't be written (e.g.
/// permissions deny access).
pub fn save_rules(path: &Path, rules: &[Rule]) -> Result<(), RulesError> {
    let rules_file = RulesFile {
        matching_rules: rules.to_vec(),
    };
    let toml = toml::to_string_pretty(&rules_file)?;
    std::fs::write(path, toml).map_err(|source| RulesError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};
    use chrono::NaiveDate;
    use domain::types::Money;

    /// Builds a minimal `ParsedTransaction` fixture with just the fields a
    /// test needs set; the rest default to `None`/empty.
    fn transaction(
        merchant: Option<&str>,
        city: Option<&str>,
        country: Option<&str>,
        raw_type: &str,
    ) -> ParsedTransaction {
        ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
            amount: Money::new(-100),
            currency: "PLN".to_string(),
            raw_type: raw_type.to_string(),
            card_last4: None,
            merchant: merchant.map(str::to_string),
            city: city.map(str::to_string),
            country: country.map(str::to_string),
            title: None,
            raw_description: String::new(),
            original_amount_fragment: None,
        }
    }

    fn rule(criteria: MatchCriteria, category: &str) -> Rule {
        Rule {
            pattern: criteria,
            category: category.to_string(),
        }
    }

    /// `normalize` strips Polish diacritics and uppercases, so e.g.
    /// "Cieszyn", "CIESZYN" and diacritic variants of "Łódź" all collapse to
    /// the same normalized form.
    #[test]
    fn normalizes_case_and_diacritics() {
        check!(normalize("Cieszyn") == normalize("CIESZYN"));
        check!(normalize("Łódź") == "LODZ");
        check!(normalize("łódź") == "LODZ");
    }

    /// `merchant_contains` is a substring match, not exact equality.
    #[test]
    fn matches_merchant_contains_as_substring() {
        let txn = transaction(Some("NETTO 5567 K.2"), None, None, "Płatność kartą");
        let r = rule(
            MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                ..Default::default()
            },
            "groceries",
        );
        check!(match_rule(&txn, &r));
    }

    /// `city`/`country`/`raw_type` are exact (normalized) matches: a
    /// substring isn't enough.
    #[test]
    fn matches_city_country_raw_type_exactly() {
        let txn = transaction(None, Some("Cieszyn"), Some("Polska"), "Płatność kartą");

        let city_rule = rule(
            MatchCriteria {
                city: Some("CIESZYN".to_string()),
                ..Default::default()
            },
            "local",
        );
        check!(match_rule(&txn, &city_rule));

        let partial_city_rule = rule(
            MatchCriteria {
                city: Some("CIESZ".to_string()),
                ..Default::default()
            },
            "local",
        );
        check!(!match_rule(&txn, &partial_city_rule));

        let country_rule = rule(
            MatchCriteria {
                country: Some("POLSKA".to_string()),
                ..Default::default()
            },
            "domestic",
        );
        check!(match_rule(&txn, &country_rule));

        let type_rule = rule(
            MatchCriteria {
                raw_type: Some("platnosc karta".to_string()),
                ..Default::default()
            },
            "card",
        );
        check!(match_rule(&txn, &type_rule));
    }

    /// A rule requiring multiple fields only matches if all of them match;
    /// one mismatching field is enough to fail the whole rule.
    #[test]
    fn requires_all_specified_fields_to_match() {
        let txn = transaction(Some("NETTO"), Some("Cieszyn"), None, "Płatność kartą");
        let r = rule(
            MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                city: Some("WARSZAWA".to_string()),
                ..Default::default()
            },
            "groceries",
        );
        check!(!match_rule(&txn, &r));
    }

    /// A rule requiring a field the transaction doesn't have (`None`) can
    /// never match.
    #[test]
    fn absent_transaction_field_never_matches_a_specified_criterion() {
        let txn = transaction(None, None, None, "Autooszczędzanie");
        let r = rule(
            MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                ..Default::default()
            },
            "groceries",
        );
        check!(!match_rule(&txn, &r));
    }

    /// First-rule-wins: when an earlier broader rule and a later narrower
    /// rule both could match, the earlier rule's category is returned.
    #[test]
    fn first_matching_rule_wins() {
        let txn = transaction(Some("NETTO 5567 K.2"), Some("Cieszyn"), None, "Płatność kartą");
        let rules = vec![
            rule(
                MatchCriteria {
                    merchant_contains: Some("netto".to_string()),
                    ..Default::default()
                },
                "groceries",
            ),
            rule(
                MatchCriteria {
                    merchant_contains: Some("netto".to_string()),
                    city: Some("CIESZYN".to_string()),
                    ..Default::default()
                },
                "groceries-cieszyn",
            ),
        ];
        check!(categorize(&txn, &rules) == Some("groceries".to_string()));
    }

    /// When no rule matches, `categorize` returns `None`.
    #[test]
    fn no_match_returns_none() {
        let txn = transaction(Some("UNKNOWN SHOP"), None, None, "Płatność kartą");
        let rules = vec![rule(
            MatchCriteria {
                merchant_contains: Some("netto".to_string()),
                ..Default::default()
            },
            "groceries",
        )];
        check!(categorize(&txn, &rules) == None);
    }

    /// Loading a well-formed rules file deserializes it into the expected
    /// `Rule`s.
    #[test]
    fn loads_rules_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("csv_import_rules_test_{}.toml", std::process::id()));
        std::fs::write(
            &path,
            "[[matching_rules]]\ncategory = \"groceries\"\n[matching_rules.match]\nmerchant_contains = \"netto\"\n\n\
             [[matching_rules]]\ncategory = \"local\"\n[matching_rules.match]\ncity = \"Cieszyn\"\nraw_type = \"Płatność kartą\"\n",
        )
        .unwrap();

        let_assert!(Ok(rules) = load_rules(&path));
        std::fs::remove_file(&path).unwrap();

        check!(rules.len() == 2);
        check!(rules[0].category == "groceries");
        check!(rules[0].pattern.merchant_contains == Some("netto".to_string()));
        check!(rules[1].category == "local");
        check!(rules[1].pattern.city == Some("Cieszyn".to_string()));
    }

    /// Loading a file that doesn't exist is a clear `RulesError::Io`, not a
    /// panic.
    #[test]
    fn missing_rules_file_is_an_error() {
        let path = std::env::temp_dir().join("csv_import_rules_test_does_not_exist.toml");
        let_assert!(Err(RulesError::Io { .. }) = load_rules(&path));
    }

    /// Saving rules and reloading them round-trips through the TOML
    /// wrapper (`RulesFile`) without changing field values.
    #[test]
    fn save_rules_round_trips_through_toml() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "csv_import_rules_test_roundtrip_{}.toml",
            std::process::id()
        ));

        let rules = vec![
            rule(
                MatchCriteria {
                    merchant_contains: Some("netto".to_string()),
                    ..Default::default()
                },
                "groceries",
            ),
            rule(
                MatchCriteria {
                    city: Some("Cieszyn".to_string()),
                    ..Default::default()
                },
                "local",
            ),
        ];

        save_rules(&path, &rules).unwrap();
        let_assert!(Ok(reloaded) = load_rules(&path));
        std::fs::remove_file(&path).unwrap();

        check!(reloaded == rules);
    }
}
