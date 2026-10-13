//! Import-time provenance details attached to a [`crate::entities::Transaction`].
//!
//! These fields capture what a CSV/bank importer saw on the original row
//! (merchant, location, card digits, raw bank-specific strings, etc.) so the
//! information survives categorization instead of being discarded. This is
//! metadata about the import, not core transaction data: it is never used to
//! determine budget attribution or amount.

use serde::{Deserialize, Serialize};

/// Details captured from an import source for a single transaction.
///
/// Every field except `source` is optional because different importers (and
/// even different rows within the same import) populate different subsets
/// of this data — a bank CSV export may omit the merchant city, a manual
/// entry has none of it, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDetails {
    /// Identifier of the importer/source that produced this transaction
    /// (e.g. a bank CSV format name).
    pub source: String,
    /// Merchant name as reported by the source, if present.
    pub merchant: Option<String>,
    /// City associated with the transaction, if present.
    pub city: Option<String>,
    /// Country associated with the transaction, if present.
    pub country: Option<String>,
    /// Last 4 digits of the card used, if present.
    pub card_last4: Option<String>,
    /// Raw transaction type string as reported by the source, if present.
    pub raw_type: Option<String>,
    /// Currency code as reported by the source, if present.
    pub currency: Option<String>,
    /// Raw, unprocessed description line from the source, if present.
    pub raw_description: Option<String>,
    /// Original amount fragment as text (e.g. before currency conversion or
    /// sign normalization), if present.
    pub original_amount_fragment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that an `ImportDetails` with every optional field populated
    /// survives a serialize/deserialize round-trip unchanged.
    #[test]
    fn test_serde_round_trip_with_all_fields_present() {
        let details = ImportDetails {
            source: "mbank_csv".to_string(),
            merchant: Some("Lidl".to_string()),
            city: Some("Warsaw".to_string()),
            country: Some("PL".to_string()),
            card_last4: Some("1234".to_string()),
            raw_type: Some("Płatność kartą".to_string()),
            currency: Some("PLN".to_string()),
            raw_description: Some("PLATNOSC KARTA LIDL WARSZAWA".to_string()),
            original_amount_fragment: Some("-123,45".to_string()),
        };

        let json = serde_json::to_string(&details).unwrap();
        let round_tripped: ImportDetails = serde_json::from_str(&json).unwrap();

        assert_eq!(details, round_tripped);
    }

    /// Verifies that an `ImportDetails` with every optional field absent
    /// still round-trips correctly, since most fields are expected to be
    /// `None` for many importers.
    #[test]
    fn test_serde_round_trip_with_all_optional_fields_none() {
        let details = ImportDetails {
            source: "manual".to_string(),
            merchant: None,
            city: None,
            country: None,
            card_last4: None,
            raw_type: None,
            currency: None,
            raw_description: None,
            original_amount_fragment: None,
        };

        let json = serde_json::to_string(&details).unwrap();
        let round_tripped: ImportDetails = serde_json::from_str(&json).unwrap();

        assert_eq!(details, round_tripped);
    }
}
