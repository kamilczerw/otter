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

      <!-- Charts -->
      <div class="glass-card mt-4 pa-4">
        <div class="section-label mb-3">{{ $t('summary.budgetVsActual') }}</div>
        <BudgetVsActualChart :categories="summary.categories" />
      </div>

      <div class="glass-card mt-3 pa-4">
        <div class="section-label mb-3">{{ $t('summary.paymentProgress') }}</div>
        <PaymentProgressDonut
          :total-budgeted="summary.total_budgeted"
          :total-paid="summary.total_paid"
        />
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

    <!-- Add Entry FAB -->
    <v-btn
      icon="mdi-plus"
      size="large"
      position="fixed"
      location="bottom end"
      class="mb-16 mr-4 btn-primary-glass"
      @click="showEntryForm = true"
    />

    <!-- Add Entry Dialog -->
    <v-dialog v-model="showEntryForm" max-width="500">
      <EntryForm
        v-if="showEntryForm"
        :month-id="monthId"
        @saved="onEntrySaved"
        @cancel="showEntryForm = false"
      />
    </v-dialog>
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
import EntryForm from '@/components/entries/EntryForm.vue'
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
const showEntryForm = ref(false)
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

async function onEntrySaved() {
  showEntryForm.value = false
  await loadData()
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
