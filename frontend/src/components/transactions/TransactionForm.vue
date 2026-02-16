<template>
  <div>
    <v-select
      v-model="entryId"
      :items="entriesWithDisplayName"
      :label="$t('entries.category')"
      item-title="displayName"
      item-value="id"
      class="mb-2"
    />
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
    <v-text-field
      ref="amountField"
      v-model="amount"
      :label="$t('transactions.amount')"
      type="number"
      min="0"
      class="mb-2"
    />
    <v-text-field
      v-model="date"
      :label="$t('transactions.date')"
      type="date"
    />
    <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mt-2">
      {{ error }}
    </v-alert>
    <div class="d-flex justify-end mt-4">
      <v-btn class="mr-2 btn-secondary-glass" @click="$emit('cancel')">{{ $t('common.cancel') }}</v-btn>
      <v-btn class="btn-primary-glass" @click="handleSave" :loading="saving">{{ $t('common.save') }}</v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { transactionsApi } from '@/api/transactions'
import type { Transaction, Entry, CreateTransactionRequest, UpdateTransactionRequest } from '@/api/types'
import { ApiError } from '@/api/types'
import { parseCurrencyToMinor } from '@/utils/currency'
import { getCategoryDisplayName } from '@/utils/category'
import { MAX_TITLE_LENGTH } from '@/constants'

const { t } = useI18n()

const props = defineProps<{
  entries: Entry[]
  transaction: Transaction | null
  preselectedEntryId?: string | null
}>()

const emit = defineEmits<{
  saved: []
  cancel: []
}>()

const amountField = ref<any>(null)
const entryId = ref('')
const title = ref('')
const amount = ref('')
const date = ref('')
const error = ref('')
const saving = ref(false)

// Map entries to include display name for the select
const entriesWithDisplayName = computed(() =>
  props.entries.map(entry => ({
    ...entry,
    displayName: getCategoryDisplayName(entry.category)
  }))
)

/**
 * Initializes the form when component is mounted.
 * Orchestrates loading existing transaction data or setting defaults.
 */
function initializeForm(): void {
  if (props.transaction) {
    populateFromTransaction(props.transaction)
  } else {
    setDefaultValues()
  }
}

/**
 * Populates form fields from an existing transaction.
 *
 * @param transaction - The transaction to load into the form
 */
function populateFromTransaction(transaction: Transaction): void {
  entryId.value = transaction.entry_id
  title.value = transaction.title ?? ''
  amount.value = (transaction.amount / 100).toFixed(2)
  date.value = transaction.date
}

/**
 * Sets default values for a new transaction form.
 * Applies preselected entry if provided and focuses amount field.
 */
function setDefaultValues(): void {
  if (props.preselectedEntryId) {
    entryId.value = props.preselectedEntryId
    // Focus amount field when opened from expanded bar with preselected entry
    nextTick(() => {
      amountField.value?.focus()
    })
  }
  const today = new Date()
  date.value = today.toISOString().split('T')[0]
  title.value = ''
}

/**
 * Builds the transaction payload for create operation.
 * Converts empty title to null.
 *
 * @returns Create transaction request payload
 */
function buildCreatePayload(): CreateTransactionRequest {
  const trimmedTitle = title.value.trim()
  return {
    entry_id: entryId.value,
    amount: parseCurrencyToMinor(amount.value),
    date: date.value,
    title: trimmedTitle || null
  }
}

/**
 * Builds the transaction payload for update operation.
 * Only includes fields that have changed.
 * Converts empty title to null.
 *
 * @param original - The original transaction being updated
 * @returns Update transaction request payload with only changed fields
 */
function buildUpdatePayload(original: Transaction): UpdateTransactionRequest {
  const payload: UpdateTransactionRequest = {}
  const trimmedTitle = title.value.trim()
  const normalizedTitle = trimmedTitle || null

  if (entryId.value !== original.entry_id) {
    payload.entry_id = entryId.value
  }

  const newAmount = parseCurrencyToMinor(amount.value)
  if (newAmount !== original.amount) {
    payload.amount = newAmount
  }

  if (date.value !== original.date) {
    payload.date = date.value
  }

  if (normalizedTitle !== original.title) {
    payload.title = normalizedTitle
  }

  return payload
}

/**
 * Validates that all required form fields are filled.
 *
 * @returns True if form is valid, false otherwise
 */
function isFormValid(): boolean {
  return !!(entryId.value && amount.value && date.value)
}

/**
 * Clears the error message.
 */
function clearError(): void {
  error.value = ''
}

/**
 * Sets the saving state to true.
 */
function startSaving(): void {
  saving.value = true
}

/**
 * Sets the saving state to false.
 */
function stopSaving(): void {
  saving.value = false
}

/**
 * Handles API errors by setting the error message.
 *
 * @param e - The error thrown by the API
 */
function handleSaveError(e: unknown): void {
  if (e instanceof ApiError) {
    error.value = t(`errors.${e.code}`, e.details || {})
  }
}

/**
 * Creates a new transaction via API.
 *
 * @throws {ApiError} If the API request fails
 */
async function createNewTransaction(): Promise<void> {
  const payload = buildCreatePayload()
  await transactionsApi.create(payload)
}

/**
 * Updates an existing transaction via API.
 * Only sends changed fields.
 *
 * @param original - The original transaction being updated
 * @throws {ApiError} If the API request fails
 */
async function updateExistingTransaction(original: Transaction): Promise<void> {
  const payload = buildUpdatePayload(original)
  await transactionsApi.update(original.id, payload)
}

/**
 * Orchestrates the save operation.
 * Delegates to create or update based on transaction state.
 *
 * @throws {ApiError} If the API request fails
 */
async function saveTransaction(): Promise<void> {
  if (props.transaction) {
    await updateExistingTransaction(props.transaction)
  } else {
    await createNewTransaction()
  }
}

/**
 * Handles the save button click.
 * Validates form, saves transaction, and emits saved event on success.
 */
async function handleSave(): Promise<void> {
  if (!isFormValid()) return

  clearError()
  startSaving()

  try {
    await saveTransaction()
    emit('saved')
  } catch (e) {
    handleSaveError(e)
  } finally {
    stopSaving()
  }
}

onMounted(() => {
  initializeForm()
})
</script>
