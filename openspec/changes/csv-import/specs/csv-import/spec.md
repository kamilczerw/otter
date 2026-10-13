## Purpose

Parse a bank's CSV transaction export into structured transactions and assign each one a spending category, using a rules engine that can be hand-edited and that learns from the user's interactive corrections, producing a categorized spending breakdown.

## ADDED Requirements

### Requirement: CSV decoding and parsing
The system SHALL read a bank CSV export encoded in Windows-1250, decode it to UTF-8, and parse it into rows using the existing header structure (named columns plus trailing unnamed description-fragment columns).

#### Scenario: Windows-1250 input decodes correctly
- **WHEN** the CLI is run with `--input` pointing to a CSV file encoded in Windows-1250 containing Polish diacritics (e.g. `Autooszczędzanie`, `Płatność kartą`)
- **THEN** all output (prompts, breakdown, exported data) renders those characters correctly in UTF-8, with no mojibake

#### Scenario: malformed row is skipped with a warning
- **WHEN** a row in the input CSV has fewer columns than the header or cannot be parsed
- **THEN** the tool prints a warning identifying the row and continues processing the remaining rows, rather than aborting

### Requirement: Structured field extraction
The system SHALL extract merchant, city, country, card last-4-digits, transaction title, and raw transaction type from each row's description columns, tolerating the two observed row shapes (card payment vs. internal transfer).

#### Scenario: card payment row extraction
- **WHEN** a row has `Typ transakcji` = `Płatność kartą` with description columns containing `Tytuł:`, `Lokalizacja: Adres: ... Miasto: <city> Kraj: <country>`, and `Numer karty: <masked>`
- **THEN** the parsed transaction has `merchant` set from the address fragment, `city` and `country` set from `Miasto:`/`Kraj:`, `card_last4` set to the last 4 digits of the card number, and `title` set from the `Tytuł:` fragment

#### Scenario: transfer row with no merchant
- **WHEN** a row has `Typ transakcji` = `Autooszczędzanie` and no `Tytuł:`/`Lokalizacja:`/`Numer karty:` fragments present
- **THEN** the parsed transaction has `merchant`, `city`, `country`, and `card_last4` set to absent/empty, and `raw_type` and `raw_description` are still populated

### Requirement: Rule-based categorization
The system SHALL categorize each parsed transaction by matching it against rules loaded from a user-specified YAML file, where a rule matches on any combination of normalized fields (`merchant_contains`, `city`, `country`, `raw_type`), using first-rule-wins ordering.

#### Scenario: transaction matches an existing rule
- **WHEN** a parsed transaction's normalized merchant contains a `merchant_contains` pattern from a rule in the `--rules-file`
- **THEN** the transaction is assigned that rule's `category` without any prompt

#### Scenario: field normalization ignores case and diacritics
- **WHEN** a rule's pattern and the transaction's field differ only in letter case or diacritics (e.g. rule `"CIESZYN"` vs. field value `"Cieszyn"`)
- **THEN** the rule still matches

### Requirement: Interactive learning on unmatched transactions
The system SHALL, when no rule matches a transaction and `--no-interactive` is not set, prompt the user for a category and append a new rule derived from that transaction to the rules file; when `--no-interactive` is set, it SHALL instead assign the category `Uncategorized` without prompting.

#### Scenario: interactive prompt on unmatched transaction
- **WHEN** no rule matches a transaction and `--no-interactive` is not passed
- **THEN** the tool pauses, displays the transaction's extracted fields, prompts for a category, and after the user answers, appends a new rule (`merchant_contains` on the normalized merchant, or `raw_type` if there is no merchant) mapping to that category to the rules file, then continues processing

#### Scenario: non-interactive run leaves transactions uncategorized
- **WHEN** no rule matches a transaction and `--no-interactive` is passed
- **THEN** the transaction is assigned category `Uncategorized`, no prompt is shown, and the rules file is not modified

### Requirement: Spending breakdown output
The system SHALL output a breakdown of categorized transactions, grouped by category with total amount and transaction count, in either `pretty` (table) or `json` format as selected by `--format`.

#### Scenario: pretty format
- **WHEN** the CLI is run with `--format pretty` (or no `--format` flag, since `pretty` is the default)
- **THEN** stdout contains a human-readable table listing each category with its total amount and transaction count

#### Scenario: json format
- **WHEN** the CLI is run with `--format json`
- **THEN** stdout contains a JSON document with the per-category breakdown and per-transaction extracted fields, suitable for piping into another tool
