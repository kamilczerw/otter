//! Library crate for importing bank CSV exports and categorizing their
//! transactions against a user-maintained rules file.
//!
//! This crate is consumed both by the `csv_import` CLI binary and, in the
//! future, by the backend `api` crate. Modules are added here as each
//! pipeline stage (decoding, parsing, extraction, rules, output) is
//! implemented.

mod cli;
mod error;

pub use cli::{Cli, OutputFormat};
pub use error::CsvImportError;
