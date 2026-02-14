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
      <v-btn class="btn-primary-glass" @click="save" :loading="saving">{{ $t('common.save') }}</v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { transactionsApi } from '@/api/transactions'
import type { Transaction, Entry } from '@/api/types'
import { ApiError } from '@/api/types'
import { parseCurrencyToMinor } from '@/utils/currency'
import { getCategoryDisplayName } from '@/utils/category'

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

const entryId = ref('')
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

onMounted(() => {
  if (props.transaction) {
    entryId.value = props.transaction.entry_id
    amount.value = (props.transaction.amount / 100).toFixed(2)
    date.value = props.transaction.date
  } else {
    if (props.preselectedEntryId) {
      entryId.value = props.preselectedEntryId
    }
    const today = new Date()
    date.value = today.toISOString().split('T')[0]
  }
})

async function save() {
  if (!entryId.value || !amount.value || !date.value) return
  error.value = ''
  saving.value = true
  try {
    if (props.transaction) {
      await transactionsApi.update(props.transaction.id, {
        entry_id: entryId.value,
        amount: parseCurrencyToMinor(amount.value),
        date: date.value,
      })
    } else {
      await transactionsApi.create({
        entry_id: entryId.value,
        amount: parseCurrencyToMinor(amount.value),
        date: date.value,
      })
    }
    emit('saved')
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  } finally {
    saving.value = false
  }
}
</script>
