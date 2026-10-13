//! Extraction of structured fields from a [`RawRow`] into a
//! [`ParsedTransaction`].
//!
//! The 5 named columns (`Data waluty`, `Kwota`, `Waluta`, `Typ transakcji`)
//! are always populated for every row shape, so they are the source of
//! `date`, `amount`, `currency` and `raw_type`. The remaining fields come
//! from free-text fragments in the trailing unnamed columns, which are only
//! populated on `Płatność kartą` rows; on other row shapes (e.g.
//! `Autooszczędzanie`) they are absent (`None`), not an error — see
//! [`extract_transaction`].

use crate::csv_input::RawRow;
use chrono::NaiveDate;
use domain::types::Money;
use serde::Serialize;

/// Expected `chrono` format of the `Data waluty` column, e.g. `2026-10-09`.
const VALUE_DATE_FORMAT: &str = "%Y-%m-%d";

/// A transaction with its structured fields extracted from a [`RawRow`].
///
/// Card-payment-only fields (`card_last4`, `merchant`, `city`, `country`,
/// `title`) are `None` on row shapes that don't carry them (e.g. plain
/// transfers), which is normal rather than an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedTransaction {
    /// Settlement date, from the `Data waluty` column.
    pub date: NaiveDate,
    /// Signed transaction amount in minor currency units (e.g. grosz), from
    /// the `Kwota` column.
    pub amount: Money,
    /// Currency code, e.g. `PLN`, from the `Waluta` column.
    pub currency: String,
    /// Raw transaction type as printed by the bank, e.g. `Płatność kartą`.
    pub raw_type: String,
    /// Last 4 digits of the masked card number, if this row carries one.
    pub card_last4: Option<String>,
    /// Merchant name, if this row carries a `Lokalizacja:` fragment.
    pub merchant: Option<String>,
    /// Merchant city, if this row carries a `Lokalizacja:` fragment.
    pub city: Option<String>,
    /// Merchant country, if this row carries a `Lokalizacja:` fragment.
    pub country: Option<String>,
    /// Payment title, if this row's `Opis transakcji` starts with `Tytuł:`.
    pub title: Option<String>,
    /// Verbatim, unmodified `Opis transakcji` column content, kept for
    /// audit/debugging regardless of how `title` extraction went.
    pub raw_description: String,
    /// Raw, unparsed original-amount fragment (e.g. a foreign-currency
    /// amount before conversion), if this row's original-amount column is
    /// non-empty. Kept verbatim for display/audit; this change deliberately
    /// does not parse it into a typed amount/currency pair.
    pub original_amount_fragment: Option<String>,
}

/// Errors from fields that are always populated and therefore genuinely
/// can't be skipped: a malformed `Kwota` or `Data waluty`.
///
/// Malformed description fragments (`Tytuł:`/`Lokalizacja:`/`Numer karty:`)
/// are deliberately *not* covered here — they degrade to `None` instead,
/// since the no-merchant transfer row shape is normal, not an error.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExtractError {
    /// The `Kwota` column didn't parse as a signed decimal amount.
    #[error("invalid amount in Kwota column: {0:?}")]
    InvalidAmount(String),

    /// The `Data waluty` column didn't parse as a `YYYY-MM-DD` date.
    #[error("invalid date in Data waluty column: {0:?}")]
    InvalidDate(String),
}

/// Extracts a [`ParsedTransaction`] from a [`RawRow`]. Pure function, no I/O.
///
/// # Errors
///
/// Returns [`ExtractError::InvalidAmount`] if `Kwota` doesn't parse as a
/// signed decimal amount, or [`ExtractError::InvalidDate`] if `Data waluty`
/// doesn't parse as a `YYYY-MM-DD` date.
pub fn extract_transaction(row: &RawRow) -> Result<ParsedTransaction, ExtractError> {
    let date = parse_value_date(&row.value_date)?;
    let amount = Money::new(parse_amount_minor_units(&row.amount)?);
    let (merchant, city, country) = parse_location_fragment(&row.location_fragment);

    Ok(ParsedTransaction {
        date,
        amount,
        currency: row.currency.clone(),
        raw_type: row.transaction_type.clone(),
        card_last4: parse_card_last4(&row.card_number_fragment),
        merchant,
        city,
        country,
        title: parse_title(&row.description),
        raw_description: row.description.clone(),
        original_amount_fragment: non_empty(&row.original_amount_fragment),
    })
}

