//! Top-level error type for the `csv_import` crate.
//!
//! Each pipeline stage (decoding, parsing, extraction, rules, etc.) will
//! define its own focused error enum; this type aggregates them via
//! `#[from]` so callers only need to handle one error type.

use crate::csv_input::CsvParseError;
use crate::decode::DecodeError;
use crate::extract::ExtractError;

/// Aggregate error type returned by the CSV import pipeline.
///
/// Sub-module error types are wired in here with `#[from]` as they are
/// introduced, so this stays the single error type library consumers (and
/// the CLI binary) need to match on.
#[derive(Debug, thiserror::Error)]
pub enum CsvImportError {
    /// The input bytes could not be decoded as Windows-1250.
    #[error(transparent)]
    Decode(#[from] DecodeError),

    /// The decoded CSV text could not be parsed.
    #[error(transparent)]
    CsvParse(#[from] CsvParseError),

    /// A row's always-present fields (`Kwota`, `Data waluty`) could not be
    /// extracted into a structured transaction.
    #[error(transparent)]
    Extract(#[from] ExtractError),
}
