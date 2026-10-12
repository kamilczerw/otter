//! Command-line argument definitions for the `csv_import` binary.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Output format for the categorized breakdown produced by the import run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table printed to stdout.
    Pretty,
    /// Machine-readable JSON printed to stdout.
    Json,
}

/// CLI arguments for importing a bank CSV export and categorizing its
/// transactions against a rules file.
#[derive(Debug, Parser)]
#[command(name = "csv_import", about = "Import and categorize a bank CSV export")]
pub struct Cli {
    /// Path to the CSV export file to import.
    #[arg(long)]
    pub input: PathBuf,

    /// Path to the YAML rules file used to categorize transactions.
    #[arg(long)]
    pub rules_file: PathBuf,

    /// Output format for the resulting breakdown.
    #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
    pub format: OutputFormat,

    /// Disable interactive prompting for unmatched transactions; they are
    /// left as `Uncategorized` instead and the rules file is not modified.
    #[arg(long)]
    pub no_interactive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    /// Parsing the two required flags should succeed and apply the
    /// documented defaults for the optional ones.
    #[test]
    fn parses_required_args_with_defaults() {
        let_assert!(
            Ok(cli) = Cli::try_parse_from([
                "csv_import",
                "--input",
                "foo.csv",
                "--rules-file",
                "rules.yaml",
            ])
        );

        check!(cli.input == PathBuf::from("foo.csv"));
        check!(cli.rules_file == PathBuf::from("rules.yaml"));
        check!(cli.format == OutputFormat::Pretty);
        check!(!cli.no_interactive);
    }

    /// `--format` and `--no-interactive` should both be overridable.
    #[test]
    fn parses_overridden_format_and_no_interactive() {
        let_assert!(
            Ok(cli) = Cli::try_parse_from([
                "csv_import",
                "--input",
                "foo.csv",
                "--rules-file",
                "rules.yaml",
                "--format",
                "json",
                "--no-interactive",
            ])
        );

        check!(cli.format == OutputFormat::Json);
        check!(cli.no_interactive);
    }
}
