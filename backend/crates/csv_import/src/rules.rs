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

/// A transaction field a [`MatchCriterion`] can be checked against.
///
/// `Description` maps to [`ParsedTransaction::raw_description`] — named
/// `Description` here since that's the user-facing/TOML-facing name, even
/// though the underlying struct field is prefixed `raw_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchField {
    /// The transaction's merchant name.
    Merchant,
    /// The transaction's raw, bank-reported type string.
    RawType,
    /// The transaction's merchant city.
    City,
    /// The transaction's merchant country.
    Country,
    /// The transaction's raw description text.
    Description,
}

/// How a [`MatchCriterion`] compares its field's value against a pattern.
///
/// Serializes as a single TOML key (`contains = "..."` or `regex = "..."`)
/// flattened into the enclosing [`MatchCriterion`] table, so a hand-written
/// rule reads naturally as `field = "merchant"` + `contains = "netto"` in
/// the same `[[matching_rules.match]]` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Case/diacritic-insensitive substring match (via [`normalized_contains`]).
    Contains(String),
    /// Regex match against the raw (non-normalized) field value. A pattern
    /// that fails to compile is treated as never matching — see
    /// [`criterion_matches`].
    Regex(String),
}

/// One field check within a [`MatchCriteria`] list: "does `field` match
/// `match_type`'s pattern?".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchCriterion {
    /// The transaction field to check.
    pub field: MatchField,
    /// How to compare that field's value against the pattern.
    #[serde(flatten)]
    pub match_type: MatchType,
}

/// The `match:` block of a [`Rule`]: a list of [`MatchCriterion`]s, ANDed
/// together — a rule matches a transaction only when *every* criterion in
/// the list matches it. An empty list matches every transaction (vacuous
/// AND), which is intentional: it lets a rule be defined by `category`
/// alone during manual TOML editing/testing, though in practice
/// [`derive_rule`](crate::learn::derive_rule) always emits at least one
/// criterion.
///
/// A flat list of criteria (rather than one `Option<String>` struct field
/// per possible field/match-type combination) keeps matching a single loop
/// over a 2-case match type, instead of ~10 near-duplicate optional struct
/// fields that would need updating every time a new field or match type is
/// added.
///
/// `#[serde(transparent)]` makes this serialize/deserialize as exactly its
/// inner `Vec<MatchCriterion>` — i.e. a TOML array of tables — rather than
/// as a wrapping table with a single field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MatchCriteria(pub Vec<MatchCriterion>);

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

/// A transaction-like type whose fields [`MatchCriterion`]s can be checked
/// against. Lets `match_rule`/`categorize` stay generic instead of coupling
/// directly to `ParsedTransaction`, and keeps field lookup in one place
/// (one `match` on [`MatchField`]) rather than duplicated per caller.
pub trait Matchable {
    /// The merchant name, if present.
    fn merchant(&self) -> Option<&str>;
    /// The raw, bank-reported transaction type.
    fn raw_type(&self) -> Option<&str>;
    /// The merchant city, if present.
    fn city(&self) -> Option<&str>;
    /// The merchant country, if present.
    fn country(&self) -> Option<&str>;
    /// The raw description text.
    fn description(&self) -> Option<&str>;
}

impl Matchable for ParsedTransaction {
    fn merchant(&self) -> Option<&str> {
        self.merchant.as_deref()
    }

    fn raw_type(&self) -> Option<&str> {
        Some(self.raw_type.as_str())
    }

    fn city(&self) -> Option<&str> {
        self.city.as_deref()
    }

    fn country(&self) -> Option<&str> {
        self.country.as_deref()
    }

    fn description(&self) -> Option<&str> {
        Some(self.raw_description.as_str())
    }
}

/// Reads the transaction field named by `field` off `transaction`, via the
/// matching [`Matchable`] accessor. The single place that maps a
/// [`MatchField`] variant to its accessor, so adding a field means touching
/// one `match` arm here instead of every call site.
fn field_value<T: Matchable>(transaction: &T, field: MatchField) -> Option<&str> {
    match field {
        MatchField::Merchant => transaction.merchant(),
        MatchField::RawType => transaction.raw_type(),
        MatchField::City => transaction.city(),
        MatchField::Country => transaction.country(),
        MatchField::Description => transaction.description(),
    }
}

/// Checks whether `value` satisfies `match_type`'s pattern.
///
/// A `Regex` pattern that fails to compile is treated as non-matching
/// rather than erroring: validating regex patterns at save time is a
/// separate, later concern, and a match-time failure must not panic or
/// bubble an error through `categorize`'s otherwise-infallible signature.
fn match_type_matches(value: &str, match_type: &MatchType) -> bool {
    match match_type {
        MatchType::Contains(pattern) => normalized_contains(value, pattern),
        MatchType::Regex(pattern) => regex::Regex::new(pattern).is_ok_and(|re| re.is_match(value)),
    }
}

/// Checks a single [`MatchCriterion`] against `transaction`. The criterion
/// never matches if its field is absent on the transaction (e.g. a
/// `city` criterion against a transaction with no `city`).
fn criterion_matches<T: Matchable>(transaction: &T, criterion: &MatchCriterion) -> bool {
    field_value(transaction, criterion.field)
        .is_some_and(|value| match_type_matches(value, &criterion.match_type))
}

/// Checks whether every criterion in `rule`'s `match` list matches
/// `transaction`. Pure function, no I/O. An empty criteria list matches
/// vacuously (see [`MatchCriteria`]).
#[must_use]
pub fn match_rule<T: Matchable>(transaction: &T, rule: &Rule) -> bool {
    rule.pattern
        .0
        .iter()
        .all(|criterion| criterion_matches(transaction, criterion))
}

