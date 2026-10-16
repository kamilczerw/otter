//! A user-stored categorization rule: a set of match criteria plus a target
//! category, evaluated against transactions during CSV import exactly like
//! the config-defined default rules (see `csv_import::rules`). Domain
//! declares its own `MatchField`/`MatchType`/`MatchCriterion` shape here
//! (rather than depending on `csv_import`) per the hexagonal layering rule
//! that `domain` has zero framework/adapter dependencies; the `api` crate
//! converts between the two shapes.

use crate::errors::MatchingRuleError;

/// A transaction field a [`MatchCriterion`] can be checked against. Mirrors
/// `csv_import::rules::MatchField`'s variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// The transaction's joined, `" | "`-separated raw-fragments string.
    RawFragments,
}

/// How a [`MatchCriterion`] compares its field's value against a pattern.
/// Mirrors `csv_import::rules::MatchType`'s variants.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MatchType {
    /// Case/diacritic-insensitive substring match.
    Contains(String),
    /// Regex match against the raw field value. Unlike
    /// `csv_import::rules::MatchType::Regex`, construction validates that
    /// the pattern compiles (see [`MatchCriterion::new`]).
    Regex(String),
}

/// One field check within a [`MatchingRule`]'s criteria list: "does `field`
/// match `match_type`'s pattern?". Constructed via [`MatchCriterion::new`]
/// so an uncompilable regex pattern can never be represented.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatchCriterion {
    /// The transaction field to check.
    pub field: MatchField,
    /// How to compare that field's value against the pattern.
    pub match_type: MatchType,
}

impl MatchCriterion {
    /// Builds a [`MatchCriterion`], validating that a [`MatchType::Regex`]
    /// pattern compiles. A [`MatchType::Contains`] pattern is a plain
    /// substring and needs no validation.
    ///
    /// # Errors
    ///
    /// Returns [`MatchingRuleError::InvalidRegex`] if `match_type` is a
    /// `Regex` whose pattern fails to compile.
    pub fn new(field: MatchField, match_type: MatchType) -> Result<Self, MatchingRuleError> {
        if let MatchType::Regex(pattern) = &match_type {
            validate_regex(pattern)?;
        }
        Ok(Self { field, match_type })
    }
}

/// Compiles `pattern` as a [`regex::Regex`] purely to validate it, discarding
/// the compiled result. The single place regex validation happens, so
/// [`MatchCriterion::new`] stays a one-line dispatch.
fn validate_regex(pattern: &str) -> Result<(), MatchingRuleError> {
    regex::Regex::new(pattern)
        .map(|_| ())
        .map_err(|source| MatchingRuleError::InvalidRegex {
            pattern: pattern.to_string(),
            reason: source.to_string(),
        })
}

/// A user-stored categorization rule, persisted with its own identity and
/// position in the evaluation order (see `MatchingRuleRepository`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchingRule {
    pub id: ulid::Ulid,
    /// Criteria ANDed together; a rule matches a transaction only when every
    /// criterion matches it.
    pub criteria: Vec<MatchCriterion>,
    /// The category assigned to transactions matching this rule.
    pub category: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Fields needed to create a new [`MatchingRule`]; the repository assigns
/// identity, position, and timestamps.
#[derive(Debug, Clone)]
pub struct NewMatchingRule {
    pub criteria: Vec<MatchCriterion>,
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Contains` criterion needs no regex validation and always
    /// constructs successfully.
    #[test]
    fn constructs_valid_contains_criterion() {
        let criterion = MatchCriterion::new(MatchField::Merchant, MatchType::Contains("netto".to_string()));
        assert!(criterion.is_ok());
    }

    /// A `Regex` criterion with a pattern that compiles constructs
    /// successfully.
    #[test]
    fn constructs_valid_regex_criterion() {
        let criterion = MatchCriterion::new(MatchField::Merchant, MatchType::Regex(r"^NETTO \d+".to_string()));
        assert!(criterion.is_ok());
    }

    /// A `Regex` criterion whose pattern fails to compile is rejected with
    /// `MatchingRuleError::InvalidRegex` rather than silently accepted.
    #[test]
    fn rejects_invalid_regex_criterion() {
        let result = MatchCriterion::new(MatchField::Merchant, MatchType::Regex("(unclosed".to_string()));
        let Err(err) = result else {
            panic!("expected invalid regex to be rejected");
        };
        assert!(matches!(err, MatchingRuleError::InvalidRegex { .. }));
    }
}
