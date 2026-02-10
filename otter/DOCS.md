# Otter Budget Tracker

A self-hosted household budget tracking application for Home Assistant.

## Features

- Track monthly budgets with categories
- Record transactions against budget entries
- View spending summaries and charts
- Multi-language support (English and Polish)

## Configuration

### Currency Code

The ISO 4217 currency code to use for display (e.g., PLN, EUR, USD).

### Minor Unit Name

The name of the smallest currency unit (e.g., "grosz" for PLN, "cent" for EUR/USD).

### Decimal Places

The number of decimal places used for currency display (typically 2).

## Data Storage

The SQLite database is stored in `/data/budget.db` using Home Assistant's persistent storage. Your data is preserved across add-on restarts and updates.

## Access

Once installed, the application appears in the Home Assistant sidebar with the configured icon and title. It is accessible through Home Assistant's Ingress system, which provides automatic authentication.
