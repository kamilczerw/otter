## ADDED Requirements

### Requirement: Draft rule preview can return a page of its actual matches
The system SHALL, given a draft rule and an optional page request (`offset`, `limit`), return that page of the currently-uncategorized transactions the rule matches, in the queue's existing stable order, alongside the existing match count. The system SHALL also accept an optional transaction id to exclude from both the returned page and its total, and SHALL report that total (excluding the specified transaction, if any) separately from the existing overall match count. Requesting no page SHALL return exactly the same response the count-only preview returns today.

#### Scenario: no page requested behaves as before
- **WHEN** a draft rule is previewed with no page request
- **THEN** the response contains only the overall match count, with no transaction list

#### Scenario: a page of matches is returned
- **WHEN** a draft rule matching 30 uncategorized transactions is previewed with `offset` 0 and `limit` 20
- **THEN** the response includes the first 20 matching transactions in queue order, in addition to the overall match count

#### Scenario: a transaction can be excluded from the page and its total
- **WHEN** a draft rule matching 30 uncategorized transactions is previewed with an excluded transaction id that is among the matches, `offset` 0, and `limit` 20
- **THEN** the returned page and the reported total both exclude that transaction, while the overall match count (unaffected by the exclusion) still reports 30

#### Scenario: paging past the end of the matches
- **WHEN** a draft rule's matches are previewed with an `offset` at or beyond the number of matches
- **THEN** the returned page is empty

### Requirement: Previewed match page size is bounded
The system SHALL cap the number of transactions returned in a single previewed page at a fixed server-side limit, regardless of the `limit` requested.

#### Scenario: requested limit exceeds the server cap
- **WHEN** a page is requested with a `limit` greater than the server's fixed cap
- **THEN** the returned page contains no more transactions than that cap
