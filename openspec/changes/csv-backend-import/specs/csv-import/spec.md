## MODIFIED Requirements

### Requirement: Rule-based categorization
The system SHALL categorize each parsed transaction by matching it against rules loaded from a user-specified TOML file, where a rule matches on any combination of normalized fields (`merchant_contains`, `city`, `country`, `raw_type`), using first-rule-wins ordering.

#### Scenario: transaction matches an existing rule
- **WHEN** a parsed transaction's normalized merchant contains a `merchant_contains` pattern from a rule in the `--rules-file`
- **THEN** the transaction is assigned that rule's `category` without any prompt

#### Scenario: field normalization ignores case and diacritics
- **WHEN** a rule's pattern and the transaction's field differ only in letter case or diacritics (e.g. rule `"CIESZYN"` vs. field value `"Cieszyn"`)
- **THEN** the rule still matches
