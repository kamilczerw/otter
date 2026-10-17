# transaction-ledger Specification

## Purpose
Lets a transaction's amount carry a meaningful sign so the system can distinguish money leaving an account (an expense) from money entering it (income), without requiring a separate account model yet.

## Requirements

### Requirement: Transactions accept any signed amount
The system SHALL accept any signed amount (negative, zero, or positive) when creating or updating a transaction. A negative amount represents money leaving the account (an expense); a zero or positive amount represents money entering the account (income).

#### Scenario: creating a transaction with a negative amount succeeds
- **WHEN** a transaction is created with a negative amount
- **THEN** the transaction is persisted with that negative amount and no validation error is raised

#### Scenario: creating a transaction with a positive amount succeeds
- **WHEN** a transaction is created with a positive amount
- **THEN** the transaction is persisted with that positive amount

#### Scenario: updating a transaction to a negative amount succeeds
- **WHEN** an existing transaction's amount is updated to a negative value
- **THEN** the update succeeds and the transaction's amount reflects the new negative value

### Requirement: Transaction direction is derived from amount sign
The system SHALL expose a transaction's direction (expense or income) derived solely from its amount's sign: a negative amount is an expense, a zero or positive amount is income. This direction is not independently stored and cannot disagree with the amount.

#### Scenario: negative amount reports as expense
- **WHEN** a transaction has a negative amount
- **THEN** its reported direction is expense

#### Scenario: positive amount reports as income
- **WHEN** a transaction has a positive amount
- **THEN** its reported direction is income

#### Scenario: zero amount reports as income
- **WHEN** a transaction has an amount of zero
- **THEN** its reported direction is income

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
