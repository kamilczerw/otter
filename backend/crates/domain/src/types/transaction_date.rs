use crate::errors::DomainError;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransactionDate(NaiveDate);

impl TransactionDate {
    pub fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    pub fn value(&self) -> NaiveDate {
        self.0
    }
}

impl fmt::Display for TransactionDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl FromStr for TransactionDate {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
            DomainError::InvalidTransactionDate {
                reason: format!("expected YYYY-MM-DD format, got '{}'", s),
            }
        })?;
        Ok(Self(date))
    }
}

impl Serialize for TransactionDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TransactionDate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        TransactionDate::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_naive_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let td = TransactionDate::new(date);
        assert_eq!(td.value(), date);
    }

    #[test]
    fn test_from_str_valid() {
        let td: TransactionDate = "2026-02-05".parse().unwrap();
        assert_eq!(td.value(), NaiveDate::from_ymd_opt(2026, 2, 5).unwrap());
    }

    #[test]
    fn test_from_str_invalid_month() {
        assert!("2026-13-01".parse::<TransactionDate>().is_err());
    }

    #[test]
    fn test_from_str_invalid_abc() {
        assert!("abc".parse::<TransactionDate>().is_err());
    }

    #[test]
    fn test_from_str_empty() {
        assert!("".parse::<TransactionDate>().is_err());
    }

    #[test]
    fn test_from_str_wrong_format() {
        assert!("2024/06/15".parse::<TransactionDate>().is_err());
    }

    #[test]
    fn test_display() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let td = TransactionDate::new(date);
        assert_eq!(td.to_string(), "2026-02-05");
    }

    #[test]
    fn test_display_padding() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        let td = TransactionDate::new(date);
        assert_eq!(td.to_string(), "2024-03-05");
    }

    #[test]
    fn test_serde_roundtrip() {
        let td: TransactionDate = "2026-02-05".parse().unwrap();
        let json = serde_json::to_string(&td).unwrap();
        assert_eq!(json, "\"2026-02-05\"");
        let deserialized: TransactionDate = serde_json::from_str(&json).unwrap();
        assert_eq!(td, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_end_of_month() {
        let td: TransactionDate = "2024-11-30".parse().unwrap();
        let json = serde_json::to_string(&td).unwrap();
        assert_eq!(json, "\"2024-11-30\"");
        let deserialized: TransactionDate = serde_json::from_str(&json).unwrap();
        assert_eq!(td, deserialized);
    }

    #[test]
    fn test_serde_deserialize_invalid() {
        assert!(serde_json::from_str::<TransactionDate>("\"2026-13-01\"").is_err());
        assert!(serde_json::from_str::<TransactionDate>("\"abc\"").is_err());
    }
}
