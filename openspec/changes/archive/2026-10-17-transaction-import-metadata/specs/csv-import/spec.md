## MODIFIED Requirements

### Requirement: Structured field extraction
The system SHALL extract merchant, city, country, card last-4-digits, transaction title, raw transaction type, and the raw (unparsed) original-amount fragment from each row's description columns, tolerating the two observed row shapes (card payment vs. internal transfer).

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
