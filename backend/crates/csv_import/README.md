# csv_import

Prototype CLI that imports a bank CSV export, decodes it from Windows-1250,
extracts structured transaction fields (merchant, city, country, card last 4
digits, title), and categorizes each transaction against a hand-editable,
self-learning TOML rules file.

It does not touch the database or the web app — output is a spending
breakdown printed to stdout. The core logic lives in the crate's library
(`src/lib.rs` and friends) so it can be reused by the `api` crate later; the
binary is a thin CLI wrapper around it.

## Usage

```bash
cargo run -p csv_import -- \
  --input statement.csv \
  --rules-file rules.toml
```

or, via the repo's `Justfile`:

```bash
just csv-import --input statement.csv --rules-file rules.toml
```

### Options

| Flag               | Description                                                                 | Default  |
| ------------------ | ---------------------------------------------------------------------------- | -------- |
| `--input`           | Path to the bank CSV export (Windows-1250 encoded)                          | required |
| `--rules-file`      | Path to the TOML rules file (read and updated as new categories are learned) | required |
| `--format`          | Output format for the breakdown: `pretty` or `json`                         | `pretty` |
| `--no-interactive`  | Skip prompting for unmatched transactions; assign `Uncategorized` and leave the rules file untouched | off |

On a first run with no rules file yet, pass a path that doesn't exist —
it's created once a rule is learned (unless `--no-interactive` is set).

### Interactive categorization

When a transaction matches no existing rule, the CLI prints its extracted
fields and prompts for a category on stdin:

```
No rule matched this transaction:
  date:     2026-10-10
  amount:   -10310
  merchant: NETTO 5567 K.2
  city:     CIESZYN
  country:  POLSKA
  raw_type: Płatność kartą
Category: 
```

(`amount` is the raw minor-unit value, e.g. grosz for PLN — `-10310` is
-103.10 PLN; only the breakdown output formats it as a decimal.)

Your answer is saved as a new rule (matching on merchant, or on the raw
transaction type for transfers with no merchant) and reused automatically for
later transactions in the same run and in future runs.

### Rules file format

```toml
[[matching_rules]]
category = "Groceries"
[matching_rules.match]
merchant_contains = "NETTO"

[[matching_rules]]
category = "Savings"
[matching_rules.match]
raw_type = "Autooszczędzanie"
```

Rules are matched top-to-bottom (first match wins) against the normalized
(uppercased, diacritic-stripped) transaction fields, so `NETTO` also matches
`Netto`. The file is plain TOML — edit it by hand to broaden, merge, or
reorder rules. The top-level `matching_rules` key is required (TOML has no
bare top-level array), and this same shape is reused by the backend's
`[transaction.import]` config section.
