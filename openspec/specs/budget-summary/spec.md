# budget-summary Specification

## Purpose
Keeps a budget entry's paid total, and the status derived from it, correct regardless of whether its transactions are expenses or income, now that transaction amounts can be signed.

## Requirements

### Requirement: Paid total reflects transaction magnitude, not raw signed sum
The system SHALL compute a budget entry's paid total as the sum of the absolute value of its transactions' amounts, so the paid total is always a non-negative magnitude regardless of whether the entry's transactions are expenses (negative amounts) or income (positive amounts).

#### Scenario: entry with expense transactions reports a positive paid total
- **WHEN** a budget entry's transactions all have negative amounts
- **THEN** the entry's paid total is the positive sum of those amounts' magnitudes

#### Scenario: entry with income transactions reports a positive paid total
- **WHEN** a budget entry's transactions all have positive amounts
- **THEN** the entry's paid total is the positive sum of those amounts

### Requirement: Budget status derivation is unaffected by transaction sign
The system SHALL derive a budget entry's status (unpaid, underspent, on-budget, overspent) by comparing its paid total against its budgeted amount, using the same comparison rules regardless of whether the entry's transactions are expenses or income.

#### Scenario: income entry with paid equal to budgeted is on-budget
- **WHEN** a budget entry's budgeted amount equals the sum of its (positive) transaction amounts
- **THEN** the entry's status is on-budget, not overspent
