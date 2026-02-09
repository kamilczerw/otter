<template>
  <v-container>
    <MonthTabs />

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mt-2" />

    <v-alert v-if="error" type="error" class="mt-4">{{ error }}</v-alert>

    <div class="glass-card mt-4 pa-4">
      <TransactionList
        :transactions="transactions"
        :entries="entries"
        :loading="loading"
        @edit="editTransaction"
        @delete="deleteTransaction"
      />
    </div>

    <!-- Add Transaction FAB -->
    <v-btn
      icon="mdi-plus"
      size="large"
      position="fixed"
      location="bottom end"
      class="mb-16 mr-4 btn-primary-glass"
      @click="openNewTransaction"
    />

    <!-- Transaction Drawer -->
    <TransactionDrawer
      v-model="drawerOpen"
      :entries="entries"
      :transaction="selectedTransaction"
      @saved="onTransactionSaved"
    />
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import MonthTabs from '@/components/layout/MonthTabs.vue'
import TransactionList from '@/components/transactions/TransactionList.vue'
import TransactionDrawer from '@/components/transactions/TransactionDrawer.vue'
import { entriesApi } from '@/api/entries'
import { transactionsApi } from '@/api/transactions'
import { useMonths } from '@/composables/useMonths'
import type { Transaction, Entry } from '@/api/types'

const route = useRoute()
const { t } = useI18n()
const { resolveMonthId: resolveMonth } = useMonths()

const monthId = ref('')
const transactions = ref<Transaction[]>([])
const entries = ref<Entry[]>([])
const loading = ref(false)
const drawerOpen = ref(false)
const selectedTransaction = ref<Transaction | null>(null)
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
  try {
    const [txData, entryData] = await Promise.all([
      transactionsApi.list(monthId.value),
      entriesApi.list(monthId.value),
    ])
    transactions.value = txData
    entries.value = entryData
  } catch (e) {
    console.error('Failed to load transactions', e)
  } finally {
    loading.value = false
  }
}

function openNewTransaction() {
  selectedTransaction.value = null
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
  } catch (e) {
    console.error('Failed to delete transaction', e)
  }
}

async function onTransactionSaved() {
  drawerOpen.value = false
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
.pa-4 {
  padding: 16px;
}

.mt-4 {
  margin-top: 16px;
}
</style>
