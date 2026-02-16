# Implementation Plan: Transaction Title Field

## Overview

Add a nullable `title` field to transactions to provide additional context for each transaction. The title will be displayed in both the compact transaction list within expanded budget bars and the full month transaction table.

## Requirements Summary

- **Field**: Nullable string (max 50 characters, configurable)
- **Display locations**:
  - PanelTransactionList: Date | Title | Amount (3-column layout)
  - TransactionList: Add title column to table
- **Empty state**: Translated i18n message with faded styling
- **Architecture**: Follow hexagonal architecture (domain/db/api separation)
- **Code quality**: Single Responsibility Principle, full documentation, comprehensive testing

## Design Decisions

### 1. Title Storage and Validation

**Pattern**: Follow the `Category.label` approach - `Option<String>` without newtype wrapper

**Rationale**:
- Titles are free-form text without domain-specific validation rules
- Unlike `DueDay` (1-31 range) or `Money` (positive integers), titles don't need value object encapsulation
- Simplifies implementation while maintaining consistency with existing nullable string fields

**Normalization strategy**:
- Empty strings and whitespace-only strings converted to `NULL` in backend
- Prevents database pollution with meaningless values
- Centralizes normalization in domain service layer

### 2. Length Limit Implementation

**Approach**: Database constraint + backend validation + frontend hint

- Database: `VARCHAR(50)` column type (SQLite TEXT with length check in migration comment)
- Backend: Validation function `validate_title_length()` in service layer
- Frontend: `maxlength="50"` on input + character counter hint

**Configuration**: Define `MAX_TITLE_LENGTH` constant in:
- Backend: `domain/src/entities/transaction.rs`
- Frontend: `src/config/constants.ts` (create if not exists)

### 3. UI Display Strategy

**PanelTransactionList** (compact, expanded budget bars):
```
┌────────────────────────────────────────┐
│ 2026-02-14  |  Grocery shopping  | 125.50 zł │
│ 2026-02-12  |  No title         | 45.00 zł  │  ← faded italic
└────────────────────────────────────────┘
```

**TransactionList** (full month table):
```
Category        | Date       | Title              | Amount    | Actions
────────────────────────────────────────────────────────────────────────
Food/Groceries  | 2026-02-14 | Grocery shopping   | 125.50 zł | Edit Delete
Utils/Electric  | 2026-02-12 | No title          | 45.00 zł  | Edit Delete
                                 ↑ faded italic
```

### 4. Empty State Styling

**CSS approach**:
```css
.transaction-title-empty {
  font-style: italic;
  opacity: 0.5;  /* Faded effect */
}
```

**i18n keys**:
- `transactions.title` - "Title"
- `transactions.noTitle` - "No title" (EN) / "Brak tytułu" (PL)

---

## Implementation Phases

### Phase 1: Database Migration

**File**: `backend/crates/db/migrations/00X_add_transaction_title.sql`

```sql
-- Add nullable title column to transactions table
-- Max length: 50 characters (configurable via backend constant)
ALTER TABLE transactions ADD COLUMN title TEXT;

-- No index needed - title is not used for filtering/searching in Phase 1
```

**Testing**: Run migration on dev database, verify schema change

---

### Phase 2: Backend - Domain Layer

**Files to modify**:
1. `backend/crates/domain/src/entities/transaction.rs`
2. `backend/crates/domain/src/services/transaction_service.rs`

#### 2.1 Update Transaction Entity

**File**: `backend/crates/domain/src/entities/transaction.rs`

Add constant and update structs:

```rust
/// Maximum length for transaction title
pub const MAX_TITLE_LENGTH: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: ulid::Ulid,
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
    pub title: Option<String>,  // NEW FIELD
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
    pub title: Option<String>,  // NEW FIELD
}
```

**Documentation requirements**:
- Add doc comment explaining title is optional descriptive text
- Document MAX_TITLE_LENGTH constant

#### 2.2 Add Title Normalization and Validation

**File**: `backend/crates/domain/src/services/transaction_service.rs`

Add focused helper functions following SRP:

