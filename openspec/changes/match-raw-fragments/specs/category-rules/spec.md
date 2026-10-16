## MODIFIED Requirements

### Requirement: A match rule is an ordered, AND'ed list of criteria over six fields
The system SHALL represent a match rule as an ordered list of criteria, each naming exactly one of six fields (`merchant`, `raw_type`, `city`, `country`, `description`, `raw_fragments`) and a match type (`contains`: a case/diacritic-insensitive substring match, or `regex`: an unmodified regular expression), where a rule matches a transaction only if every criterion in the list matches.

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

#### Scenario: raw_fragments field matches the joined unmapped-fragments string
- **WHEN** a rule has a criterion (`contains` or `regex`) on the `raw_fragments` field
- **THEN** it is matched against the transaction's joined, `" | "`-separated raw-fragments string (the verbatim trailing CSV columns extraction didn't map into a typed field)

#### Scenario: raw_fragments criterion against a transaction with no raw fragments
- **WHEN** a rule has a criterion on the `raw_fragments` field and the transaction has no raw-fragments value (e.g. every trailing column was empty, or it has no import metadata at all)
- **THEN** the criterion does not match, the same as any criterion checked against a field the transaction doesn't have

### Requirement: A starting rule is derived from the transaction under review
For the transaction at the head of the queue, the system SHALL derive a starting match rule the same way interactive learning already does in the `csv_import` CLI: a `Contains` criterion on `merchant` (from that transaction's import metadata) when a merchant is present, or otherwise an exact match on `raw_type`. The derived starting rule SHALL NOT use the `raw_fragments` field; a user who wants to match on `raw_fragments` adds that criterion manually.

#### Scenario: transaction has a merchant
- **WHEN** the transaction at the head of the queue has a non-empty merchant in its import metadata
- **THEN** the derived starting rule is a single `Contains` criterion on the `merchant` field using that value

#### Scenario: transaction has no merchant
- **WHEN** the transaction at the head of the queue has no merchant in its import metadata
- **THEN** the derived starting rule is a single criterion on the `raw_type` field matching that transaction's raw type exactly

#### Scenario: transaction has only a raw_fragments value and no merchant
- **WHEN** the transaction at the head of the queue has no merchant in its import metadata but does have a `raw_fragments` value
- **THEN** the derived starting rule is still the `raw_type` fallback, not a `raw_fragments` criterion
