//! Library crate for importing bank CSV exports and categorizing their
//! transactions against a user-maintained rules file.
//!
//! This crate is consumed both by the `csv_import` CLI binary and, in the
//! future, by the backend `api` crate. Modules are added here as each
//! pipeline stage (decoding, parsing, extraction, rules, output) is
//! implemented.

mod breakdown;
mod cli;
mod csv_input;
mod decode;
mod error;
mod extract;
mod learn;
mod rules;

pub use breakdown::{aggregate, print_breakdown, render_json, render_pretty, Breakdown, CategoryTotal};
pub use cli::{Cli, OutputFormat};
pub use csv_input::{parse_rows, report_warnings, CsvParseError, ParseWarning, RawRow};
pub use decode::{decode_windows1250, read_and_decode_windows1250, DecodeError};
pub use error::CsvImportError;
pub use extract::{extract_transaction, ExtractError, ParsedTransaction};
pub use learn::{categorize_or_learn, derive_rule, prompt_for_category};
pub use rules::{
    categorize, load_rules, load_rules_or_empty, match_rule, normalize, save_rules, Category,
    Matchable, MatchCriteria, MatchCriterion, MatchField, MatchType, Rule, RulesError,
};