/// Categorizes `transaction` against `rules`, first-rule-wins: returns the
/// `category` of the first rule in file order whose `match` criteria all
/// match, or `None` if no rule matches. Pure function, no I/O.
#[must_use]
pub fn categorize<T: Matchable>(transaction: &T, rules: &[Rule]) -> Option<Category> {
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
/// [[matching_rules.match]]
/// field = "merchant"
/// contains = "netto"
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

    /// Builds a `Rule` from a list of `(field, match_type)` pairs and a
    /// category, so tests can write criteria tersely.
    fn rule(criteria: Vec<MatchCriterion>, category: &str) -> Rule {
        Rule {
            pattern: MatchCriteria(criteria),
            category: category.to_string(),
        }
    }

    /// Builds a `Contains` criterion for `field`.
    fn contains(field: MatchField, pattern: &str) -> MatchCriterion {
        MatchCriterion {
            field,
            match_type: MatchType::Contains(pattern.to_string()),
        }
    }

    /// Builds a `Regex` criterion for `field`.
    fn regex(field: MatchField, pattern: &str) -> MatchCriterion {
        MatchCriterion {
            field,
            match_type: MatchType::Regex(pattern.to_string()),
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

    /// A `Contains` criterion is a case/diacritic-insensitive substring
    /// match, not exact equality.
    #[test]
    fn matches_contains_as_substring() {
        let txn = transaction(Some("NETTO 5567 K.2"), None, None, "Płatność kartą");
        let r = rule(vec![contains(MatchField::Merchant, "netto")], "groceries");
        check!(match_rule(&txn, &r));
    }

    /// A `Regex` criterion matches the raw (non-normalized) field value
    /// against the compiled pattern.
    #[test]
    fn matches_regex_criterion() {
        let txn = transaction(Some("NETTO 5567 K.2"), None, None, "Płatność kartą");
        let matching = rule(vec![regex(MatchField::Merchant, r"^NETTO \d+")], "groceries");
        check!(match_rule(&txn, &matching));

        let non_matching = rule(vec![regex(MatchField::Merchant, r"^ZABKA")], "snacks");
        check!(!match_rule(&txn, &non_matching));
    }

    /// `Description` matches against `raw_description`, not `merchant`.
    #[test]
    fn matches_description_field() {
        let mut txn = transaction(None, None, None, "Przelew");
        txn.raw_description = "Tytuł: Czynsz za mieszkanie".to_string();

        let r = rule(vec![contains(MatchField::Description, "czynsz")], "rent");
        check!(match_rule(&txn, &r));
    }

    /// A rule requiring multiple criteria only matches if all of them
    /// match; one mismatching criterion is enough to fail the whole rule.
    #[test]
    fn requires_all_criteria_to_match() {
        let txn = transaction(Some("NETTO"), Some("Cieszyn"), None, "Płatność kartą");
        let r = rule(
            vec![
                contains(MatchField::Merchant, "netto"),
                contains(MatchField::City, "WARSZAWA"),
            ],
            "groceries",
        );
        check!(!match_rule(&txn, &r));
    }

    /// A criterion for a field the transaction doesn't have (`None`) can
    /// never match.
    #[test]
    fn absent_transaction_field_never_matches_a_criterion() {
        let txn = transaction(None, None, None, "Autooszczędzanie");
        let r = rule(vec![contains(MatchField::Merchant, "netto")], "groceries");
        check!(!match_rule(&txn, &r));
    }

    /// First-rule-wins: when an earlier broader rule and a later narrower
    /// rule both could match, the earlier rule's category is returned.
    #[test]
    fn first_matching_rule_wins() {
        let txn = transaction(Some("NETTO 5567 K.2"), Some("Cieszyn"), None, "Płatność kartą");
        let rules = vec![
            rule(vec![contains(MatchField::Merchant, "netto")], "groceries"),
            rule(
                vec![
                    contains(MatchField::Merchant, "netto"),
                    contains(MatchField::City, "CIESZYN"),
                ],
                "groceries-cieszyn",
            ),
        ];
        check!(categorize(&txn, &rules) == Some("groceries".to_string()));
    }

    /// When no rule matches, `categorize` returns `None`.
    #[test]
    fn no_match_returns_none() {
        let txn = transaction(Some("UNKNOWN SHOP"), None, None, "Płatność kartą");
        let rules = vec![rule(vec![contains(MatchField::Merchant, "netto")], "groceries")];
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
            "[[matching_rules]]\ncategory = \"groceries\"\n[[matching_rules.match]]\nfield = \"merchant\"\ncontains = \"netto\"\n\n\
             [[matching_rules]]\ncategory = \"local\"\n[[matching_rules.match]]\nfield = \"city\"\ncontains = \"Cieszyn\"\n[[matching_rules.match]]\nfield = \"raw_type\"\ncontains = \"Płatność kartą\"\n",
        )
        .unwrap();

        let_assert!(Ok(rules) = load_rules(&path));
        std::fs::remove_file(&path).unwrap();

        check!(rules.len() == 2);
        check!(rules[0].category == "groceries");
        check!(rules[0].pattern.0 == vec![contains(MatchField::Merchant, "netto")]);
        check!(rules[1].category == "local");
        check!(rules[1].pattern.0[0] == contains(MatchField::City, "Cieszyn"));
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
            rule(vec![contains(MatchField::Merchant, "netto")], "groceries"),
            rule(vec![contains(MatchField::City, "Cieszyn")], "local"),
        ];

        save_rules(&path, &rules).unwrap();
        let_assert!(Ok(reloaded) = load_rules(&path));
        std::fs::remove_file(&path).unwrap();

        check!(reloaded == rules);
    }
}
