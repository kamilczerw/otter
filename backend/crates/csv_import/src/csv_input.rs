//! Parsing of decoded bank CSV text into raw, unextracted rows.
//!
//! The bank export always has 13 columns: 7 named in the header
//! (`Data operacji`, `Data waluty`, `Typ transakcji`, `Kwota`, `Waluta`,
//! `Saldo po transakcji`, `Opis transakcji`) plus 6 unnamed trailing columns
//! whose content depends on the transaction type (e.g. `Płatność kartą` rows
//! carry `Tytuł:`/`Lokalizacja:`/etc. fragments there; `Autooszczędzanie`
//! rows leave them empty). This module only shapes rows into [`RawRow`];
//! extracting structured fields out of the free-text fragments is a later
//! pipeline stage.

/// Number of columns in every row of the bank export, including the header.
const EXPECTED_COLUMN_COUNT: usize = 13;

/// One parsed-but-unextracted CSV row from a bank export.
///
/// Fields map 1:1 to the 13 columns by position. The first 7 use the real
/// header names; the remaining 6 are named for what they hold in practice on
/// `Płatność kartą` rows, even though other row shapes (e.g.
/// `Autooszczędzanie`) leave them empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRow {
    /// Column 1: `Data operacji` — the date the operation was recorded.
    pub operation_date: String,
    /// Column 2: `Data waluty` — the value/settlement date.
    pub value_date: String,
    /// Column 3: `Typ transakcji` — e.g. `Autooszczędzanie`, `Płatność kartą`.
    pub transaction_type: String,
    /// Column 4: `Kwota` — signed transaction amount, as printed by the bank.
    pub amount: String,
    /// Column 5: `Waluta` — currency code, e.g. `PLN`.
    pub currency: String,
    /// Column 6: `Saldo po transakcji` — account balance after the
    /// transaction. Observed empty in every sample row.
    pub balance_after: String,
    /// Column 7: `Opis transakcji` — free-text description. For card
    /// payments this holds a `Tytuł: ...` fragment; for transfers it holds
    /// the full description directly.
    pub description: String,
    /// Column 8: unnamed in the header; holds a `Lokalizacja: ...` fragment
    /// on card payment rows, empty otherwise.
    pub location_fragment: String,
    /// Column 9: unnamed in the header; holds a
    /// `Data wykonania operacji: ...` fragment on card payment rows, empty
    /// otherwise.
    pub execution_date_fragment: String,
    /// Column 10: unnamed in the header; holds an
    /// `Oryginalna kwota operacji: ...` fragment on card payment rows, empty
    /// otherwise.
    pub original_amount_fragment: String,
    /// Column 11: unnamed in the header; holds a `Numer karty: ...` fragment
    /// on card payment rows, empty otherwise.
    pub card_number_fragment: String,
    /// Column 12: unnamed in the header; always empty in every observed row
    /// shape, but kept for a faithful 1:1 mapping to the source columns.
    pub trailing_fragment_1: String,
    /// Column 13: unnamed in the header; always empty in every observed row
    /// shape, but kept for a faithful 1:1 mapping to the source columns.
    pub trailing_fragment_2: String,
}

/// Errors that make the whole CSV unparseable, as opposed to a single
/// malformed row (which is skipped with a [`ParseWarning`] instead).
#[derive(Debug, thiserror::Error)]
pub enum CsvParseError {
    /// The underlying CSV reader failed in a way that can't be attributed
    /// to one row, e.g. it couldn't even read the header.
    #[error("failed to read CSV data: {0}")]
    Csv(#[from] csv::Error),

    /// The header row doesn't have the expected 13-column shape, so there
    /// is no reliable way to interpret any data row either.
    #[error("unexpected CSV header: expected {EXPECTED_COLUMN_COUNT} columns, found {0}")]
    UnexpectedHeader(usize),
}

/// A warning produced when a CSV row is skipped instead of aborting the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseWarning {
    /// 1-based row number among the data rows (header excluded).
    pub row_number: usize,
    /// Human-readable reason the row was skipped.
    pub reason: String,
}

impl std::fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "row {}: {}", self.row_number, self.reason)
    }
}

/// Parses decoded CSV text into raw rows, collecting a warning for every
/// row that doesn't match the expected 13-column shape instead of failing
/// the whole run.
///
/// This is a pure function (no I/O) so it can be tested directly; printing
/// the returned warnings is left to [`report_warnings`].
///
/// # Errors
///
/// Returns `CsvParseError::UnexpectedHeader` if the header row doesn't have
/// 13 columns, or `CsvParseError::Csv` if the reader fails to read the
/// header at all.
pub fn parse_rows(csv_text: &str) -> Result<(Vec<RawRow>, Vec<ParseWarning>), CsvParseError> {
    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    validate_header(reader.headers()?)?;

    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    for (index, result) in reader.records().enumerate() {
        let row_number = index + 1;
        match result {
            Ok(record) => match row_from_record(&record) {
                Some(row) => rows.push(row),
                None => warnings.push(column_count_warning(row_number, record.len())),
            },
            Err(err) => warnings.push(ParseWarning {
                row_number,
                reason: err.to_string(),
            }),
        }
    }
    Ok((rows, warnings))
}

/// Checks that the CSV header has the expected 13-column shape.
fn validate_header(header: &csv::StringRecord) -> Result<(), CsvParseError> {
    if header.len() == EXPECTED_COLUMN_COUNT {
        Ok(())
    } else {
        Err(CsvParseError::UnexpectedHeader(header.len()))
    }
}

