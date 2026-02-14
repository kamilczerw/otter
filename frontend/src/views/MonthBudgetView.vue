<template>
  <v-container>
    <!-- Month Navigation Bar -->
    <MonthNavigationBar
      v-if="monthId"
      :current-month="route.params.month as string"
      :current-month-id="monthId"
      :all-months="allMonths"
    />

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mt-2" />

    <v-alert v-if="error" type="error" class="mt-4">{{ error }}</v-alert>

    <template v-if="!loading && summary">
      <!-- Summary Stats -->
      <div class="stats-row mt-4">
        <div class="stat-block">
          <div class="stat-label">{{ $t('summary.totalBudgeted') }}</div>
          <div class="stat-value text-magenta">{{ formatCurrency(summary.total_budgeted) }}</div>
        </div>
        <div class="stat-block">
          <div class="stat-label">{{ $t('summary.remaining') }}</div>
          <div class="stat-value" :class="summary.remaining >= 0 ? 'text-success-cosmic' : 'text-danger-cosmic'">
            {{ formatCurrency(summary.remaining) }}
          </div>
        </div>
      </div>

      <!-- Budget Progress Bars -->
      <BudgetProgressBars
        :categories="summary.categories"
        :bar-size="budgetBarSize"
        v-model:expanded-entry-id="expandedEntryId"
        @edit-budget="onEditBudget"
        @add-transaction="onAddTransaction"
        @edit-transaction="onEditTransaction"
        class="mt-4"
      />

      <!-- Charts Section (collapsible) -->
      <div class="glass-card mt-4 pa-4">
        <button class="section-toggle" @click="chartsOpen = !chartsOpen">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="12" width="4" height="9" rx="1" />
            <rect x="10" y="7" width="4" height="14" rx="1" />
            <rect x="17" y="3" width="4" height="18" rx="1" />
          </svg>
          <span class="section-label">{{ $t('summary.charts') }}</span>
          <v-icon class="toggle-icon" :class="{ rotated: chartsOpen }">mdi-chevron-down</v-icon>
        </button>

        <div v-show="chartsOpen" class="charts-content">
          <div class="charts-grid">
            <div class="chart-cell chart-cell--bar">
              <div class="section-label mb-3">{{ $t('summary.budgetVsActual') }}</div>
              <BudgetVsActualChart :categories="summary.categories" />
            </div>
            <div class="chart-cell chart-cell--donut">
              <div class="section-label mb-3">{{ $t('summary.paymentProgress') }}</div>
              <PaymentProgressDonut
                :total-budgeted="summary.total_budgeted"
                :total-paid="summary.total_paid"
              />
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Categories Section -->
    <div class="glass-card mt-4 pa-4">
      <div class="section-header mb-3">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z" />
          <line x1="7" y1="7" x2="7.01" y2="7" />
        </svg>
        <span class="section-label">{{ $t('budget.categories') }}</span>
      </div>
      <EntryList
        :month-id="monthId"
        :entries="entries"
        :loading="loadingEntries"
        :category-summaries="summary?.categories"
        @refresh="loadData"
      />
    </div>

    <!-- Transactions Section -->
    <div class="glass-card mt-4 pa-4">
      <div class="section-header mb-3">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
        <span class="section-label">{{ $t('transactions.title') }}</span>
      </div>
      <TransactionList
        :transactions="transactions"
        :entries="entries"
        :loading="loadingTransactions"
        @add="openNewTransaction"
        @edit="editTransaction"
        @delete="deleteTransaction"
      />
    </div>

    <!-- Transaction Drawer -->
    <TransactionDrawer
      v-model="drawerOpen"
      :entries="entries"
      :transaction="selectedTransaction"
      :preselected-entry-id="preselectedEntryId"
      @saved="onTransactionSaved"
    />

    <EntryDrawer
      v-model="entryDrawerOpen"
      :month-id="monthId"
      :entry="selectedEntry"
      @saved="onEntrySaved"
      @deleted="onEntryDeleted"
    />
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import MonthNavigationBar from '@/components/layout/MonthNavigationBar.vue'
import BudgetVsActualChart from '@/components/charts/BudgetVsActualChart.vue'
import PaymentProgressDonut from '@/components/charts/PaymentProgressDonut.vue'
import BudgetProgressBars from '@/components/budget/BudgetProgressBars.vue'
import EntryList from '@/components/entries/EntryList.vue'
import TransactionList from '@/components/transactions/TransactionList.vue'
import TransactionDrawer from '@/components/transactions/TransactionDrawer.vue'
import EntryDrawer from '@/components/entries/EntryDrawer.vue'
import { entriesApi } from '@/api/entries'
import { summaryApi } from '@/api/summary'
import { transactionsApi } from '@/api/transactions'
import { useMonths } from '@/composables/useMonths'
import { useUiPreferences } from '@/composables/useUiPreferences'
import { useCategoryTransactions } from '@/composables/useCategoryTransactions'
import type { Entry, Month, MonthSummary, Transaction, CategoryBudgetSummary } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { resolveMonthId: resolveMonth, getMonths } = useMonths()
const { budgetBarSize } = useUiPreferences()
const { invalidate: invalidateTransactions, invalidateAll: invalidateAllTransactions } = useCategoryTransactions()

