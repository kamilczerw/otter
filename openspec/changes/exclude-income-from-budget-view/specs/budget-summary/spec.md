## MODIFIED Requirements

### Requirement: Paid total reflects transaction magnitude, not raw signed sum
The system SHALL compute a budget entry's paid total as the sum of the absolute value of its expense (negative-amount) transactions only. Income (zero or positive amount) transactions SHALL NOT contribute to the paid total.

#### Scenario: entry with expense transactions reports a positive paid total
- **WHEN** a budget entry's transactions all have negative amounts
- **THEN** the entry's paid total is the positive sum of those amounts' magnitudes

#### Scenario: entry with income transactions reports a positive paid total
- **WHEN** a budget entry's transactions all have positive amounts
- **THEN** the entry's paid total is zero, since income transactions no longer contribute to the paid total

#### Scenario: entry with mixed expense and income transactions counts only the expenses
- **WHEN** a budget entry has both negative-amount and positive-amount transactions
- **THEN** the entry's paid total is the positive sum of only the negative-amount transactions' magnitudes

### Requirement: Budget status derivation is unaffected by transaction sign
The system SHALL derive a budget entry's status (unpaid, underspent, on-budget, overspent) by comparing its expense-only paid total against its budgeted amount, using the same comparison rules regardless of whether the entry also has income transactions.

#### Scenario: income entry with paid equal to budgeted is on-budget
- **WHEN** a budget entry's budgeted amount equals the sum of its (positive) income transaction amounts, and it has no expense transactions
- **THEN** the entry's status is unpaid, not on-budget, since income no longer counts toward paid — note this entry would in fact be excluded from the summary entirely under the new "entries with only income transactions are excluded" rule below

## ADDED Requirements

### Requirement: Entries with only income transactions are excluded from the summary
The system SHALL exclude a budget entry from the month's budget summary when the entry has at least one transaction and all of its transactions are income (zero or positive amount). An entry with no transactions, or with at least one expense transaction, SHALL still appear in the summary.

#### Scenario: income-only entry is excluded from the summary
- **WHEN** a budget entry has one or more transactions and all of them are income
- **THEN** the entry does not appear in the month's budget summary's entry list

#### Scenario: entry with no transactions still appears in the summary
- **WHEN** a budget entry has zero transactions
- **THEN** the entry appears in the month's budget summary's entry list

#### Scenario: entry with at least one expense transaction still appears in the summary
- **WHEN** a budget entry has both expense and income transactions
- **THEN** the entry appears in the month's budget summary's entry list, with its paid total reflecting only the expense transactions

### Requirement: Summary totals exclude income-only entries
The system SHALL compute the month's total budgeted, total paid, and remaining amounts from only the entries included in the summary (per the exclusion rule above), so income-only entries do not contribute to these totals.

#### Scenario: income-only entry's budgeted amount does not inflate the month total
- **WHEN** a month contains an income-only budget entry alongside other entries
- **THEN** the month's total budgeted, total paid, and remaining amounts are computed without that entry