/// Builds the warning for a data row with the wrong number of columns.
fn column_count_warning(row_number: usize, found: usize) -> ParseWarning {
    ParseWarning {
        row_number,
        reason: format!("expected {EXPECTED_COLUMN_COUNT} columns, found {found}"),
    }
}

/// Converts a CSV record into a [`RawRow`], or `None` if it doesn't have
/// exactly 13 columns.
fn row_from_record(record: &csv::StringRecord) -> Option<RawRow> {
    if record.len() != EXPECTED_COLUMN_COUNT {
        return None;
    }
    let field = |i: usize| record.get(i).unwrap_or_default().to_string();
    Some(RawRow {
        operation_date: field(0),
        value_date: field(1),
        transaction_type: field(2),
        amount: field(3),
        currency: field(4),
        balance_after: field(5),
        description: field(6),
        location_fragment: field(7),
        execution_date_fragment: field(8),
        original_amount_fragment: field(9),
        card_number_fragment: field(10),
        trailing_fragment_1: field(11),
        trailing_fragment_2: field(12),
    })
}

/// Prints every warning to stderr, identifying the row and the reason it
/// was skipped. Kept as a thin I/O wrapper so [`parse_rows`] itself stays
/// pure and directly testable.
pub fn report_warnings(warnings: &[ParseWarning]) {
    for warning in warnings {
        eprintln!("warning: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::decode_windows1250;
    use assert2::{check, let_assert};

    /// The sample fixture is a real Windows-1250 encoded file; decoding it
    /// should round-trip to the UTF-8 text we encoded it from.
    #[test]
    fn decodes_fixture_to_expected_utf8() {
        let bytes = include_bytes!("../tests/fixtures/sample_export.csv");
        let text = decode_windows1250(bytes);
        check!(text.starts_with("\"Data operacji\",\"Data waluty\""));
        check!(text.contains("AUTOOSZCZĘDZANIE-PRZELEW"));
        check!(text.contains("Miasto: KASZOW"));
    }

    /// Parsing the decoded fixture should yield 12 rows with no warnings,
    /// covering both the transfer and card payment row shapes.
    #[test]
    fn parses_fixture_rows() {
        let bytes = include_bytes!("../tests/fixtures/sample_export.csv");
        let text = decode_windows1250(bytes);
        let_assert!(Ok((rows, warnings)) = parse_rows(&text));

        check!(rows.len() == 12);
        check!(warnings.is_empty());
    }

    /// An `Autooszczędzanie` row has no merchant fragments; only the first
    /// 7 columns are populated.
    #[test]
    fn parses_transfer_row_shape() {
        let bytes = include_bytes!("../tests/fixtures/sample_export.csv");
        let text = decode_windows1250(bytes);
        let_assert!(Ok((rows, _warnings)) = parse_rows(&text));

        let_assert!(Some(row) = rows.first());
        check!(row.transaction_type == "Autooszczędzanie");
        check!(row.amount == "-2.86");
        check!(row.description == "AUTOOSZCZĘDZANIE-PRZELEW");
        check!(row.location_fragment.is_empty());
        check!(row.card_number_fragment.is_empty());
    }

    /// A `Płatność kartą` row carries its merchant details across the
    /// trailing unnamed columns.
    #[test]
    fn parses_card_payment_row_shape() {
        let bytes = include_bytes!("../tests/fixtures/sample_export.csv");
        let text = decode_windows1250(bytes);
        let_assert!(Ok((rows, _warnings)) = parse_rows(&text));

        let_assert!(Some(row) = rows.get(3));
        check!(row.transaction_type == "Płatność kartą");
        check!(row.amount == "-103.10");
        check!(row.description.starts_with("Tytuł:"));
        check!(row.location_fragment.contains("Miasto: CIESZYN"));
        check!(row.card_number_fragment == "Numer karty: 425125******1836");
    }

    /// A row with the wrong number of columns is skipped with a warning
    /// instead of aborting the whole parse; the remaining valid rows still
    /// come through.
    #[test]
    fn skips_malformed_row_with_warning() {
        let csv_text = "\"Data operacji\",\"Data waluty\",\"Typ transakcji\",\"Kwota\",\"Waluta\",\"Saldo po transakcji\",\"Opis transakcji\",\"\",\"\",\"\",\"\",\"\",\"\"\n\
             \"a\",\"b\",\"c\",\"d\",\"e\"\n\
             \"2026-10-11\",\"2026-10-09\",\"Autooszczędzanie\",\"-2.86\",\"PLN\",\"\",\"AUTOOSZCZĘDZANIE-PRZELEW\",\"\",\"\",\"\",\"\",\"\",\"\"\n";

        let_assert!(Ok((rows, warnings)) = parse_rows(csv_text));

        check!(rows.len() == 1);
        let_assert!(Some(warning) = warnings.first());
        check!(warning.row_number == 1);
        check!(warning.reason.contains('5'));
    }

    /// A header with the wrong column count is an unrecoverable error,
    /// since no data row can be reliably interpreted either.
    #[test]
    fn rejects_header_with_wrong_column_count() {
        let csv_text = "\"a\",\"b\",\"c\"\n";
        let_assert!(Err(CsvParseError::UnexpectedHeader(3)) = parse_rows(csv_text));
    }
}
