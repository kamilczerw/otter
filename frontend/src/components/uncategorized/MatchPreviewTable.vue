<!--
  Presentational preview of a draft rule's "other matches": collapsed, it's a
  single toggle row showing how many other transactions match; expanded, it's
  a scrollable table of those transactions that requests more pages as the
  user scrolls near the bottom. Owns no data-fetching — the parent view feeds
  it `transactions`/`otherCount`/`loading`/`hasMore` and reacts to its emits.
-->
<template>
  <div>
    <div
      data-testid="match-preview-toggle"
      class="d-flex align-center ga-1 toggle-row"
      role="button"
      @click="$emit('toggle')"
    >
      <v-icon size="small">{{ expanded ? 'mdi-chevron-down' : 'mdi-chevron-right' }}</v-icon>
      <span>
        {{ expanded ? $t('uncategorized.hideOtherMatches') : $t('uncategorized.showOtherMatches', { count: otherCount }) }}
      </span>
    </div>

    <template v-if="expanded">
      <v-progress-linear v-if="loading" indeterminate color="primary" />
      <div
        data-testid="match-preview-scroll"
        ref="scrollContainer"
        class="scroll-container"
        @scroll="onScroll"
      >
        <v-table density="compact">
          <thead>
            <tr>
              <th>{{ $t('transactions.date') }}</th>
              <th>{{ $t('transactions.title') }}</th>
              <th>{{ $t('transactions.amount') }}</th>
              <th>{{ $t('transactions.merchant') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="transaction in transactions"
              :key="transaction.id"
              data-testid="match-preview-row"
              class="match-row"
              @click="$emit('row-click', transaction)"
            >
              <td>{{ transaction.date }}</td>
              <td>{{ transaction.title || $t('transactions.noTitle') }}</td>
              <td>{{ formatCurrency(transaction.amount) }}</td>
              <td>{{ transaction.metadata?.merchant }}</td>
            </tr>
          </tbody>
        </v-table>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Transaction } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

const props = defineProps<{
  /** The current page of "other matches" to render, in queue order. */
  transactions: Transaction[]
  /** Whether the table is expanded (showing transactions) or collapsed (showing just the count). */
  expanded: boolean
  /** Count of other matching transactions, shown on the collapsed toggle. */
  otherCount: number
  /** Whether a page fetch is in flight — suppresses further load-more emits. */
  loading: boolean
  /** Whether more pages remain to be fetched. */
  hasMore: boolean
}>()

const emit = defineEmits<{
  toggle: []
  'load-more': []
  'row-click': [transaction: Transaction]
}>()

/** Template ref to the scrollable table container, read for scroll geometry. */
const scrollContainer = ref<HTMLElement | null>(null)

/** How close to the bottom (in pixels) triggers a load-more request. */
const LOAD_MORE_THRESHOLD_PX = 16

/**
 * Emits `load-more` once the scrollable container is scrolled near its
 * bottom, but only when there's actually another page to fetch and no fetch
 * is already in flight.
 */
function onScroll(): void {
  if (!props.hasMore || props.loading) return

  const el = scrollContainer.value
  if (!el) return

  const nearBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - LOAD_MORE_THRESHOLD_PX
  if (nearBottom) {
    emit('load-more')
  }
}
</script>

<style scoped>
.toggle-row {
  cursor: pointer;
}

.scroll-container {
  max-height: 240px;
  overflow-y: auto;
}

.match-row {
  cursor: pointer;
}
</style>
