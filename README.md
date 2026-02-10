# Otter Budget Tracker

A self-hosted household budget tracking application, available as a **Home Assistant add-on** or as a standalone **Docker** deployment.

Otter helps you manage your family's monthly budget by organizing spending into categories, tracking transactions against budget entries, and visualizing where your money goes.

## Features

- **Monthly budget tracking** — Create monthly budgets with categories and planned amounts. New months automatically copy budget entries from the previous month.
- **Transaction recording** — Log payments against budget entries with dates and notes.
- **Spending summaries and charts** — Visualize spending vs. budgeted amounts per category and month.
- **Multi-language support** — English and Polish interfaces via vue-i18n.
- **Configurable currency** — Set your currency code, minor unit name, and decimal places (defaults to PLN).
- **Mobile-first design** — Built with Vuetify (Material Design) and optimized for mobile use.

## Installation as a Home Assistant Add-on

### Prerequisites

- Home Assistant **2024.1.0** or newer
- Home Assistant Supervisor (the add-on system requires a supervised installation — Home Assistant OS or Home Assistant Supervised)

### Step 1: Add the repository

1. In Home Assistant, navigate to **Settings > Add-ons > Add-on Store**.
2. Click the three-dot menu in the top right and select **Repositories**.
3. Add the following repository URL:
   ```
   https://github.com/kamilczerw/squirrel
   ```
4. Click **Add** and then **Close**.

### Step 2: Install the add-on

1. Find **Otter Budget Tracker** in the add-on store (you may need to refresh the page).
2. Click on it and then click **Install**.

### Step 3: Configure

Before starting the add-on, configure the currency settings under the **Configuration** tab:

| Option | Description | Default |
|--------|-------------|---------|
| `currency_code` | ISO 4217 currency code (e.g., `PLN`, `EUR`, `USD`) | `PLN` |
| `currency_minor_unit_name` | Name of the smallest currency unit (e.g., `grosz`, `cent`) | `grosz` |
| `currency_decimal_places` | Number of decimal places for currency display | `2` |

### Step 4: Start

1. Click **Start** to launch the add-on.
2. Once running, Otter appears in the Home Assistant **sidebar** with a cash register icon.
3. Click the sidebar entry to open the application. Authentication is handled automatically through Home Assistant's Ingress system.

### Data storage

The SQLite database is stored at `/data/budget.db` using Home Assistant's persistent storage. Your data is preserved across add-on restarts and updates.

To back up your data, use Home Assistant's built-in backup feature, which includes add-on data.

### Supported architectures

- `amd64`
- `aarch64`

## Standalone Docker Deployment

If you want to run Otter outside of Home Assistant, you can use Docker Compose.

### Prerequisites

- Docker and Docker Compose

### Running

```bash
docker compose up -d
```

The application will be available at `http://localhost:3000`.

The SQLite database is persisted in a Docker volume (`budget-data`).

### Configuration

The standalone deployment uses environment variables with the `APP__` prefix (double underscores as separators):

```bash
# Example overrides
APP__CURRENCY__CODE=EUR
APP__CURRENCY__MINOR_UNIT_NAME=cent
APP__CURRENCY__DECIMAL_PLACES=2
APP__DATABASE__URL=sqlite:///data/budget.db
```

Add these to the `environment` section of `docker-compose.yml` as needed.

## Development

### Backend (Rust)

```bash
cd backend
cargo build       # Build all workspace members
cargo test        # Run all tests
cargo run --bin api  # Start the API server on port 3000
```

### Frontend (Vue 3 + TypeScript)

```bash
cd frontend
npm install
npm run dev       # Start Vite dev server on port 5173
npm run build     # Type-check and build for production
```

### Architecture

The backend follows hexagonal (ports & adapters) architecture as a Rust workspace:

- **`domain`** — Pure business logic, domain types, and repository traits. No framework dependencies.
- **`db`** — SQLite adapter implementing domain repository traits via SQLx.
- **`api`** — HTTP/REST layer using Axum, with request/response types separate from domain entities.

The frontend is a Vue 3 single-page application using the Composition API, Vuetify for UI components, and Chart.js for data visualization.

## License

See the repository for license details.
