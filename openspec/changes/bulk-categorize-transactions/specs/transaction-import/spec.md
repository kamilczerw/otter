## MODIFIED Requirements

### Requirement: Rule-based categorization on import
The system SHALL categorize each successfully extracted transaction using the same rule-matching semantics as the CSV import rules engine (first-rule-wins, criteria-list matching over the `merchant`/`raw_type`/`city`/`country`/`description` fields), resolving the current set of rules fresh at the time each import runs rather than from a set cached since server startup, and SHALL assign a transaction to a fixed `"uncategorized"` category when no rule matches, or when the matching rule's category is empty or otherwise not usable as a category name.

#### Scenario: transaction matches a configured rule
- **WHEN** an imported transaction's fields match a configured rule
- **THEN** the transaction is persisted under that rule's category

#### Scenario: no rule matches
- **WHEN** an imported transaction's fields match no configured rule
- **THEN** the transaction is persisted under the `"uncategorized"` category

#### Scenario: matching rule has an empty or invalid category
- **WHEN** an imported transaction matches a rule whose category is empty or otherwise invalid
- **THEN** the transaction is persisted under the `"uncategorized"` category, the same as if no rule had matched

#### Scenario: a rule saved after server startup still applies
- **WHEN** a rule is saved (e.g. through the uncategorized review queue) after the server has already started, and a CSV import runs afterward
- **THEN** the import resolves rules fresh and applies the newly saved rule, with no server restart required