const monthId = ref('')
const entries = ref<Entry[]>([])
const transactions = ref<Transaction[]>([])
const summary = ref<MonthSummary | null>(null)
const loading = ref(false)
const loadingEntries = ref(false)
const loadingTransactions = ref(false)
const chartsOpen = ref(true)
const drawerOpen = ref(false)
const selectedTransaction = ref<Transaction | null>(null)
const expandedEntryId = ref<string | null>(null)
const entryDrawerOpen = ref(false)
const selectedEntry = ref<Entry | null>(null)
const preselectedEntryId = ref<string | null>(null)
const error = ref('')
const allMonths = ref<Month[]>([])

async function doResolveMonthId() {
  const monthStr = route.params.month as string
  error.value = ''
  try {
    monthId.value = await resolveMonth(monthStr)
    allMonths.value = await getMonths()
  } catch {
    monthId.value = ''
    error.value = t('errors.MONTH_NOT_FOUND')
  }
}

async function loadData() {
  if (!monthId.value) return
  loading.value = true
  loadingEntries.value = true
  loadingTransactions.value = true
  try {
    const [summaryData, entriesData, transactionsData] = await Promise.all([
      summaryApi.get(monthId.value),
      entriesApi.list(monthId.value),
      transactionsApi.list(monthId.value),
    ])
    summary.value = summaryData
    entries.value = entriesData
    transactions.value = transactionsData
  } catch (e) {
    console.error('Failed to load month data', e)
  } finally {
    loading.value = false
    loadingEntries.value = false
    loadingTransactions.value = false
  }
}

function onEditBudget(item: CategoryBudgetSummary) {
  const entry = entries.value.find(e => e.id === item.entry_id)
  if (entry) {
    selectedEntry.value = entry
    entryDrawerOpen.value = true
  }
}

function onAddTransaction(item: CategoryBudgetSummary) {
  selectedTransaction.value = null
  preselectedEntryId.value = item.entry_id
  drawerOpen.value = true
}

function onEditTransaction(tx: Transaction) {
  selectedTransaction.value = tx
  preselectedEntryId.value = null
  drawerOpen.value = true
}

function openNewTransaction() {
  selectedTransaction.value = null
  preselectedEntryId.value = null
  drawerOpen.value = true
}

function editTransaction(tx: Transaction) {
  selectedTransaction.value = tx
  drawerOpen.value = true
}

async function deleteTransaction(tx: Transaction) {
  try {
    await transactionsApi.delete(tx.id)
    await loadData()
    await invalidateTransactions(tx.entry_id)
  } catch (e) {
    console.error('Failed to delete transaction', e)
  }
}

async function onTransactionSaved() {
  drawerOpen.value = false
  const entryId = preselectedEntryId.value || selectedTransaction.value?.entry_id
  await loadData()
  if (entryId) {
    await invalidateTransactions(entryId)
  }
}

async function onEntrySaved() {
  entryDrawerOpen.value = false
  await loadData()
}

async function onEntryDeleted() {
  entryDrawerOpen.value = false
  expandedEntryId.value = null
  await loadData()
}

watch(() => route.params.month, async () => {
  expandedEntryId.value = null
  invalidateAllTransactions()
  await doResolveMonthId()
  await loadData()
})

onMounted(async () => {
  await doResolveMonthId()
  await loadData()
})
</script>

<style scoped>
/* Stats */
.stats-row {
  display: flex;
  gap: 10px;
}

.stats-row .stat-block {
  flex: 1;
}

/* Section Headers */
.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
}

.section-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  color: var(--text-secondary);
}

.section-toggle:hover {
  color: var(--text-primary);
}

.toggle-icon {
  margin-left: auto;
  transition: transform 0.2s ease;
  color: var(--text-secondary);
}

.toggle-icon.rotated {
  transform: rotate(180deg);
}

.charts-content {
  margin-top: 16px;
}

.charts-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
}

@media (min-width: 768px) {
  .charts-grid {
    grid-template-columns: 1fr 1fr;
  }
}

@media (min-width: 1024px) {
  .charts-grid {
    grid-template-columns: 3fr 2fr;
  }
}

.pa-4 {
  padding: 16px;
}

.mt-4 {
  margin-top: 16px;
}

.mt-3 {
  margin-top: 12px;
}

.mb-3 {
  margin-bottom: 12px;
}
</style>
