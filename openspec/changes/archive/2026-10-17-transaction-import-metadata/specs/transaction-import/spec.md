## ADDED Requirements

### Requirement: Imported transactions are tagged and carry import metadata
The system SHALL tag every transaction created through CSV import with `imported/csv`, and SHALL attach an import metadata record built from that row's extracted fields (merchant, city, country, card last-4, raw type, currency, raw description, and raw original-amount fragment), identifying the import source.

#### Scenario: imported transaction is tagged
- **WHEN** a transaction is created through CSV import
- **THEN** its tag list includes `imported/csv`

#### Scenario: imported transaction carries metadata from its row
- **WHEN** a transaction is created through CSV import from a row with extracted merchant, city, and country values
- **THEN** the transaction's import metadata reflects those same values and identifies CSV import as the source

#### Scenario: fields absent from the row are absent from the metadata
- **WHEN** a transaction is created through CSV import from a row where a given field (e.g. merchant) could not be extracted
- **THEN** that field is absent from the transaction's import metadata rather than causing an error
