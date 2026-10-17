## Purpose

Lets a user view, create, edit, and delete the categorization rules that decide which category an imported transaction gets, and move rules between Otter instances via file import, without editing any file by hand.

## ADDED Requirements

### Requirement: List the full rule set in match order
The system SHALL expose the complete active rule set as two ordered groups: config-defined default rules (read-only), followed by the user's own stored rules, in the exact order they are evaluated (first-rule-wins) during transaction categorization.

#### Scenario: Listing shows defaults before user rules
- **WHEN** a client requests the rule set
- **THEN** the response lists every config-defined default rule, in their config-file order, followed by every user-stored rule, in creation order

#### Scenario: Empty user rule set
- **WHEN** a client requests the rule set and no user rule has ever been created
- **THEN** the response lists only the config-defined default rules, with an empty list of user rules

### Requirement: Create a user rule
The system SHALL allow a user to create a new rule by specifying one or more match criteria (each a field, and either a substring or regex pattern) and a target category, and SHALL evaluate it after every existing user rule (appended, never inserted ahead of an existing rule).

#### Scenario: Successful creation
- **WHEN** a user submits a new rule with valid match criteria and a category
- **THEN** the system stores it and it appears last among the user's rules in the next listing

#### Scenario: Invalid regex pattern rejected
- **WHEN** a user submits a rule whose match criteria includes a regex pattern that fails to compile
- **THEN** the system rejects the request, stores nothing, and reports which pattern is invalid

### Requirement: Edit a user rule in place
The system SHALL allow a user to change an existing user rule's match criteria and/or category without changing its position in the evaluation order, identified by the rule's own identity rather than its position.

#### Scenario: Successful edit
- **WHEN** a user submits updated match criteria and/or category for an existing user rule
- **THEN** the system stores the update and the rule keeps its prior position in the evaluation order

#### Scenario: Editing an unknown rule
- **WHEN** a user submits an edit for a rule identity that does not exist among the user's rules
- **THEN** the system rejects the request and makes no change

#### Scenario: Editing a default rule is not possible
- **WHEN** a user attempts to edit a config-defined default rule
- **THEN** the system rejects the request, since default rules are read-only

### Requirement: Delete a user rule
The system SHALL allow a user to permanently remove one of their own stored rules. Config-defined default rules SHALL NOT be deletable through this capability.

#### Scenario: Successful deletion
- **WHEN** a user deletes an existing user rule
- **THEN** the rule no longer appears in the rule set and no longer affects future categorization

#### Scenario: Deleting an unknown rule
- **WHEN** a user attempts to delete a rule identity that does not exist among the user's rules
- **THEN** the system rejects the request and makes no change

### Requirement: Import rules from a file
The system SHALL allow a user to upload a rules file (in the same portable format used for hand-editing/exporting rules today) and add its rules to their stored rule set, skipping any rule that is an exact duplicate (identical match criteria and category) of one they already have.

#### Scenario: Importing new rules
- **WHEN** a user uploads a file containing rules none of which duplicate an existing user rule
- **THEN** every rule in the file is added to the user's stored rules, and the system reports how many were imported

#### Scenario: Importing with some duplicates
- **WHEN** a user uploads a file where some rules exactly match existing user rules and others do not
- **THEN** only the non-duplicate rules are added, and the system reports how many were imported and how many were skipped as duplicates

#### Scenario: Importing a rule with an invalid pattern
- **WHEN** a user uploads a file containing a rule whose pattern fails to compile
- **THEN** that rule is not added, the rest of the file's valid, non-duplicate rules are still added, and the system reports which rule failed and why

### Requirement: Imported rules participate in categorization
The system SHALL use every stored user rule, exactly as it uses config-defined default rules today, when categorizing transactions during CSV import: config defaults first, then user rules, first-rule-wins, in the order shown by the listing.

#### Scenario: A rule created through this capability categorizes a new import
- **WHEN** a user has created or imported a rule matching some transactions, and a CSV import is later run
- **THEN** transactions matching that rule are assigned its category, exactly as if the rule had been present in a hand-edited rules file
