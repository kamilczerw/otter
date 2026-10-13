## Purpose

Lets a user clear out the backlog of `"uncategorized"` transactions by reviewing them one at a time, deriving a reusable matching rule from each, and applying that rule immediately to every other uncategorized transaction it matches, without editing a config file or restarting the server.

## ADDED Requirements

### Requirement: Uncategorized review queue spans all months
The system SHALL expose the full set of transactions currently assigned to the `"uncategorized"` category as a single queue, regardless of which month they belong to, along with the total count remaining.

#### Scenario: queue includes transactions from multiple months
- **WHEN** uncategorized transactions exist in more than one month
- **THEN** the queue and its count include all of them, not just the current or most recent month

#### Scenario: queue is empty
- **WHEN** no transaction is currently assigned to `"uncategorized"`
- **THEN** the queue is empty and its count is zero

### Requirement: A starting rule is derived from the transaction under review
For the transaction at the head of the queue, the system SHALL derive a starting match rule the same way interactive learning already does in the `csv_import` CLI: a `Contains` criterion on `merchant` (from that transaction's import metadata) when a merchant is present, or otherwise an exact match on `raw_type`.

#### Scenario: transaction has a merchant
- **WHEN** the transaction at the head of the queue has a non-empty merchant in its import metadata
- **THEN** the derived starting rule is a single `Contains` criterion on the `merchant` field using that value

#### Scenario: transaction has no merchant
- **WHEN** the transaction at the head of the queue has no merchant in its import metadata
- **THEN** the derived starting rule is a single criterion on the `raw_type` field matching that transaction's raw type exactly

### Requirement: A match rule is an ordered, AND'ed list of criteria over five fields
The system SHALL represent a match rule as an ordered list of criteria, each naming exactly one of five fields (`merchant`, `raw_type`, `city`, `country`, `description`) and a match type (`contains`: a case/diacritic-insensitive substring match, or `regex`: an unmodified regular expression), where a rule matches a transaction only if every criterion in the list matches.

#### Scenario: single-criterion rule
- **WHEN** a rule has one `contains` criterion on `merchant` with pattern `"netto"`
- **THEN** it matches any transaction whose merchant contains "netto" (case/diacritic-insensitively), regardless of its other fields

#### Scenario: multi-criterion rule requires all criteria to match
- **WHEN** a rule has a `contains` criterion on `merchant` and a `contains` criterion on `city`
- **THEN** it matches a transaction only if both the merchant and the city criteria match that transaction

#### Scenario: regex criterion
- **WHEN** a rule has a `regex` criterion on `description` with a valid regular expression
- **THEN** it matches any transaction whose raw description matches that regular expression, with no case or diacritic normalization applied

#### Scenario: description field matches raw bank description
- **WHEN** a rule has a criterion on the `description` field
- **THEN** it is matched against the transaction's raw imported description text

### Requirement: Invalid regex is rejected at save time
The system SHALL reject a rule containing a `regex` criterion whose pattern does not compile as a valid regular expression, with an error identifying the invalid criterion, and SHALL NOT persist the rule or apply it to any transaction.

#### Scenario: malformed regex pattern
- **WHEN** a user attempts to save a rule containing a `regex` criterion with a syntactically invalid pattern
- **THEN** the save is rejected, no rule is persisted, and no transaction is re-categorized

### Requirement: Draft rule match count previews live against the current queue
The system SHALL, given a draft (not-yet-saved) rule, report how many of the currently-uncategorized transactions it matches, reflecting any edit to the draft rule's criteria.

#### Scenario: editing a draft rule changes the preview count
- **WHEN** a user narrows a draft rule's `merchant` criterion from a broad to a narrower substring
- **THEN** the previewed match count decreases to reflect only transactions matching the narrower pattern

### Requirement: Saving a rule re-categorizes every uncategorized transaction it matches
The system SHALL, when a rule is saved, persist it for future use and immediately re-categorize every transaction currently assigned to `"uncategorized"` that the rule matches, assigning each to the rule's target category within that transaction's own month (creating the month's budget entry for that category if it does not already exist), and SHALL reflect the resulting drop in the queue's remaining count.

#### Scenario: saved rule clears all its current matches
- **WHEN** a rule matching 20 of the 100 currently-uncategorized transactions is saved with target category "groceries"
- **THEN** all 20 matching transactions are re-categorized to "groceries" in their own months, and the queue's remaining count becomes 80

#### Scenario: saved rule applies to future imports
- **WHEN** a rule is saved
- **THEN** a transaction imported afterward that matches the rule is categorized accordingly, without any server restart

### Requirement: Rules are read fresh, not cached at startup
The system SHALL resolve the current set of rules at the time each import, preview, or apply operation runs, rather than caching a fixed set of rules in memory for the life of the running server.

#### Scenario: a newly saved rule applies without restarting
- **WHEN** a rule is saved while the server is running
- **THEN** the very next CSV import, queue preview, or apply operation reflects that rule, with no server restart

### Requirement: Transactions with no import metadata are unreachable by rules
The system SHALL leave a transaction with no import metadata in the uncategorized queue when no rule matches it, since such a transaction has none of the five matchable fields populated.

#### Scenario: manually created uncategorized transaction
- **WHEN** a transaction with no import metadata is assigned to `"uncategorized"`
- **THEN** it remains in the queue regardless of what rules exist, since it has no merchant, raw type, city, country, or description for any criterion to match against
