//! CLI entry point for `csv_import`: reads a bank CSV export, decodes it,
//! extracts structured transactions, categorizes them against a
//! user-maintained rules file (prompting to learn new rules interactively
//! unless `--no-interactive` is set), and prints a per-category breakdown.

use std::path::Path;

use clap::Parser;
use csv_import::{
    aggregate, categorize_or_learn, extract_transaction, load_rules_or_empty, parse_rows,
    print_breakdown, read_and_decode_windows1250, report_warnings, Category, Cli, CsvImportError,
    ParsedTransaction, Rule,
};

fn main() -> Result<(), CsvImportError> {
    let cli = Cli::parse();
    if let Err(err) = run(&cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

/// Runs the full import pipeline: decode, parse, extract, categorize, print.
fn run(cli: &Cli) -> Result<(), CsvImportError> {
    let transactions = read_transactions(&cli.input)?;

    let mut rules = load_rules_or_empty(&cli.rules_file)?;
    let interactive = !cli.no_interactive;
    let categorized = categorize_all(&transactions, &mut rules, &cli.rules_file, interactive)?;

    let breakdown = aggregate(&categorized);
    print_breakdown(&breakdown, &categorized, cli.format)
}

/// Reads, decodes and parses the input file into extracted transactions,
/// printing a warning for every row that's skipped (malformed CSV shape or
/// unparseable required fields) rather than aborting the whole run.
fn read_transactions(input: &Path) -> Result<Vec<ParsedTransaction>, CsvImportError> {
    let csv_text = read_and_decode_windows1250(input)?;
    let (rows, warnings) = parse_rows(&csv_text)?;
    report_warnings(&warnings);

    let mut transactions = Vec::with_capacity(rows.len());
    for row in &rows {
        match extract_transaction(row) {
            Ok(transaction) => transactions.push(transaction),
            Err(err) => eprintln!("warning: skipping row: {err}"),
        }
    }
    Ok(transactions)
}

/// Categorizes every transaction against `rules`, learning new rules
/// interactively as needed (see [`categorize_or_learn`]).
fn categorize_all(
    transactions: &[ParsedTransaction],
    rules: &mut Vec<Rule>,
    rules_file: &Path,
    interactive: bool,
) -> Result<Vec<(ParsedTransaction, Category)>, CsvImportError> {
    transactions
        .iter()
        .map(|transaction| {
            let category = categorize_or_learn(transaction, rules, rules_file, interactive)?;
            Ok((transaction.clone(), category))
        })
        .collect()
}
