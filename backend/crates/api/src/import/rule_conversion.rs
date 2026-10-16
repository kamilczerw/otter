//! Conversions between `domain::entities::MatchingRule`'s match-criteria
//! shape and `csv_import::rules`'s shape.
//!
//! The two crates declare their own near-identical `MatchField`/`MatchType`/
//! `MatchCriterion` enums rather than sharing one (see the
//! `add-rule-editor-ui` design doc's "Decisions" section on why `domain`
//! can't depend on `csv_import`). This module is the single place that
//! bridges them, so neither the read path
//! ([`crate::import::ImportDeps::current_rules`], domain -> csv_import) nor
//! the write path (the uncategorized "save rule" handler, csv_import ->
//! domain) needs to know about the other side's enum shape.

use domain::entities::{
    MatchCriterion as DomainMatchCriterion, MatchField as DomainMatchField,
    MatchType as DomainMatchType, MatchingRule as DomainMatchingRule, NewMatchingRule,
};
use domain::errors::MatchingRuleError;

/// Converts a domain match field into its `csv_import` equivalent.
fn domain_field_to_csv(field: DomainMatchField) -> csv_import::MatchField {
    match field {
        DomainMatchField::Merchant => csv_import::MatchField::Merchant,
        DomainMatchField::RawType => csv_import::MatchField::RawType,
        DomainMatchField::City => csv_import::MatchField::City,
        DomainMatchField::Country => csv_import::MatchField::Country,
        DomainMatchField::Description => csv_import::MatchField::Description,
        DomainMatchField::RawFragments => csv_import::MatchField::RawFragments,
    }
}

/// Converts a domain match type into its `csv_import` equivalent.
fn domain_match_type_to_csv(match_type: DomainMatchType) -> csv_import::MatchType {
    match match_type {
        DomainMatchType::Contains(pattern) => csv_import::MatchType::Contains(pattern),
        DomainMatchType::Regex(pattern) => csv_import::MatchType::Regex(pattern),
    }
}

/// Converts one domain match criterion into its `csv_import` equivalent.
fn domain_criterion_to_csv(criterion: DomainMatchCriterion) -> csv_import::MatchCriterion {
    csv_import::MatchCriterion {
        field: domain_field_to_csv(criterion.field),
        match_type: domain_match_type_to_csv(criterion.match_type),
    }
}

/// Converts a stored [`DomainMatchingRule`] into the `csv_import::Rule` shape
/// the categorization engine (`csv_import::match_rule`/`categorize`)
/// consumes. Used by [`crate::import::ImportDeps::current_rules`] to append
/// user-stored rules after the config-defined inline rules.
pub fn domain_rule_to_csv_rule(rule: DomainMatchingRule) -> csv_import::Rule {
    csv_import::Rule {
        pattern: csv_import::MatchCriteria(
            rule.criteria.into_iter().map(domain_criterion_to_csv).collect(),
        ),
        category: rule.category,
    }
}

/// Converts a `csv_import` match field into its domain equivalent.
fn csv_field_to_domain(field: csv_import::MatchField) -> DomainMatchField {
    match field {
        csv_import::MatchField::Merchant => DomainMatchField::Merchant,
        csv_import::MatchField::RawType => DomainMatchField::RawType,
        csv_import::MatchField::City => DomainMatchField::City,
        csv_import::MatchField::Country => DomainMatchField::Country,
        csv_import::MatchField::Description => DomainMatchField::Description,
        csv_import::MatchField::RawFragments => DomainMatchField::RawFragments,
    }
}

/// Converts a `csv_import` match type into its domain equivalent.
fn csv_match_type_to_domain(match_type: csv_import::MatchType) -> DomainMatchType {
    match match_type {
        csv_import::MatchType::Contains(pattern) => DomainMatchType::Contains(pattern),
        csv_import::MatchType::Regex(pattern) => DomainMatchType::Regex(pattern),
    }
}

