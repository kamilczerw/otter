<template>
  <div>
    <v-table v-if="transactions.length > 0" density="compact">
      <thead>
        <tr>
          <th>{{ $t('entries.category') }}</th>
          <th class="text-right">{{ $t('transactions.amount') }}</th>
          <th>{{ $t('transactions.date') }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="tx in transactions" :key="tx.id">
          <td>{{ getCategoryName(tx.entry_id) }}</td>
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
  </div>
</template>

<script setup lang="ts">
import type { Transaction, Entry } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

const props = defineProps<{
  transactions: Transaction[]
  entries: Entry[]
  loading: boolean
}>()

defineEmits<{
  edit: [tx: Transaction]
  delete: [tx: Transaction]
}>()

function getCategoryName(entryId: string): string {
  const entry = props.entries.find(e => e.id === entryId)
  return entry?.category.name ?? '\u2014'
}
</script>
