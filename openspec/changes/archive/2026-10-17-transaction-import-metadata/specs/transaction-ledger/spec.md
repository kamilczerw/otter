## ADDED Requirements

### Requirement: Transactions carry tags
The system SHALL store a list of tags on every transaction, defaulting to an empty list, and SHALL expose that list unchanged on read.

#### Scenario: transaction created with no tags
- **WHEN** a transaction is created without specifying tags
- **THEN** it is persisted with an empty tag list

#### Scenario: transaction created with tags
- **WHEN** a transaction is created with a non-empty tag list
- **THEN** it is persisted with exactly that tag list, and reading it back returns the same list

### Requirement: Transactions carry optional import-sourced metadata
The system SHALL store an optional metadata record on every transaction describing where its data came from (which source produced it, and source-specific details such as merchant, city, country, card last-4, raw type, currency, raw description, and a raw original-amount fragment), and SHALL leave this metadata absent for transactions not created by an import process.

#### Scenario: manually created transaction has no import metadata
- **WHEN** a transaction is created through manual entry rather than an import process
- **THEN** its metadata is absent

#### Scenario: transaction created with import metadata
- **WHEN** a transaction is created with an import metadata record attached
- **THEN** reading it back returns that same metadata record unchanged