/// Converts one `csv_import` match criterion into a validated domain
/// criterion.
///
/// # Errors
///
/// Returns [`MatchingRuleError::InvalidRegex`] if the criterion is a
/// `Regex` whose pattern fails to compile (see `DomainMatchCriterion::new`).
fn csv_criterion_to_domain(
    criterion: csv_import::MatchCriterion,
) -> Result<DomainMatchCriterion, MatchingRuleError> {
    DomainMatchCriterion::new(
        csv_field_to_domain(criterion.field),
        csv_match_type_to_domain(criterion.match_type),
    )
}

/// Converts a learned `csv_import::Rule` (built by the uncategorized-review
/// "save rule" handler) into a [`NewMatchingRule`] ready for
/// `MatchingRuleRepository::create`, validating every criterion's regex
/// along the way.
///
/// # Errors
///
/// Returns [`MatchingRuleError::InvalidRegex`] if any criterion's `Regex`
/// pattern fails to compile.
pub fn csv_rule_to_new_matching_rule(
    rule: csv_import::Rule,
) -> Result<NewMatchingRule, MatchingRuleError> {
    let criteria = rule
        .pattern
        .0
        .into_iter()
        .map(csv_criterion_to_domain)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NewMatchingRule {
        criteria,
        category: rule.category,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a validated domain `Contains` criterion, so tests don't repeat
    /// `MatchCriterion::new(...).unwrap()` boilerplate.
    fn domain_contains(field: DomainMatchField, pattern: &str) -> DomainMatchCriterion {
        DomainMatchCriterion::new(field, DomainMatchType::Contains(pattern.to_string())).unwrap()
    }

    /// Builds a `csv_import::MatchCriterion::Contains` for `field`.
    fn csv_contains(field: csv_import::MatchField, pattern: &str) -> csv_import::MatchCriterion {
        csv_import::MatchCriterion {
            field,
            match_type: csv_import::MatchType::Contains(pattern.to_string()),
        }
    }

    /// A stored domain rule converts into an equivalent `csv_import::Rule`:
    /// same category, same criteria (field + pattern), in the same order.
    #[test]
    fn domain_rule_converts_to_equivalent_csv_rule() {
        let now = chrono::Utc::now();
        let domain_rule = DomainMatchingRule {
            id: ulid::Ulid::new(),
            criteria: vec![domain_contains(DomainMatchField::Merchant, "netto")],
            category: "groceries".to_string(),
            created_at: now,
            updated_at: now,
        };

        let csv_rule = domain_rule_to_csv_rule(domain_rule);

        assert_eq!(csv_rule.category, "groceries");
        assert_eq!(
            csv_rule.pattern.0,
            vec![csv_contains(csv_import::MatchField::Merchant, "netto")]
        );
    }

    /// A learned `csv_import::Rule` with a valid criterion converts into an
    /// equivalent `NewMatchingRule`.
    #[test]
    fn csv_rule_converts_to_new_matching_rule() {
        let csv_rule = csv_import::Rule {
            pattern: csv_import::MatchCriteria(vec![csv_contains(csv_import::MatchField::City, "Cieszyn")]),
            category: "local".to_string(),
        };

        let new_rule = csv_rule_to_new_matching_rule(csv_rule).unwrap();

        assert_eq!(new_rule.category, "local");
        assert_eq!(new_rule.criteria, vec![domain_contains(DomainMatchField::City, "Cieszyn")]);
    }

    /// A learned `csv_import::Rule` with an uncompilable regex pattern fails
    /// conversion with `MatchingRuleError::InvalidRegex`, rather than
    /// silently being stored as a never-matching rule.
    #[test]
    fn csv_rule_with_invalid_regex_fails_conversion() {
        let csv_rule = csv_import::Rule {
            pattern: csv_import::MatchCriteria(vec![csv_import::MatchCriterion {
                field: csv_import::MatchField::Merchant,
                match_type: csv_import::MatchType::Regex("(unclosed".to_string()),
            }]),
            category: "groceries".to_string(),
        };

        let result = csv_rule_to_new_matching_rule(csv_rule);
        assert!(matches!(result, Err(MatchingRuleError::InvalidRegex { .. })));
    }
}
