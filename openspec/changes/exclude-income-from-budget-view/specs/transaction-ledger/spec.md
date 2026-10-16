## ADDED Requirements

### Requirement: Transactions can be filtered by direction when listed
The system SHALL support filtering a transaction listing by direction (expense or income), returning only transactions whose derived direction matches the requested filter. When no direction filter is given, transactions of both directions SHALL be returned.

#### Scenario: filtering by expense returns only negative-amount transactions
- **WHEN** a transaction listing is requested with an expense direction filter
- **THEN** only transactions with negative amounts are returned

#### Scenario: filtering by income returns only non-negative-amount transactions
- **WHEN** a transaction listing is requested with an income direction filter
- **THEN** only transactions with zero or positive amounts are returned

#### Scenario: no direction filter returns all transactions
- **WHEN** a transaction listing is requested without a direction filter
- **THEN** transactions of both directions are returned