/// Converts an empty string into `None`, leaving a non-empty string as
/// `Some` of itself. Used for columns that are either absent (empty) or
/// present verbatim, with no further parsing of their content.
fn non_empty(raw: &str) -> Option<String> {
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

/// Parses the `Data waluty` column as a `YYYY-MM-DD` date.
fn parse_value_date(raw: &str) -> Result<NaiveDate, ExtractError> {
    NaiveDate::parse_from_str(raw.trim(), VALUE_DATE_FORMAT)
        .map_err(|_| ExtractError::InvalidDate(raw.to_string()))
}

/// Parses a signed decimal amount string (e.g. `"-103.10"`) into minor
/// currency units (e.g. grosz) as an `i64`.
///
/// Parses the sign, integer part and 2-digit fractional part directly as
/// integers instead of going through floating point, to avoid rounding
/// bugs on money values.
fn parse_amount_minor_units(raw: &str) -> Result<i64, ExtractError> {
    let invalid = || ExtractError::InvalidAmount(raw.to_string());
    let trimmed = raw.trim();

    let (is_negative, unsigned) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };

    let (int_part, frac_part) = unsigned.split_once('.').ok_or_else(invalid)?;
    if frac_part.len() != 2 {
        return Err(invalid());
    }

    let int_value: i64 = int_part.parse().map_err(|_| invalid())?;
    let frac_value: i64 = frac_part.parse().map_err(|_| invalid())?;
    let magnitude = int_value * 100 + frac_value;

    Ok(if is_negative { -magnitude } else { magnitude })
}

/// Parses a `Lokalizacja: Adres: ... Miasto: ... Kraj: ...` fragment into
/// `(merchant, city, country)`.
///
/// Returns `(None, None, None)` if the fragment is empty or doesn't contain
/// all three labels — this is normal for row shapes without a merchant
/// (e.g. transfers), not an error.
fn parse_location_fragment(fragment: &str) -> (Option<String>, Option<String>, Option<String>) {
    let no_match = (None, None, None);

    let Some((_, after_adres)) = fragment.split_once("Adres:") else {
        return no_match;
    };
    let Some((merchant, after_miasto)) = after_adres.split_once("Miasto:") else {
        return no_match;
    };
    let Some((city, country)) = after_miasto.split_once("Kraj:") else {
        return no_match;
    };

    (
        Some(merchant.trim().to_string()),
        Some(city.trim().to_string()),
        Some(country.trim().to_string()),
    )
}

/// Parses a `Numer karty: 425125******1836` fragment into its last 4 digits.
///
/// Returns `None` if the fragment is empty, doesn't carry the `Numer
/// karty:` label, or has fewer than 4 characters after it.
fn parse_card_last4(fragment: &str) -> Option<String> {
    let after_label = fragment.trim().strip_prefix("Numer karty:")?.trim();
    let char_count = after_label.chars().count();
    if char_count < 4 {
        return None;
    }
    Some(after_label.chars().skip(char_count - 4).collect())
}

