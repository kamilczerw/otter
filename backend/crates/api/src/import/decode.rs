//! Charset-aware decoding of an uploaded CSV file part into text, and
//! per-row structural parsing/extraction into [`RowOutcome`]s.
//!
//! This module is adapter glue only: the actual decoding/parsing/extraction
//! logic lives in `csv_import` and is reused here unchanged. What's added
//! here is (1) picking a decoder from the upload's declared charset and (2)
//! turning csv_import's "structural failure vs. one bad row" distinction
//! into this module's own [`ImportError`]/[`RowOutcome`] types, so the
//! pipeline (see `pipeline.rs`) can tell the two apart.

use csv_import::{CsvParseError, ParsedTransaction, RawRow};

/// A charset this module knows how to decode an uploaded file as.
///
/// Mirrors exactly the two charsets `csv_import` supports a decoder for
/// (UTF-8 natively, Windows-1250 via [`csv_import::decode_windows1250`]) —
/// there is no third charset to add support for without first adding a new
/// decoder to `csv_import`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// Plain UTF-8, the default when no charset is declared.
    Utf8,
    /// Windows-1250 ("CP1250"), used by some Polish bank exports.
    Windows1250,
}

/// Errors that make the whole import request invalid, as opposed to a
/// single bad row (see [`RowOutcome::Failed`] for that case instead).
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// The `charset=` parameter on the upload's `Content-Type` named a
    /// charset this module has no decoder for.
    #[error("unsupported charset: {0}")]
    UnsupportedCharset(String),

    /// The bytes weren't valid UTF-8 (only reachable when UTF-8 was the
    /// resolved charset, since Windows-1250 decoding is infallible).
    #[error("uploaded content is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    /// The decoded text couldn't be split into CSV rows/columns at all
    /// (e.g. the header doesn't have the expected shape). Per the design,
    /// this rejects the whole request; nothing gets persisted.
    #[error("uploaded content is not structurally valid CSV: {0}")]
    Structural(#[from] CsvParseError),
}

/// The outcome of extracting one data row: either a successfully extracted
/// transaction (ready for the pipeline to resolve month/category/entry for),
/// or a failure recorded with enough detail to report back to the caller.
#[derive(Debug, Clone, PartialEq)]
pub enum RowOutcome {
    /// Extraction succeeded. `row` is the row's 1-based position among
    /// successfully-shaped data rows (header excluded).
    Extracted {
        row: usize,
        transaction: ParsedTransaction,
    },
    /// Extraction failed; the row is skipped but recorded, not discarded.
    Failed(RowFailure),
}

/// A single row that failed extraction, with enough detail to identify it.
#[derive(Debug, Clone, PartialEq)]
pub struct RowFailure {
    /// 1-based position among data rows (header excluded).
    pub row: usize,
    /// Best-effort reconstruction of the row's raw field values.
    pub raw: String,
    /// Human-readable reason extraction failed.
    pub reason: String,
}

/// Extracts the `charset` parameter from a `Content-Type` header value,
/// e.g. `"text/csv; charset=windows-1250"` -> `Some("windows-1250")`.
fn extract_charset_param(content_type: &str) -> Option<&str> {
    content_type
        .split(';')
        .skip(1)
        .find_map(|part| part.trim().strip_prefix("charset="))
}

/// Resolves the [`Charset`] to decode an upload with, from its declared
/// `Content-Type` header. Defaults to UTF-8 when no charset is declared.
///
/// # Errors
///
/// Returns [`ImportError::UnsupportedCharset`] if a charset is declared but
/// isn't one this module has a decoder for.
pub fn parse_charset(content_type: Option<&str>) -> Result<Charset, ImportError> {
    let Some(declared) = content_type.and_then(extract_charset_param) else {
        return Ok(Charset::Utf8);
    };

    match declared.trim().trim_matches('"').to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" => Ok(Charset::Utf8),
        "windows-1250" | "cp1250" | "windows1250" => Ok(Charset::Windows1250),
        other => Err(ImportError::UnsupportedCharset(other.to_string())),
    }
}

/// Decodes raw upload bytes into text, per the given [`Charset`].
///
/// # Errors
///
/// Returns [`ImportError::InvalidUtf8`] if `charset` is `Utf8` and the bytes
/// aren't valid UTF-8. Windows-1250 decoding is infallible (see
/// [`csv_import::decode_windows1250`]), so it never errors here.
pub fn decode_bytes(bytes: &[u8], charset: Charset) -> Result<String, ImportError> {
    match charset {
        Charset::Utf8 => String::from_utf8(bytes.to_vec()).map_err(ImportError::InvalidUtf8),
        Charset::Windows1250 => Ok(csv_import::decode_windows1250(bytes)),
    }
}

