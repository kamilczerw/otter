<template>
  <v-container>
    <MonthTabs />

    <v-progress-linear v-if="loading" indeterminate class="mt-2" />

    <v-alert v-if="error" type="error" class="mt-4">{{ error }}</v-alert>

    <template v-if="!loading && summary">
      <!-- Summary Cards -->
      <v-row class="mt-4">
        <v-col cols="4">
          <v-card variant="tonal" color="primary">
            <v-card-text class="text-center">
              <div class="text-caption">{{ $t('summary.totalBudgeted') }}</div>
              <div class="text-h6">{{ formatCurrency(summary.total_budgeted) }}</div>
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="4">
          <v-card variant="tonal" color="secondary">
            <v-card-text class="text-center">
              <div class="text-caption">{{ $t('summary.totalPaid') }}</div>
              <div class="text-h6">{{ formatCurrency(summary.total_paid) }}</div>
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="4">
          <v-card variant="tonal" :color="summary.remaining >= 0 ? 'success' : 'error'">
            <v-card-text class="text-center">
              <div class="text-caption">{{ $t('summary.remaining') }}</div>
              <div class="text-h6">{{ formatCurrency(summary.remaining) }}</div>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>

      <!-- Charts -->
      <v-row class="mt-4">
        <v-col cols="12" md="8">
          <v-card>
            <v-card-title>{{ $t('summary.budgetVsActual') }}</v-card-title>
            <v-card-text>
              <BudgetVsActualChart :categories="summary.categories" />
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="12" md="4">
          <v-card>
            <v-card-title>{{ $t('summary.paymentProgress') }}</v-card-title>
            <v-card-text>
              <PaymentProgressDonut
                :total-budgeted="summary.total_budgeted"
                :total-paid="summary.total_paid"
              />
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </template>

    <!-- Entries -->
    <v-row class="mt-4">
      <v-col cols="12">
        <EntryList
          :month-id="monthId"
          :entries="entries"
          :loading="loadingEntries"
          @refresh="loadData"
        />
      </v-col>
    </v-row>

    <!-- Add Entry FAB -->
    <v-btn
      color="primary"
      icon="mdi-plus"
      size="large"
      position="fixed"
      location="bottom end"
      class="mb-16 mr-4"
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