```rust
/// Normalizes title by trimming whitespace and converting empty strings to None.
///
/// # Arguments
///
/// * `title` - The title to normalize
///
/// # Returns
///
/// `None` if the title is empty/whitespace-only, otherwise `Some(trimmed_title)`
fn normalize_title(title: Option<String>) -> Option<String> {
    title.and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Validates that a title does not exceed the maximum allowed length.
///
/// # Arguments
///
/// * `title` - The title to validate
///
/// # Returns
///
/// `Ok(())` if valid, or `TransactionError::TitleTooLong` if too long
///
/// # Errors
///
/// Returns error if title length exceeds MAX_TITLE_LENGTH
fn validate_title_length(title: &Option<String>) -> Result<(), TransactionError> {
    if let Some(t) = title {
        if t.len() > MAX_TITLE_LENGTH {
            return Err(TransactionError::TitleTooLong {
                length: t.len(),
                max: MAX_TITLE_LENGTH,
            });
        }
    }
    Ok(())
}
```

Update service methods to use helpers:

```rust
pub async fn create(
    &self,
    entry_id: ulid::Ulid,
    amount: Money,
    date: TransactionDate,
    title: Option<String>,
) -> Result<Transaction, TransactionError> {
    validate_amount(&amount)?;

    let normalized_title = normalize_title(title);
    validate_title_length(&normalized_title)?;

    verify_entry_exists(&self.entry_repo, &entry_id).await?;

    let new_transaction = NewTransaction {
        entry_id,
        amount,
        date,
        title: normalized_title,
    };

    self.transaction_repo.create(new_transaction).await
}

pub async fn update(
    &self,
    id: &ulid::Ulid,
    entry_id: Option<ulid::Ulid>,
    amount: Option<Money>,
    date: Option<TransactionDate>,
    title: Option<Option<String>>,  // Triple nesting for "don't change" semantics
) -> Result<Transaction, TransactionError> {
    let normalized_title = title.map(normalize_title);

    if let Some(ref t) = normalized_title {
        validate_title_length(t)?;
    }

    // ... rest of update logic

    self.transaction_repo.update(id, entry_id, amount, date, normalized_title).await
}
```

#### 2.3 Add Error Variant

**File**: `backend/crates/domain/src/entities/transaction.rs`

Add to TransactionError enum:

```rust
#[derive(Debug, Error)]
pub enum TransactionError {
    // ... existing variants ...

    #[error("Title too long: {length} characters (max {max})")]
    TitleTooLong { length: usize, max: usize },
}
```

**Testing**:
- Unit test `normalize_title()`: empty string, whitespace, normal text
- Unit test `validate_title_length()`: under limit, at limit, over limit
- Integration test: create transaction with title, without title, with empty title

---

### Phase 3: Backend - Database Repository

**File**: `backend/crates/db/src/repos/transaction_repo.rs`

Update repository methods to handle title:

#### 3.1 Update Row Mapping

```rust
fn map_row_to_transaction(row: &sqlx::sqlite::SqliteRow) -> Result<Transaction, TransactionError> {
    // ... existing field mapping ...

    let title: Option<String> = row.get("title");

    Ok(Transaction {
        id,
        entry_id,
        amount,
        date,
        title,  // NEW FIELD
        created_at,
        updated_at,
    })
}
```

#### 3.2 Update Create Method

```rust
async fn create(&self, transaction: NewTransaction) -> Result<Transaction, TransactionError> {
    let id = ulid::Ulid::new();
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO transactions (id, entry_id, amount, date, title, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(transaction.entry_id.to_string())
    .bind(transaction.amount.value())
    .bind(transaction.date.to_string())
    .bind(&transaction.title)  // NEW BINDING
    .bind(&now)
    .bind(&now)
    .execute(&self.pool)
    .await;

    // ... error handling and fetch ...
}
```

#### 3.3 Update Update Method