/// Best-effort reconstruction of a [`RawRow`]'s fields, joined by commas,
/// for use as the `raw` content of a [`RowFailure`]. Not a faithful CSV
/// re-encoding (no quoting) — just enough for a human to recognize the row.
fn raw_row_to_string(row: &RawRow) -> String {
    [
        row.operation_date.as_str(),
        row.value_date.as_str(),
        row.transaction_type.as_str(),
        row.amount.as_str(),
        row.currency.as_str(),
        row.balance_after.as_str(),
        row.description.as_str(),
        row.location_fragment.as_str(),
        row.execution_date_fragment.as_str(),
        row.original_amount_fragment.as_str(),
        row.card_number_fragment.as_str(),
        row.trailing_fragment_1.as_str(),
        row.trailing_fragment_2.as_str(),
    ]
    .join(",")
}

/// Extracts a single [`RawRow`] into a [`RowOutcome`], at the given 1-based
/// row position.
fn extract_row(row_number: usize, row: RawRow) -> RowOutcome {
    match csv_import::extract_transaction(&row) {
        Ok(transaction) => RowOutcome::Extracted {
            row: row_number,
            transaction,
        },
        Err(err) => RowOutcome::Failed(RowFailure {
            row: row_number,
            raw: raw_row_to_string(&row),
            reason: err.to_string(),
        }),
    }
}

/// Decodes `bytes` using the charset declared on `content_type` (defaulting
/// to UTF-8), parses the result into CSV rows, and extracts each row into a
/// [`RowOutcome`].
///
/// # Errors
///
/// Returns [`ImportError`] if the charset is unsupported, the bytes aren't
/// valid text for the resolved charset, or the text isn't structurally
/// parseable as CSV at all. A single bad row is never an `Err` here — it
/// comes back as `Ok` containing a [`RowOutcome::Failed`] entry.
pub fn decode_and_extract_rows(
    bytes: &[u8],
    content_type: Option<&str>,
) -> Result<Vec<RowOutcome>, ImportError> {
    let charset = parse_charset(content_type)?;
    let text = decode_bytes(bytes, charset)?;
    let (rows, _warnings) = csv_import::parse_rows(&text)?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| extract_row(index + 1, row))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No `Content-Type` at all defaults to UTF-8.
    #[test]
    fn defaults_to_utf8_when_no_content_type() {
        assert_eq!(parse_charset(None).unwrap(), Charset::Utf8);
    }

    /// No `charset=` parameter on an otherwise-present `Content-Type`
    /// defaults to UTF-8 too.
    #[test]
    fn defaults_to_utf8_when_no_charset_param() {
        assert_eq!(parse_charset(Some("text/csv")).unwrap(), Charset::Utf8);
    }

    /// An explicit `charset=windows-1250` resolves to `Windows1250`.
    #[test]
    fn parses_windows_1250_charset() {
        assert_eq!(
            parse_charset(Some("text/csv; charset=windows-1250")).unwrap(),
            Charset::Windows1250
        );
    }

    /// Charset matching is case-insensitive.
    #[test]
    fn charset_matching_is_case_insensitive() {
        assert_eq!(
            parse_charset(Some("text/csv; CHARSET=UTF-8")).unwrap(),
            Charset::Utf8
        );
    }

    /// An unrecognized charset is rejected rather than silently defaulting.
    #[test]
    fn rejects_unsupported_charset() {
        let err = parse_charset(Some("text/csv; charset=shift-jis")).unwrap_err();
        assert!(matches!(err, ImportError::UnsupportedCharset(c) if c == "shift-jis"));
    }

    /// Decoding Windows-1250 bytes never fails, even for high bytes.
    #[test]
    fn decodes_windows_1250_bytes() {
        let decoded = decode_bytes(&[0xB9], Charset::Windows1250).unwrap();
        assert_eq!(decoded, "ą");
    }

    /// Decoding invalid UTF-8 bytes as UTF-8 is an error.
    #[test]
    fn rejects_invalid_utf8_bytes() {
        let err = decode_bytes(&[0xFF, 0xFE], Charset::Utf8).unwrap_err();
        assert!(matches!(err, ImportError::InvalidUtf8(_)));
    }

    /// A structurally invalid header (wrong column count) is a structural
    /// error, not a per-row failure.
    #[test]
    fn rejects_structurally_invalid_csv() {
        let err = decode_and_extract_rows(b"\"a\",\"b\",\"c\"\n", None).unwrap_err();
        assert!(matches!(err, ImportError::Structural(_)));
    }

    /// One malformed data row (bad amount) among good ones comes back as a
    /// `RowOutcome::Failed` entry, with the rest still extracted.
    #[test]
    fn collects_per_row_extraction_failures() {
        let bytes = include_bytes!("../../../csv_import/tests/fixtures/sample_export.csv");
        let text = csv_import::decode_windows1250(bytes);
        // Corrupt the amount on the first data row.
        let corrupted = text.replacen("-2.86", "not-a-number", 1);

        let outcomes = decode_and_extract_rows(corrupted.as_bytes(), None).unwrap();

        let failures: Vec<_> = outcomes
            .iter()
            .filter_map(|o| match o {
                RowOutcome::Failed(f) => Some(f),
                RowOutcome::Extracted { .. } => None,
            })
            .collect();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].row, 1);
        assert!(failures[0].reason.contains("amount"));

        let extracted_count = outcomes
            .iter()
            .filter(|o| matches!(o, RowOutcome::Extracted { .. }))
            .count();
        assert_eq!(extracted_count, 11);
    }
}
