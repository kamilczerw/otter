//! Thin CLI entry point for `csv_import`.
//!
//! Parses command-line arguments; the actual import/categorization pipeline
//! is implemented in the library crate and wired in by later tasks.

use clap::Parser;
use csv_import::Cli;

/// Parses CLI arguments and exits with clap's usage error on invalid input.
///
/// The import/categorization pipeline is not implemented yet, so this is
/// currently a no-op beyond argument validation.
fn main() {
    let _cli = Cli::parse();
}
