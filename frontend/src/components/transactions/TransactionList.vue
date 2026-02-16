<template>
  <div>
    <v-table v-if="transactions.length > 0" density="compact">
      <thead>
        <tr>
          <th>{{ $t('entries.category') }}</th>
          <th>{{ $t('transactions.title') }}</th>
          <th class="text-right">{{ $t('transactions.amount') }}</th>
          <th>{{ $t('transactions.date') }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="tx in transactions" :key="tx.id">
          <td>{{ getCategoryName(tx.entry_id) }}</td>
          <td :class="getTitleCellClass(tx.title)">
            {{ getDisplayTitle(tx.title) }}
          </td>
          <td class="text-right">{{ formatCurrency(tx.amount) }}</td>
          <td>{{ tx.date }}</td>
          <td class="text-right">
            <v-btn icon size="x-small" variant="text" @click="$emit('edit', tx)">
              <v-icon size="small">mdi-pencil</v-icon>
            </v-btn>
            <v-btn icon size="x-small" variant="text" color="error" @click="$emit('delete', tx)">
              <v-icon size="small">mdi-delete</v-icon>
            </v-btn>
          </td>
        </tr>
      </tbody>
    </v-table>
    <v-alert v-else-if="!loading" type="info" variant="tonal">
      {{ $t('transactions.noTransactions') }}
    </v-alert>

    <div class="mt-3">
      <v-btn
        variant="text"
        color="primary"
        size="small"
        prepend-icon="mdi-plus"
        @click="$emit('add')"
      >
        {{ $t('transactions.addTransaction') }}
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { Transaction, Entry } from '@/api/types'
import { formatCurrency } from '@/utils/currency'
import { getCategoryDisplayName } from '@/utils/category'

const { t } = useI18n()

const props = defineProps<{
  transactions: Transaction[]
  entries: Entry[]
  loading: boolean
}>()

defineEmits<{
  add: []
  edit: [tx: Transaction]
  delete: [tx: Transaction]
}>()

/**
 * Gets the category display name for a transaction.
 *
 * @param entryId - The entry ID to look up
 * @returns Category display name or em dash if not found
 */
function getCategoryName(entryId: string): string {
  const entry = props.entries.find(e => e.id === entryId)
  return entry ? getCategoryDisplayName(entry.category) : '\u2014'
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
 * Gets CSS class for title cell display.
 *
 * @param title - The transaction title (may be null)
 * @returns CSS class string
 */
function getTitleCellClass(title: string | null): string {
  return title ? '' : 'title-empty'
}
</script>

<style scoped>
.mt-3 {
  margin-top: 12px;
}

.title-empty {
  font-style: italic;
  opacity: 0.5;
}
</style>
