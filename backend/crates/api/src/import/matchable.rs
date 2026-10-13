//! `csv_import::Matchable` implementation for `domain::entities::ImportDetails`.
//!
//! Lives in `api` (not `domain`) because `domain` must have no dependency on
//! `csv_import` (hexagonal architecture: `domain` has no adapter/framework
//! deps), while `api` already depends on both. This lets the bulk-categorize
//! feature (section 3) run the same `csv_import::rules` matching logic
//! against already-persisted transactions' stored `ImportDetails`, not just
//! freshly parsed CSV rows.

use csv_import::Matchable;
use domain::entities::ImportDetails;

/// Thin borrowing wrapper making `&ImportDetails` matchable against
/// `csv_import::rules`. A newtype rather than `impl Matchable for
/// ImportDetails` directly: both the trait and the type are foreign to this
/// crate, so Rust's orphan rules forbid that impl — this wrapper is the
/// standard workaround.
pub struct MatchableImportDetails<'a>(pub &'a ImportDetails);

impl Matchable for MatchableImportDetails<'_> {
    fn merchant(&self) -> Option<&str> {
        self.0.merchant.as_deref()
    }

    fn raw_type(&self) -> Option<&str> {
        self.0.raw_type.as_deref()
    }

    fn city(&self) -> Option<&str> {
        self.0.city.as_deref()
    }

    fn country(&self) -> Option<&str> {
        self.0.country.as_deref()
    }

    fn description(&self) -> Option<&str> {
        self.0.raw_description.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv_import::{match_rule, MatchCriteria, MatchCriterion, MatchField, MatchType};

    /// Builds an `ImportDetails` with every optional field `None` — only
    /// `source` (non-optional) is set.
    fn bare_import_details() -> ImportDetails {
        ImportDetails {
            source: "csv_import".to_string(),
            merchant: None,
            city: None,
            country: None,
            card_last4: None,
            raw_type: None,
            currency: None,
            raw_description: None,
            original_amount_fragment: None,
        }
    }

    /// Every `MatchField` variant maps to a `None` accessor on a bare
    /// `ImportDetails`, so a single-criterion rule on any field never
    /// matches it (criteria never match an absent field — see
    /// `csv_import::rules::criterion_matches`).
    #[test]
    fn single_criterion_never_matches_a_bare_import_details() {
        let details = bare_import_details();
        let matchable = MatchableImportDetails(&details);

        for field in [
            MatchField::Merchant,
            MatchField::RawType,
            MatchField::City,
            MatchField::Country,
            MatchField::Description,
        ] {
            let rule = csv_import::Rule {
                pattern: MatchCriteria(vec![MatchCriterion {
                    field,
                    match_type: MatchType::Contains("anything".to_string()),
                }]),
                category: "whatever".to_string(),
            };
            assert!(
                !match_rule(&matchable, &rule),
                "expected field {field:?} to never match a bare ImportDetails"
            );
        }
    }
}
