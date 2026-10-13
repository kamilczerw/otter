## Purpose

Lets a transaction's amount carry a meaningful sign so the system can distinguish money leaving an account (an expense) from money entering it (income), without requiring a separate account model yet.

## ADDED Requirements

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
