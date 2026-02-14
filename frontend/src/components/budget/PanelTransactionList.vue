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
        <span class="transaction-date">{{ tx.date }}</span>
        <span class="transaction-amount">{{ formatCurrency(tx.amount) }}</span>
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
import { useCategoryTransactions } from '@/composables/useCategoryTransactions'
import { formatCurrency } from '@/utils/currency'
import { TRANSACTION_LIST_MAX_HEIGHT } from '@/constants'
import type { Transaction } from '@/api/types'

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

onMounted(() => {
  load(props.entryId)
})

watch(() => props.entryId, (newId) => {
  load(newId)
})

function handleShowMore() {
  loadMore(props.entryId)
}

function onScroll() {
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
}

.empty-state {
  text-align: center;
  padding: 16px 0;
  color: var(--text-secondary, #888);
  font-size: 13px;
}

.transaction-items--scrollable {
  overflow-y: auto;
  scroll-behavior: smooth;
}

.transaction-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.transaction-date {
  color: var(--text-secondary, #888);
  font-size: 13px;
}

.transaction-amount {
  color: #E8EAF0;
  font-size: 14px;
  font-weight: 500;
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