/// Parses a `Tytuł: ...` description into its title, or `None` if the
/// description doesn't start with the `Tytuł:` label (e.g. transfer rows,
/// where the full text goes into `raw_description` instead).
fn parse_title(description: &str) -> Option<String> {
    description
        .strip_prefix("Tytuł:")
        .map(|rest| rest.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_input::parse_rows;
    use crate::decode::decode_windows1250;
    use assert2::{check, let_assert};

    /// Loads and parses the sample fixture into raw rows.
    fn fixture_rows() -> Vec<RawRow> {
        let bytes = include_bytes!("../tests/fixtures/sample_export.csv");
        let text = decode_windows1250(bytes);
        let_assert!(Ok((rows, _warnings)) = parse_rows(&text));
        rows
    }

    /// Transfer rows (`Autooszczędzanie`) have no merchant fragments; all
    /// merchant-only fields should be `None`, and the plain description
    /// text should end up in `raw_description`, not `title`.
    #[test]
    fn extracts_transfer_row() {
        let rows = fixture_rows();
        let_assert!(Some(row) = rows.first());
        let_assert!(Ok(parsed) = extract_transaction(row));

        check!(parsed.date == NaiveDate::from_ymd_opt(2026, 10, 9).unwrap());
        check!(parsed.amount == Money::new(-286));
        check!(parsed.currency == "PLN");
        check!(parsed.raw_type == "Autooszczędzanie");
        check!(parsed.card_last4 == None);
        check!(parsed.merchant == None);
        check!(parsed.city == None);
        check!(parsed.country == None);
        check!(parsed.title == None);
        check!(parsed.raw_description == "AUTOOSZCZĘDZANIE-PRZELEW");
        check!(parsed.original_amount_fragment == None);
    }

    /// A card payment row with a simple, single-word merchant name.
    #[test]
    fn extracts_card_payment_row() {
        let rows = fixture_rows();
        let_assert!(Some(row) = rows.get(3));
        let_assert!(Ok(parsed) = extract_transaction(row));

        check!(parsed.date == NaiveDate::from_ymd_opt(2026, 10, 10).unwrap());
        check!(parsed.amount == Money::new(-10310));
        check!(parsed.currency == "PLN");
        check!(parsed.raw_type == "Płatność kartą");
        check!(parsed.card_last4 == Some("1836".to_string()));
        check!(parsed.merchant == Some("NETTO 5567 K.2".to_string()));
        check!(parsed.city == Some("CIESZYN".to_string()));
        check!(parsed.country == Some("POLSKA".to_string()));
        check!(parsed.title == Some("000498849 74230786284201674667219".to_string()));
    }

    /// A card payment row with a multi-word, digit-containing `Adres:`
    /// fragment, to confirm the split handles more than a single token.
    #[test]
    fn extracts_card_payment_row_with_multiword_address() {
        let rows = fixture_rows();
        let_assert!(Some(row) = rows.get(5));
        let_assert!(Ok(parsed) = extract_transaction(row));

        check!(parsed.merchant == Some("AUTOMATY AUTOMATSPEC 2026".to_string()));
        check!(parsed.city == Some("KASZOW".to_string()));
        check!(parsed.country == Some("POLSKA".to_string()));
    }

    /// A card payment row whose country contains spaces and diacritics,
    /// confirming the country field captures the whole rest of the string.
    #[test]
    fn extracts_card_payment_row_with_diacritic_country() {
        let rows = fixture_rows();
        let_assert!(Some(row) = rows.get(8));
        let_assert!(Ok(parsed) = extract_transaction(row));

        check!(parsed.merchant == Some("UBER   *ONE MEMBERSHIP".to_string()));
        check!(parsed.city == Some("AMSTERDAM".to_string()));
        check!(parsed.country == Some("KRÓLESTWO NIDERLANDÓW".to_string()));
    }

    /// A row with a non-empty original-amount column (e.g. a
    /// foreign-currency purchase) carries that column's raw text verbatim
    /// in `original_amount_fragment`, unparsed.
    #[test]
    fn extracts_non_empty_original_amount_fragment() {
        let mut row = fixture_rows().remove(0);
        row.original_amount_fragment = "Oryginalna kwota operacji: 1.00 EUR".to_string();
        let_assert!(Ok(parsed) = extract_transaction(&row));

        check!(
            parsed.original_amount_fragment
                == Some("Oryginalna kwota operacji: 1.00 EUR".to_string())
        );
    }

    /// A row whose original-amount column is empty has no
    /// `original_amount_fragment`.
    #[test]
    fn extracts_empty_original_amount_fragment_as_none() {
        let mut row = fixture_rows().remove(0);
        row.original_amount_fragment = String::new();
        let_assert!(Ok(parsed) = extract_transaction(&row));

        check!(parsed.original_amount_fragment == None);
    }

    /// An unparseable `Kwota` is a hard error since it's an always-present
    /// required column.
    #[test]
    fn rejects_invalid_amount() {
        let mut row = fixture_rows().remove(0);
        row.amount = "not-a-number".to_string();
        let_assert!(Err(ExtractError::InvalidAmount(_)) = extract_transaction(&row));
    }

    /// An unparseable `Data waluty` is a hard error since it's an
    /// always-present required column.
    #[test]
    fn rejects_invalid_date() {
        let mut row = fixture_rows().remove(0);
        row.value_date = "not-a-date".to_string();
        let_assert!(Err(ExtractError::InvalidDate(_)) = extract_transaction(&row));
    }

    /// Positive, negative and whole-number-looking amounts all parse to the
    /// correct minor-unit integer without floating point rounding.
    #[test]
    fn parses_amount_minor_units_directly() {
        check!(parse_amount_minor_units("-103.10") == Ok(-10310));
        check!(parse_amount_minor_units("103.10") == Ok(10310));
        check!(parse_amount_minor_units("-2.86") == Ok(-286));
        check!(parse_amount_minor_units("21.00") == Ok(2100));
        check!(parse_amount_minor_units("0.00") == Ok(0));
    }
}
