//! Top-level error type for the `csv_import` crate.
//!
//! Each pipeline stage (decoding, parsing, extraction, rules, etc.) will
//! define its own focused error enum; this type aggregates them via
//! `#[from]` so callers only need to handle one error type.

/// Aggregate error type returned by the CSV import pipeline.
///
/// Sub-module error types are wired in here with `#[from]` as they are
/// introduced, so this stays the single error type library consumers (and
/// the CLI binary) need to match on.
#[derive(Debug, thiserror::Error)]
pub enum CsvImportError {}
