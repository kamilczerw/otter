<!--
  Presentational card rendering a single transaction's own fields: title,
  amount, date, and (when present) its import metadata — merchant, city,
  country, raw type, card last 4 digits, currency, and a click-to-expand raw
  description row. Falls back to an info alert when metadata is absent.
  Self-contained: owns its own expand/collapse state for the raw description.
-->
<template>
  <v-card variant="tonal" class="pa-3">
    <div>{{ $t('transactions.title') }}: {{ transaction.title || $t('transactions.noTitle') }}</div>
    <div>{{ $t('transactions.amount') }}: {{ formatCurrency(transaction.amount) }}</div>
    <div>{{ $t('transactions.date') }}: {{ transaction.date }}</div>
    <template v-if="transaction.metadata">
      <div v-if="transaction.metadata.merchant">
        {{ $t('transactions.merchant') }}: {{ transaction.metadata.merchant }}
      </div>
      <div v-if="transaction.metadata.city">{{ $t('transactions.city') }}: {{ transaction.metadata.city }}</div>
      <div v-if="transaction.metadata.country">
        {{ $t('transactions.country') }}: {{ transaction.metadata.country }}
      </div>
      <div v-if="transaction.metadata.raw_type">
        {{ $t('transactions.rawType') }}: {{ transaction.metadata.raw_type }}
      </div>
      <div v-if="transaction.metadata.card_last4">
        {{ $t('transactions.cardLast4') }}: {{ transaction.metadata.card_last4 }}
      </div>
      <div v-if="transaction.metadata.currency">
        {{ $t('transactions.currency') }}: {{ transaction.metadata.currency }}
      </div>
      <div
        v-if="transaction.metadata.raw_description"
        :class="showFullDescription ? 'description-full' : 'description-truncated'"
        @click="showFullDescription = !showFullDescription"
      >
        {{ $t('transactions.rawDescription') }}: {{ transaction.metadata.raw_description }}
      </div>
    </template>
    <v-alert v-else type="info" variant="tonal" density="compact" class="mt-2">
      {{ $t('uncategorized.noMetadata') }}
    </v-alert>
  </v-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Transaction } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

const props = defineProps<{
  /** The transaction whose details (and import metadata, if any) to render. */
  transaction: Transaction
}>()

/** Whether the raw-description row is expanded to its full (un-truncated) text. */
const showFullDescription = ref(false)

// Collapse back to truncated whenever a different transaction is shown.
watch(
  () => props.transaction,
  () => {
    showFullDescription.value = false
  },
)
</script>

<style scoped>
.description-truncated {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.description-full {
  white-space: normal;
  cursor: pointer;
}
</style>