```rust
async fn update(
    &self,
    id: &ulid::Ulid,
    entry_id: Option<ulid::Ulid>,
    amount: Option<Money>,
    date: Option<TransactionDate>,
    title: Option<Option<String>>,  // NEW PARAMETER
) -> Result<Transaction, TransactionError> {
    let mut set_clauses: Vec<String> = Vec::new();

    if entry_id.is_some() {
        set_clauses.push("entry_id = ?".to_string());
    }
    if amount.is_some() {
        set_clauses.push("amount = ?".to_string());
    }
    if date.is_some() {
        set_clauses.push("date = ?".to_string());
    }
    if title.is_some() {  // NEW CLAUSE
        set_clauses.push("title = ?".to_string());
    }

    if set_clauses.is_empty() {
        return self.find_by_id(id).await;
    }

    let sql = format!(
        "UPDATE transactions SET {} WHERE id = ?",
        set_clauses.join(", ")
    );

    let mut query = sqlx::query(&sql);

    if let Some(eid) = entry_id {
        query = query.bind(eid.to_string());
    }
    if let Some(amt) = amount {
        query = query.bind(amt.value());
    }
    if let Some(d) = date {
        query = query.bind(d.to_string());
    }
    if let Some(t) = title {  // NEW BINDING
        query = query.bind(t);  // Binds null or string value
    }

    query = query.bind(id.to_string());

    // ... execute and fetch ...
}
```

