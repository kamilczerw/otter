## MODIFIED Requirements

### Requirement: Rule-based categorization
The system SHALL categorize each parsed transaction by matching it against rules loaded from a user-specified TOML file, where a rule is an ordered, AND'ed list of criteria, each naming one of five fields (`merchant`, `raw_type`, `city`, `country`, `description`) and a match type (`contains`: a case/diacritic-insensitive substring match, or `regex`: an unmodified regular expression), using first-rule-wins ordering.

#### Scenario: transaction matches an existing rule
- **WHEN** a parsed transaction's fields satisfy every criterion of a rule loaded from the `--rules-file`
- **THEN** the transaction is assigned that rule's `category` without any prompt

#### Scenario: field normalization ignores case and diacritics
- **WHEN** a `contains` criterion's pattern and the transaction's field differ only in letter case or diacritics (e.g. pattern `"CIESZYN"` vs. field value `"Cieszyn"`)
- **THEN** the criterion still matches

#### Scenario: regex criteria apply no normalization
- **WHEN** a rule has a `regex` criterion
- **THEN** it is matched against the transaction's field exactly as written, with no case or diacritic folding applied

#### Scenario: description criterion matches raw description
- **WHEN** a rule has a criterion on the `description` field
- **THEN** it is matched against the transaction's raw extracted description text

### Requirement: Interactive learning on unmatched transactions
The system SHALL, when no rule matches a transaction and `--no-interactive` is not set, prompt the user for a category and append a new rule derived from that transaction to the rules file; when `--no-interactive` is set, it SHALL instead assign the category `Uncategorized` without prompting.

#### Scenario: interactive prompt on unmatched transaction
- **WHEN** no rule matches a transaction and `--no-interactive` is not passed
- **THEN** the tool pauses, displays the transaction's extracted fields, prompts for a category, and after the user answers, appends a new rule mapping to that category to the rules file, then continues processing: a single `contains` criterion on the `merchant` field (using the normalized merchant) when a merchant is present, or otherwise a single `contains` criterion on the `raw_type` field

#### Scenario: non-interactive run leaves transactions uncategorized
- **WHEN** no rule matches a transaction and `--no-interactive` is passed
- **THEN** the transaction is assigned category `Uncategorized`, no prompt is shown, and the rules file is not modified
