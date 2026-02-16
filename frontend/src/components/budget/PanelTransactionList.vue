<template>
  <div class="panel-transaction-list">
    <!-- Loading state -->
    <div v-if="isLoading && transactions.length === 0" class="text-center py-3">
      <v-progress-circular indeterminate size="24" width="2" color="grey" />
    </div>

    <!-- Empty state -->
    <div v-else-if="!isLoading && transactions.length === 0" class="empty-state">
      {{ $t('budget.panel.noTransactions') }}
    </div>

    <!-- Transaction list -->
    <div
      v-else
      ref="scrollContainer"
      class="transaction-items"
      :class="{ 'transaction-items--scrollable': showMoreClicked }"
      :style="showMoreClicked ? { maxHeight: TRANSACTION_LIST_MAX_HEIGHT + 'px' } : {}"
      @scroll="onScroll"
    >
      <div
        v-for="tx in transactions"
        :key="tx.id"
        class="transaction-row"
        @click="$emit('edit-transaction', tx)"
      >
        <div class="transaction-grid">
          <!-- Date column -->
          <span class="transaction-date text-medium-emphasis">
            {{ formatDate(tx.date) }}
          </span>

          <!-- Title column -->
          <span :class="getTitleClass(tx.title)">
            {{ getDisplayTitle(tx.title) }}
          </span>

          <!-- Amount column -->
          <span class="transaction-amount text-right">
            {{ formatCurrency(tx.amount) }}
          </span>
        </div>
      </div>

      <!-- Inline loading indicator during loadMore -->
      <div v-if="isLoading && transactions.length > 0" class="text-center py-2">
        <v-progress-circular indeterminate size="20" width="2" color="grey" />
      </div>
    </div>

    <!-- Show more link (before scrollable mode kicks in) -->
    <button
      v-if="hasMore && !showMoreClicked"
      class="show-more-btn"
      @click="handleShowMore"
    >
      {{ $t('budget.panel.showMore', { count: '+' }) }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCategoryTransactions } from '@/composables/useCategoryTransactions'
import { formatCurrency } from '@/utils/currency'
import { TRANSACTION_LIST_MAX_HEIGHT } from '@/constants'
import type { Transaction } from '@/api/types'

const { t } = useI18n()

const props = defineProps<{
  entryId: string
}>()

defineEmits<{
  'edit-transaction': [tx: Transaction]
}>()

const { load, loadMore, getTransactions, getHasMore, getIsLoading, getShowMoreClicked } = useCategoryTransactions()

const scrollContainer = ref<HTMLElement | null>(null)

const transactions = computed(() => getTransactions(props.entryId))
const hasMore = computed(() => getHasMore(props.entryId))
const isLoading = computed(() => getIsLoading(props.entryId))
const showMoreClicked = computed(() => getShowMoreClicked(props.entryId))

/**
 * Formats a date string for display.
 *
 * @param date - ISO date string (YYYY-MM-DD)
 * @returns Formatted date string
 */
function formatDate(date: string): string {
  return date
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

onMounted(() => {
  load(props.entryId)
})

watch(() => props.entryId, (newId) => {
  load(newId)
})

/**
 * Handles the "Show more" button click.
 */
function handleShowMore(): void {
  loadMore(props.entryId)
}

/**
 * Handles scroll events to trigger lazy loading.
 */
function onScroll(): void {
  if (!scrollContainer.value || !hasMore.value || isLoading.value) return
  const el = scrollContainer.value
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50
  if (nearBottom) {
    loadMore(props.entryId)
  }
}
</script>

<style scoped>
.panel-transaction-list {
  padding: 0 4px;
  min-height: 52px;
}

.empty-state {
  text-align: center;
  padding: 16px 0;
  color: var(--text-secondary, #888);
  font-size: 13px;
  min-height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.transaction-items--scrollable {
  overflow-y: auto;
  scroll-behavior: smooth;
}

.transaction-row {
  padding: 10px 8px;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.transaction-row:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.transaction-row + .transaction-row {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
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

.show-more-btn {
  display: block;
  width: 100%;
  padding: 8px;
  text-align: center;
  background: none;
  border: none;
  color: #3ddc84;
  font-size: 13px;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.show-more-btn:hover {
  background-color: rgba(61, 220, 132, 0.08);
}
</style>