**Testing**:
- Integration test: Create transaction with title, verify stored correctly
- Integration test: Update title (set, clear, don't change)
- Integration test: List transactions, verify title returned

---

### Phase 4: Backend - API Layer

**Files to modify**:
1. `backend/crates/api/src/requests/mod.rs`
2. `backend/crates/api/src/responses/mod.rs`
3. `backend/crates/api/src/handlers/transactions.rs`

#### 4.1 Update Request Types

**File**: `backend/crates/api/src/requests/mod.rs`

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub entry_id: String,
    pub amount: i64,
    pub date: String,
    pub title: Option<String>,  // NEW FIELD
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTransactionRequest {
    pub entry_id: Option<String>,
    pub amount: Option<i64>,
    pub date: Option<String>,
    pub title: Option<Option<String>>,  // NEW FIELD - triple nesting
}
```

#### 4.2 Update Response Type

**File**: `backend/crates/api/src/responses/mod.rs`

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub id: String,
    pub entry_id: String,
    pub amount: i64,
    pub date: String,
    pub title: Option<String>,  // NEW FIELD
    pub created_at: String,
    pub updated_at: String,
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id.to_string(),
            entry_id: t.entry_id.to_string(),
            amount: t.amount.value(),
            date: t.date.to_string(),
            title: t.title,  // NEW FIELD
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}
```

#### 4.3 Update Handlers

**File**: `backend/crates/api/src/handlers/transactions.rs`

```rust
pub async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), ApiError> {
    let entry_ulid = parse_ulid(&req.entry_id)?;
    let amount = Money::new(req.amount);
    let date = parse_transaction_date(&req.date)?;
    let title = req.title;  // NEW PARAMETER

    let transaction = state
        .transaction_service
        .create(entry_ulid, amount, date, title)
        .await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTransactionRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let id_ulid = parse_ulid(&id)?;

    let entry_id = req.entry_id.map(|e| parse_ulid(&e)).transpose()?;
    let amount = req.amount.map(Money::new);
    let date = req.date.map(|d| parse_transaction_date(&d)).transpose()?;
    let title = req.title;  // NEW PARAMETER - already in correct form

    let transaction = state
        .transaction_service
        .update(&id_ulid, entry_id, amount, date, title)
        .await?;

    Ok(Json(transaction.into()))
}
```

#### 4.4 Update Error Mapping

**File**: `backend/crates/api/src/error.rs`

Add mapping for new error variant:

```rust
impl From<TransactionError> for ApiError {
    fn from(err: TransactionError) -> Self {
        match err {
            // ... existing mappings ...
            TransactionError::TitleTooLong { length, max } => ApiError::ValidationError {
                code: "TITLE_TOO_LONG".to_string(),
                details: serde_json::json!({ "length": length, "max": max }),
            },
        }
    }
}
```

**Testing**:
- API integration test: POST with title, verify response
- API integration test: POST without title (null)
- API integration test: PATCH to set title
- API integration test: PATCH to clear title (set to null)
- API integration test: PATCH without title field (no change)
- API integration test: POST with title exceeding 50 chars, verify 400 error

---

### Phase 5: Frontend - Type Definitions

**Files to modify**:
1. `frontend/src/api/types.ts`
2. `frontend/src/config/constants.ts` (create new file)

#### 5.1 Update Transaction Types

**File**: `frontend/src/api/types.ts`

```typescript
export interface Transaction {
  id: string
  entry_id: string
  amount: number
  date: string
  title: string | null  // NEW FIELD
  created_at: string
  updated_at: string
}

export interface CreateTransactionRequest {
  entry_id: string
  amount: number
  date: string
  title?: string | null  // NEW FIELD - optional in request
}

export interface UpdateTransactionRequest {
  entry_id?: string
  amount?: number
  date?: string
  title?: string | null  // NEW FIELD - undefined = don't change, null = clear
}
```

#### 5.2 Create Constants File

**File**: `frontend/src/config/constants.ts`

```typescript
/**
 * Maximum length for transaction title.
 * Must match backend MAX_TITLE_LENGTH constant.
 */
export const MAX_TITLE_LENGTH = 50
```

**Testing**: Run `npm run type-check`, verify no errors

---

### Phase 6: Frontend - TransactionForm Component

**File**: `frontend/src/components/transactions/TransactionForm.vue`

Add title input field with proper handling:

```vue
<template>
  <v-form @submit.prevent="handleSave">
    <!-- Existing category select -->
    <v-select
      v-model="entryId"
      :items="entryOptions"
      :label="$t('transactions.category')"
      item-title="label"
      item-value="value"
      required
      class="mb-2"
    />

    <!-- NEW: Title input field -->
    <v-text-field
      v-model="title"
      :label="$t('transactions.title')"
      :maxlength="MAX_TITLE_LENGTH"
      :counter="MAX_TITLE_LENGTH"
      clearable
      class="mb-2"
      :hint="$t('transactions.titleHint')"
      persistent-hint
    />

    <!-- Existing amount field -->
    <v-text-field
      v-model="amount"
      :label="$t('transactions.amount')"
      type="number"
      step="0.01"
      required
      class="mb-2"
    />

    <!-- Existing date field -->
    <v-text-field
      v-model="date"
      :label="$t('transactions.date')"
      type="date"
      required
      class="mb-2"
    />

    <!-- Submit button -->
    <v-btn type="submit" color="primary" block :loading="saving">
      {{ transaction ? $t('common.save') : $t('common.add') }}
    </v-btn>
  </v-form>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Transaction, Entry } from '@/api/types'
import { transactionsApi } from '@/api/transactions'
import { MAX_TITLE_LENGTH } from '@/config/constants'

const props = defineProps<{
  transaction: Transaction | null
  entries: Entry[]
}>()

const emit = defineEmits<{
  saved: []
}>()

const entryId = ref('')
const amount = ref('')
const date = ref('')
const title = ref<string>('')  // NEW STATE
const saving = ref(false)
const error = ref('')

// Initialize form when transaction changes
watch(() => props.transaction, (newTransaction) => {
  initializeForm(newTransaction)
}, { immediate: true })

/**
 * Initializes form fields from transaction or with defaults.
 *
 * @param transaction - Transaction to edit, or null for new transaction
 */
function initializeForm(transaction: Transaction | null) {
  if (transaction) {
    populateFromTransaction(transaction)
  } else {
    setDefaultValues()
  }
}

/**
 * Populates form fields from existing transaction.
 */
function populateFromTransaction(transaction: Transaction) {
  entryId.value = transaction.entry_id
  amount.value = formatAmountForDisplay(transaction.amount)
  date.value = transaction.date
  title.value = transaction.title ?? ''  // Convert null to empty string
}

/**
 * Sets default values for new transaction.
 */
function setDefaultValues() {
  entryId.value = ''
  amount.value = ''
  date.value = getCurrentDate()
  title.value = ''
}

/**
 * Formats amount from minor units to major units for display.
 */
function formatAmountForDisplay(minorUnits: number): string {
  return (minorUnits / 100).toFixed(2)
}

/**
 * Gets current date in YYYY-MM-DD format.
 */
function getCurrentDate(): string {
  return new Date().toISOString().split('T')[0]
}

/**
 * Validates that required form fields are filled.
 */
function isFormValid(): boolean {
  return !!(entryId.value && amount.value && date.value)
}

/**
 * Builds transaction payload from form values.
 */
function buildTransactionPayload() {
  const trimmedTitle = title.value.trim()

  return {
    entry_id: entryId.value,
    amount: Math.round(parseFloat(amount.value) * 100),
    date: date.value,
    title: trimmedTitle || null,  // Convert empty string to null
  }
}

/**
 * Saves the transaction (create or update).
 */
async function saveTransaction(payload: CreateTransactionRequest) {
  if (props.transaction) {
    await updateExistingTransaction(payload)
  } else {
    await createNewTransaction(payload)
  }
}

/**
 * Creates a new transaction.
 */
async function createNewTransaction(payload: CreateTransactionRequest) {
  await transactionsApi.create(payload)
}

/**
 * Updates existing transaction with only changed fields.
 */
async function updateExistingTransaction(payload: CreateTransactionRequest) {
  const updatePayload: UpdateTransactionRequest = {}
  const original = props.transaction!

  if (payload.amount !== original.amount) {
    updatePayload.amount = payload.amount
  }

  if (payload.date !== original.date) {
    updatePayload.date = payload.date
  }

  if (payload.entry_id !== original.entry_id) {
    updatePayload.entry_id = payload.entry_id
  }

  // Title: distinguish between "don't change" and "clear"
  const newTitle = payload.title ?? null
  if (newTitle !== original.title) {
    updatePayload.title = newTitle
  }

  if (Object.keys(updatePayload).length > 0) {
    await transactionsApi.update(original.id, updatePayload)
  }
}

/**
 * Handles form submission.
 */
async function handleSave() {
  if (!isFormValid()) return

  clearError()
  startSaving()

  try {
    const payload = buildTransactionPayload()
    await saveTransaction(payload)
    emit('saved')
  } catch (e) {
    handleSaveError(e)
  } finally {
    stopSaving()
  }
}

function clearError() {
  error.value = ''
}

function startSaving() {
  saving.value = true
}

function stopSaving() {
  saving.value = false
}

function handleSaveError(e: unknown) {
  if (e instanceof ApiError) {
    error.value = $t(`errors.${e.code}`, e.details || {})
  } else {
    error.value = $t('errors.UNKNOWN')
  }
}
</script>
```

**i18n additions needed**:
- `transactions.title` - "Title"
- `transactions.titleHint` - "Optional description (max 50 characters)"

**Testing**: Manual test create/edit flows with title, verify persistence

---

### Phase 7: Frontend - PanelTransactionList Component

**File**: `frontend/src/components/budget/PanelTransactionList.vue`

Update to 3-column layout (Date | Title | Amount):

```vue
<template>
  <div v-if="state.initialLoaded" class="transaction-list">
    <!-- Empty state -->
    <div v-if="transactions.length === 0" class="text-center py-4 text-medium-emphasis">
      {{ $t('transactions.empty') }}
    </div>

    <!-- Transaction list -->
    <div
      v-else
      ref="containerRef"
      :class="['transaction-list-container', { 'scrollable': state.showMoreClicked }]"
      @scroll="handleScroll"
    >
      <div
        v-for="transaction in transactions"
        :key="transaction.id"
        class="transaction-row"
        @click="emit('edit-transaction', transaction)"
      >
        <!-- Three-column grid layout -->
        <div class="transaction-grid">
          <!-- Date column -->
          <span class="transaction-date text-medium-emphasis">
            {{ formatDate(transaction.date) }}
          </span>

          <!-- Title column -->
          <span
            :class="getTitleClass(transaction.title)"
          >
            {{ getDisplayTitle(transaction.title) }}
          </span>

          <!-- Amount column -->
          <span class="transaction-amount text-right">
            {{ formatCurrency(transaction.amount) }}
          </span>
        </div>
      </div>

      <!-- Loading indicator when scrolling -->
      <div v-if="state.loading" class="text-center py-2">
        <v-progress-circular indeterminate size="24" />
      </div>
    </div>

    <!-- Show More button -->
    <div v-if="!state.showMoreClicked && state.hasMore" class="text-center mt-2">
      <v-btn
        variant="text"
        color="success"
        @click="handleShowMore"
      >
        {{ $t('transactions.showMore') }}
      </v-btn>
    </div>

    <!-- Initial loading -->
    <div v-if="!state.initialLoaded" class="text-center py-4">
      <v-progress-circular indeterminate />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatCurrency } from '@/utils/currency'
import type { Transaction } from '@/api/types'

const { t } = useI18n()

const props = defineProps<{
  entryId: string
}>()

const emit = defineEmits<{
  'edit-transaction': [transaction: Transaction]
}>()

// Use existing composable for transaction loading
const { state, transactions, load, loadMore } = useCategoryTransactions(props.entryId)

// Load initial transactions
load()

/**
 * Formats date for display.
 */
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}

/**
 * Gets the display text for transaction title.
 *
 * @param title - The transaction title (may be null)
 * @returns Display text (title or "No title" message)
 */
function getDisplayTitle(title: string | null): string {
  return title ?? t('transactions.noTitle')
}

/**
 * Gets CSS class for title display.
 *
 * @param title - The transaction title (may be null)
 * @returns CSS class string
 */
function getTitleClass(title: string | null): string {
  return title ? 'transaction-title' : 'transaction-title transaction-title-empty'
}

/**
 * Handles "Show More" button click.
 */
function handleShowMore() {
  state.showMoreClicked = true
  loadMore()
}

/**
 * Handles scroll event for infinite scrolling.
 */
function handleScroll(event: Event) {
  const container = event.target as HTMLElement
  const scrolledToBottom =
    container.scrollHeight - container.scrollTop <= container.clientHeight + 50

  if (scrolledToBottom && state.hasMore && !state.loading) {
    loadMore()
  }
}
</script>

<style scoped>
.transaction-list-container {
  max-height: 280px;
}

.transaction-list-container.scrollable {
  overflow-y: auto;
}

.transaction-row {
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.transaction-row:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.transaction-grid {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 12px;
  align-items: center;
}

.transaction-date {
  font-size: 0.875rem;
  min-width: 80px;
}

.transaction-title {
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.transaction-title-empty {
  font-style: italic;
  opacity: 0.5;
}

.transaction-amount {
  font-size: 0.875rem;
  font-weight: 500;
  white-space: nowrap;
}
</style>
```

**i18n additions**:
- `transactions.noTitle` - "No title" (EN) / "Brak tytułu" (PL)

**Testing**: Manual test display with/without titles, verify layout

---

### Phase 8: Frontend - TransactionList Component

**File**: `frontend/src/components/transactions/TransactionList.vue`

Add title column to full month transaction table:

```vue
<template>
  <v-table v-if="transactions.length > 0" density="compact">
    <thead>
      <tr>
        <th>{{ $t('transactions.category') }}</th>
        <th>{{ $t('transactions.date') }}</th>
        <th>{{ $t('transactions.title') }}</th>  <!-- NEW COLUMN -->
        <th class="text-right">{{ $t('transactions.amount') }}</th>
        <th class="text-center">{{ $t('common.actions') }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="transaction in transactions" :key="transaction.id">
        <td>{{ getCategoryName(transaction) }}</td>
        <td>{{ formatDate(transaction.date) }}</td>
        <td :class="getTitleCellClass(transaction.title)">  <!-- NEW CELL -->
          {{ getDisplayTitle(transaction.title) }}
        </td>
        <td class="text-right">{{ formatCurrency(transaction.amount) }}</td>
        <td class="text-center">
          <v-btn
            icon="mdi-pencil"
            size="small"
            variant="text"
            @click="emit('edit', transaction)"
          />
          <v-btn
            icon="mdi-delete"
            size="small"
            variant="text"
            @click="emit('delete', transaction)"
          />
        </td>
      </tr>
    </tbody>
  </v-table>
  <div v-else class="text-center py-4 text-medium-emphasis">
    {{ $t('transactions.empty') }}
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { formatCurrency } from '@/utils/currency'
import type { Transaction } from '@/api/types'

const { t } = useI18n()

defineProps<{
  transactions: Transaction[]
}>()

const emit = defineEmits<{
  edit: [transaction: Transaction]
  delete: [transaction: Transaction]
}>()

/**
 * Formats date for display.
 */
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}

/**
 * Gets the display text for transaction title.
 *
 * @param title - The transaction title (may be null)
 * @returns Display text (title or "No title" message)
 */
function getDisplayTitle(title: string | null): string {
  return title ?? t('transactions.noTitle')
}

/**
 * Gets CSS class for title table cell.
 *
 * @param title - The transaction title (may be null)
 * @returns CSS class string
 */
function getTitleCellClass(title: string | null): string {
  return title ? '' : 'title-empty'
}

/**
 * Gets category name for transaction (requires fetching entry data).
 */
function getCategoryName(transaction: Transaction): string {
  // Implementation depends on how entries are passed to this component
  // May need to accept entries as prop or fetch via composable
  return '...'  // Placeholder
}
</script>

<style scoped>
.title-empty {
  font-style: italic;
  opacity: 0.5;
}
</style>
```

**Testing**: Manual test table display with mixed title/no-title transactions

---

### Phase 9: Frontend - i18n Translations

**Files to modify**:
1. `frontend/src/i18n/locales/en.json`
2. `frontend/src/i18n/locales/pl.json`

#### English translations

**File**: `frontend/src/i18n/locales/en.json`

Add to `transactions` section:

```json
{
  "transactions": {
    "title": "Title",
    "titleHint": "Optional description (max 50 characters)",
    "noTitle": "No title"
  },
  "errors": {
    "TITLE_TOO_LONG": "Title is too long ({length} characters, max {max})"
  }
}
```

#### Polish translations

**File**: `frontend/src/i18n/locales/pl.json`

```json
{
  "transactions": {
    "title": "Tytuł",
    "titleHint": "Opcjonalny opis (maksymalnie 50 znaków)",
    "noTitle": "Brak tytułu"
  },
  "errors": {
    "TITLE_TOO_LONG": "Tytuł jest zbyt długi ({length} znaków, maksymalnie {max})"
  }
}
```

---

## Testing Strategy

### Backend Tests

**Unit Tests** (`backend/crates/domain/src/services/transaction_service.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_empty_string() {
        assert_eq!(normalize_title(Some("".to_string())), None);
    }

    #[test]
    fn test_normalize_title_whitespace() {
        assert_eq!(normalize_title(Some("   ".to_string())), None);
    }

    #[test]
    fn test_normalize_title_trimmed() {
        assert_eq!(
            normalize_title(Some("  hello  ".to_string())),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_validate_title_length_valid() {
        let title = Some("Short title".to_string());
        assert!(validate_title_length(&title).is_ok());
    }

    #[test]
    fn test_validate_title_length_too_long() {
        let title = Some("a".repeat(51));
        assert!(matches!(
            validate_title_length(&title),
            Err(TransactionError::TitleTooLong { .. })
        ));
    }

    #[test]
    fn test_validate_title_length_at_limit() {
        let title = Some("a".repeat(50));
        assert!(validate_title_length(&title).is_ok());
    }
}
```

**Integration Tests** (`backend/crates/api/tests/transactions_test.rs`):
```rust
#[tokio::test]
async fn test_create_transaction_with_title() {
    // Arrange: Set up test DB, create entry
    // Act: POST /transactions with title
    // Assert: Response contains title, DB contains title
}

#[tokio::test]
async fn test_create_transaction_without_title() {
    // Arrange: Set up test DB, create entry
    // Act: POST /transactions with title: null
    // Assert: Response has null title, DB has NULL
}

#[tokio::test]
async fn test_update_transaction_set_title() {
    // Arrange: Create transaction without title
    // Act: PATCH with title
    // Assert: Title updated
}

#[tokio::test]
async fn test_update_transaction_clear_title() {
    // Arrange: Create transaction with title
    // Act: PATCH with title: null
    // Assert: Title cleared
}

#[tokio::test]
async fn test_create_transaction_title_too_long() {
    // Arrange: Build request with 51-char title
    // Act: POST /transactions
    // Assert: 400 Bad Request, error code TITLE_TOO_LONG
}
```

### Frontend Tests

**Type Checking**:
```bash
npm run type-check
```

**Manual Testing Checklist**:
- [ ] Create transaction with title
- [ ] Create transaction without title
- [ ] Edit transaction to add title
- [ ] Edit transaction to remove title (clear field)
- [ ] Edit transaction to change title
- [ ] Verify title display in PanelTransactionList (3-column layout)
- [ ] Verify title display in TransactionList (table)
- [ ] Verify empty state styling (italic, faded)
- [ ] Verify character counter appears
- [ ] Verify maxlength enforcement (50 chars)
- [ ] Test i18n: Switch to Polish, verify translations
- [ ] Test mobile layout

---

## Migration Strategy

1. **Run migration**: `sqlx migrate run` in backend directory
2. **Update sqlx-data.json**: `sqlx prepare` to update compile-time query metadata
3. **Verify schema**: Check transactions table has title column
4. **Backward compatibility**: Old transactions will have `NULL` title (displayed as "No title")

---

## Clean Code Compliance

### Single Responsibility Principle

**Backend examples**:
- `normalize_title()` - Only normalizes, doesn't validate
- `validate_title_length()` - Only validates length, doesn't normalize
- `initializeForm()` - Delegates to `populateFromTransaction()` or `setDefaultValues()`
- `handleSave()` - Orchestrates save flow, delegates to helpers

**Frontend examples**:
- `getDisplayTitle()` - Only determines display text
- `getTitleClass()` - Only determines CSS class
- `buildTransactionPayload()` - Only builds payload object
- `saveTransaction()` - Only saves, doesn't build or validate

### Documentation

All functions include:
- Purpose description
- `@param` / `# Arguments` for parameters
- `@returns` / `# Returns` for return values
- `@throws` / `# Errors` for error cases
- Examples where helpful

### Testing

- Unit tests for all validation/normalization functions
- Integration tests for API endpoints
- Manual E2E testing for UI flows

---

## Risk Assessment

### Low Risk
- Database migration (simple ALTER TABLE ADD COLUMN)
- Backward compatibility (NULL values handled gracefully)
- UI changes (additive, doesn't break existing functionality)

### Medium Risk
- Form validation logic (ensure empty strings convert to NULL correctly)
- Update semantics (distinguish between "don't change" and "clear")

### Mitigation
- Comprehensive testing at each layer
- Follow existing patterns (Category.label)
- Incremental implementation (backend first, then frontend)

---

## Estimated Effort

- **Phase 1-4 (Backend)**: 2 hours
- **Phase 5-8 (Frontend)**: 2 hours
- **Phase 9 (i18n)**: 15 minutes
- **Testing**: 30 minutes
- **Total**: ~4.5 hours

---

## Critical Files Summary

**Backend (5 files)**:
1. `backend/crates/db/migrations/00X_add_transaction_title.sql` - Schema change
2. `backend/crates/domain/src/entities/transaction.rs` - Domain entity + validation
3. `backend/crates/domain/src/services/transaction_service.rs` - Business logic
4. `backend/crates/db/src/repos/transaction_repo.rs` - Database layer
5. `backend/crates/api/src/handlers/transactions.rs` - HTTP layer

**Frontend (6 files)**:
1. `frontend/src/api/types.ts` - TypeScript types
2. `frontend/src/config/constants.ts` - Configuration constant (new file)
3. `frontend/src/components/transactions/TransactionForm.vue` - Input form
4. `frontend/src/components/budget/PanelTransactionList.vue` - Compact display
5. `frontend/src/components/transactions/TransactionList.vue` - Full table
6. `frontend/src/i18n/locales/*.json` - Translations

---

## Task Directory Structure

After implementation, this plan should be moved to:

```
tasks/
└── 2.0-transaction-title/
    ├── PLAN.md      (this file)
    └── STATUS.md    (implementation progress tracker)
```

The STATUS.md file should track:
- [ ] Phase 1: Database Migration
- [ ] Phase 2: Backend - Domain Layer
- [ ] Phase 3: Backend - Database Repository
- [ ] Phase 4: Backend - API Layer
- [ ] Phase 5: Frontend - Type Definitions
- [ ] Phase 6: Frontend - TransactionForm
- [ ] Phase 7: Frontend - PanelTransactionList
- [ ] Phase 8: Frontend - TransactionList
- [ ] Phase 9: Frontend - i18n Translations
- [ ] Testing completed
- [ ] Code review completed
- [ ] PR merged
