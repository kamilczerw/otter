<template>
  <v-container>
    <MonthTabs />

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
          <div class="stat-label">{{ $t('summary.totalPaid') }}</div>
          <div class="stat-value">{{ formatCurrency(summary.total_paid) }}</div>
        </div>
        <div class="stat-block">
          <div class="stat-label">{{ $t('summary.remaining') }}</div>
          <div class="stat-value" :class="summary.remaining >= 0 ? 'text-success-cosmic' : 'text-danger-cosmic'">
            {{ formatCurrency(summary.remaining) }}
          </div>
        </div>
      </div>

      <!-- Charts (collapsible) -->
      <div class="glass-card mt-4 pa-4">
        <button class="charts-toggle" @click="chartsOpen = !chartsOpen">
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

    <!-- Entries -->
    <div class="glass-card mt-4 pa-4">
      <EntryList
        :month-id="monthId"
        :entries="entries"
        :loading="loadingEntries"
        @refresh="loadData"
      />
    </div>

  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import MonthTabs from '@/components/layout/MonthTabs.vue'
import BudgetVsActualChart from '@/components/charts/BudgetVsActualChart.vue'
import PaymentProgressDonut from '@/components/charts/PaymentProgressDonut.vue'
import EntryList from '@/components/entries/EntryList.vue'
import { entriesApi } from '@/api/entries'
import { summaryApi } from '@/api/summary'
import { useMonths } from '@/composables/useMonths'
import type { Entry, MonthSummary } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

const route = useRoute()
const { t } = useI18n()
const { resolveMonthId: resolveMonth } = useMonths()

const monthId = ref('')
const entries = ref<Entry[]>([])
const summary = ref<MonthSummary | null>(null)
const loading = ref(false)
const loadingEntries = ref(false)
const chartsOpen = ref(true)
const error = ref('')

async function doResolveMonthId() {
  const monthStr = route.params.month as string
  error.value = ''
  try {
    monthId.value = await resolveMonth(monthStr)
  } catch {
    monthId.value = ''
    error.value = t('errors.MONTH_NOT_FOUND')
  }
}

async function loadData() {
  if (!monthId.value) return
  loading.value = true
  loadingEntries.value = true
  try {
    const [summaryData, entriesData] = await Promise.all([
      summaryApi.get(monthId.value),
      entriesApi.list(monthId.value),
    ])
    summary.value = summaryData
    entries.value = entriesData
  } catch (e) {
    console.error('Failed to load month data', e)
  } finally {
    loading.value = false
    loadingEntries.value = false
  }
}

watch(() => route.params.month, async () => {
  await doResolveMonthId()
  await loadData()
})

onMounted(async () => {
  await doResolveMonthId()
  await loadData()
})
</script>

<style scoped>
.stats-row {
  display: flex;
  gap: 10px;
}

.stats-row .stat-block {
  flex: 1;
}

.charts-toggle {
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

.charts-toggle:hover {
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
