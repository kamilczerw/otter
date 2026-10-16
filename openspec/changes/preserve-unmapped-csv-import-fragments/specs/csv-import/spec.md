## MODIFIED Requirements

### Requirement: Structured field extraction
The system SHALL extract merchant, city, country, card last-4-digits, transaction title, raw transaction type, and the raw (unparsed) original-amount fragment from each row's description columns, tolerating the two observed row shapes (card payment vs. internal transfer). It SHALL additionally capture a verbatim join of every trailing description-fragment column (columns 8–13), separated by `" | "` and skipping empty columns, so that content from row shapes not otherwise recognized by this requirement is still preserved rather than silently discarded.

#### Scenario: card payment row extraction
- **WHEN** a row has `Typ transakcji` = `Płatność kartą` with description columns containing `Tytuł:`, `Lokalizacja: Adres: ... Miasto: <city> Kraj: <country>`, and `Numer karty: <masked>`
- **THEN** the parsed transaction has `merchant` set from the address fragment, `city` and `country` set from `Miasto:`/`Kraj:`, `card_last4` set to the last 4 digits of the card number, and `title` set from the `Tytuł:` fragment

#### Scenario: transfer row with no merchant
- **WHEN** a row has `Typ transakcji` = `Autooszczędzanie` and no `Tytuł:`/`Lokalizacja:`/`Numer karty:` fragments present
- **THEN** the parsed transaction has `merchant`, `city`, `country`, and `card_last4` set to absent/empty, and `raw_type` and `raw_description` are still populated

#### Scenario: foreign-currency row carries an original-amount fragment
- **WHEN** a row has a non-empty original-amount column (e.g. a foreign-currency purchase showing the amount in the original currency before conversion)
- **THEN** the parsed transaction's original-amount fragment is set to that column's raw text, unparsed

#### Scenario: row with no original-amount column
- **WHEN** a row's original-amount column is empty
- **THEN** the parsed transaction's original-amount fragment is absent

#### Scenario: row shape not recognized by structured extraction still preserves its trailing columns
- **WHEN** a row has a `Typ transakcji` whose trailing columns don't match any of the recognized fragment patterns (e.g. a `Płatność web - kod mobilny` row carrying a `Numer telefonu:` phone fragment and a `Numer referencyjny:` reference-number fragment instead of `Lokalizacja:`/`Numer karty:`)
- **THEN** `merchant`, `city`, `country`, and `card_last4` are absent as usual, but the parsed transaction's raw-fragments field contains the verbatim, `" | "`-joined content of every non-empty trailing column, so the phone and reference-number text is not lost

#### Scenario: all trailing columns empty
- **WHEN** a row's columns 8–13 are all empty (e.g. a plain transfer row)
- **THEN** the parsed transaction's raw-fragments field is absent

#### Scenario: raw-fragments field is populated even when extraction fully succeeds
- **WHEN** a row's trailing columns are fully recognized and extracted into `merchant`, `city`, `country`, and `card_last4` (e.g. the card payment row above)
- **THEN** the parsed transaction's raw-fragments field is still populated with the verbatim joined columns, independent of whether structured extraction also succeeded
