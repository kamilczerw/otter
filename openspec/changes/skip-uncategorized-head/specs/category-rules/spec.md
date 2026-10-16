## ADDED Requirements

### Requirement: Review queue head can be offset by a client-supplied skip count
The system SHALL, when the uncategorized review queue is requested with a `skip` count, return as `head` the transaction at position `skip modulo remaining` in the queue's stable order, so repeatedly incrementing `skip` advances through every transaction once before wrapping back to the start. `remaining` SHALL continue to report the total count of transactions currently assigned to `"uncategorized"`, unaffected by `skip`.

#### Scenario: skip advances past the current head
- **WHEN** the queue is requested with `skip` equal to 1
- **THEN** the returned `head` is the second transaction in the queue's stable order, and `remaining` is unchanged

#### Scenario: no skip behaves as before
- **WHEN** the queue is requested with no `skip` value (or `skip` equal to 0)
- **THEN** `head` is the first transaction in queue order, exactly as if skip had never been introduced

#### Scenario: skip wraps back to the start
- **WHEN** the queue is requested with `skip` equal to or greater than `remaining`
- **THEN** the returned `head` wraps back into the queue (position `skip modulo remaining`), rather than returning no transaction

#### Scenario: skip has no effect on an empty queue
- **WHEN** no transaction is currently assigned to `"uncategorized"`
- **THEN** `head` is `null` and `remaining` is zero regardless of the requested `skip` value

### Requirement: Head transaction card shows the raw import description
The system SHALL render the head transaction's `metadata.raw_description`, when present, as part of the review card, truncated to a single line with an ellipsis, with a tap/click control that reveals the full text. Absence of `raw_description` SHALL NOT display an empty row or placeholder.

#### Scenario: raw description is present
- **WHEN** the head transaction's import metadata has a non-null `raw_description`
- **THEN** the card displays it truncated to one line

#### Scenario: truncated description is expanded
- **WHEN** the user taps/clicks the truncated description
- **THEN** the full, untruncated text is displayed in its place

#### Scenario: raw description is absent
- **WHEN** the head transaction's import metadata has a null `raw_description` (or no metadata at all)
- **THEN** no description row is rendered

### Requirement: Skip offset reloads the queue regardless of how it changes
The system SHALL reload the queue whenever the effective `skip` value changes, whether that change originates from the Skip button, a direct edit to the route's `skip` query parameter, browser back/forward navigation, or any other same-document route change — not only from the Skip button's own click handler.

#### Scenario: direct navigation to a nonzero skip
- **WHEN** the view is already mounted and the route's `skip` query parameter changes to a new value by any means other than the Skip button
- **THEN** the queue reloads and displays the transaction at the new skip position

#### Scenario: Skip button still works
- **WHEN** the user clicks the Skip button
- **THEN** the queue reloads and displays the next transaction, exactly as before this change

### Requirement: Skip offset survives a successful save
The system SHALL preserve the current raw `skip` value after a rule is successfully saved, rather than resetting it to 0. The preserved value is not adjusted for how many transactions the save removed from the queue.

#### Scenario: skip is unchanged after save
- **WHEN** a rule is saved successfully while `skip` is a nonzero value N
- **THEN** the queue reloads using `skip` = N (not 0), and the displayed "skipped: N" count is unchanged
