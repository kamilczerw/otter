# transaction-import Specification

## Purpose
Imports a bank CSV export directly into the application's database as real transactions, categorizing and auto-provisioning the months/categories/budget entries they belong to, without requiring any manual setup first.

## Requirements

### Requirement: CSV upload endpoint
The system SHALL expose an endpoint that accepts a bank CSV export as a multipart file upload, decodes it using the charset declared on the uploaded file part's `Content-Type` header (defaulting to UTF-8 when no charset is declared), and rejects the request with an error if an explicitly declared charset is not supported.

#### Scenario: upload with explicit charset
- **WHEN** a CSV file is uploaded with `Content-Type: text/csv; charset=windows-1250`
- **THEN** the file is decoded as Windows-1250 before parsing

#### Scenario: upload with no charset declared
- **WHEN** a CSV file is uploaded with no charset on its `Content-Type`
- **THEN** the file is decoded as UTF-8

#### Scenario: upload with unsupported charset
- **WHEN** a CSV file is uploaded with a `charset` value the system does not support
- **THEN** the request is rejected as invalid and nothing is persisted

### Requirement: Structurally invalid CSV is rejected wholesale
The system SHALL reject the entire request, persisting nothing, when the uploaded content cannot be split into rows and columns at all (e.g. it is not CSV-shaped data).

#### Scenario: unparseable upload
- **WHEN** the uploaded content is not valid CSV structure
- **THEN** the request is rejected as invalid, no months/categories/entries/transactions are created, and no partial results are reported

### Requirement: Per-row failures are reported, not fatal
The system SHALL continue importing the remaining rows when an individual row fails to extract required fields (e.g. an unparseable date or amount), and SHALL report each such row in the response with enough detail to identify it (its position in the file, its raw content, and the reason it failed) rather than discarding it silently.

#### Scenario: one bad row among many good ones
- **WHEN** an uploaded file has 50 rows and one row has an unparseable amount
- **THEN** the other 49 rows are imported, the request succeeds, and the response lists the failed row's position, raw content, and failure reason

### Requirement: Rule-based categorization on import
The system SHALL categorize each successfully extracted transaction using the same rule-matching semantics as the CSV import rules engine (first-rule-wins, normalized field matching), and SHALL assign a transaction to a fixed `"uncategorized"` category when no rule matches, or when the matching rule's category is empty or otherwise not usable as a category name.

#### Scenario: transaction matches a configured rule
- **WHEN** an imported transaction's fields match a configured rule
- **THEN** the transaction is persisted under that rule's category

#### Scenario: no rule matches
- **WHEN** an imported transaction's fields match no configured rule
- **THEN** the transaction is persisted under the `"uncategorized"` category

#### Scenario: matching rule has an empty or invalid category
- **WHEN** an imported transaction matches a rule whose category is empty or otherwise invalid
- **THEN** the transaction is persisted under the `"uncategorized"` category, the same as if no rule had matched

### Requirement: Auto-provisioning of months, categories, and budget entries
The system SHALL create, on demand, any month, category, or budget entry that a transaction needs but that does not yet exist: the month for the transaction's date (with no budget entries copied in), the category by its resolved name (including `"uncategorized"` itself), and the budget entry linking that month and category, initialized with a zero budgeted amount.

#### Scenario: importing into a month that doesn't exist yet
- **WHEN** an imported transaction's date falls in a month that has no existing month record
- **THEN** that month is created, with no budget entries copied into it, before the transaction is persisted

#### Scenario: importing a category seen for the first time
- **WHEN** an imported transaction resolves to a category name that doesn't exist yet
- **THEN** that category is created, and a budget entry for it in the transaction's month is created with a budgeted amount of zero, before the transaction is persisted

### Requirement: Duplicate transactions are skipped
The system SHALL NOT create a transaction that is identical (same budget entry, date, amount, and title) to one that already exists, and SHALL report the number of such duplicates skipped rather than treating them as a failure.

#### Scenario: re-importing an overlapping file
- **WHEN** a CSV file is imported twice, with some rows representing transactions already imported the first time
- **THEN** the second import does not create duplicate transaction records for those rows, and the response reports how many were skipped as duplicates

### Requirement: Import result reporting
The system SHALL report, for every import request that is not rejected wholesale, how many transactions were imported, how many were skipped as duplicates, how many were assigned to the `"uncategorized"` category, which months and categories were newly created, and the itemized list of rows that failed extraction.

#### Scenario: summary reflects the outcome
- **WHEN** an import request completes without being rejected wholesale
- **THEN** the response includes counts of imported, duplicate, and uncategorized transactions, the names of any newly created months and categories, and the list of failed rows (empty if none failed)

### Requirement: Response status reflects whether anything was created
The system SHALL respond indicating a resource was created when at least one transaction was persisted, and SHALL respond indicating success without creation when the request was well-formed but no transaction was newly persisted (for example, every row was a duplicate or failed extraction).

#### Scenario: at least one transaction imported
- **WHEN** an import request results in at least one newly persisted transaction
- **THEN** the response indicates a resource was created

#### Scenario: nothing new persisted
- **WHEN** an import request is well-formed but results in zero newly persisted transactions (e.g. a fully-duplicate re-import)
- **THEN** the response indicates success without indicating anything was created
