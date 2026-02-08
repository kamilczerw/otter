<template>
  <v-container>
    <MonthTabs />

    <v-progress-linear v-if="loading" indeterminate class="mt-2" />

    <v-row class="mt-4">
      <v-col cols="12">
        <TransactionList
          :transactions="transactions"
          :entries="entries"
          :loading="loading"
          @edit="editTransaction"
          @delete="deleteTransaction"
        />
      </v-col>
    </v-row>

    <!-- Add Transaction FAB -->
    <v-btn
      color="primary"
      icon="mdi-plus"
      size="large"
      position="fixed"
      location="bottom end"
      class="mb-16 mr-4"
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
import MonthTabs from '@/components/layout/MonthTabs.vue'
import TransactionList from '@/components/transactions/TransactionList.vue'
import TransactionDrawer from '@/components/transactions/TransactionDrawer.vue'
import { monthsApi } from '@/api/months'
import { entriesApi } from '@/api/entries'
import { transactionsApi } from '@/api/transactions'
import type { Transaction, Entry, Month } from '@/api/types'

const route = useRoute()

const monthId = ref('')
const transactions = ref<Transaction[]>([])
const entries = ref<Entry[]>([])
const loading = ref(false)
const drawerOpen = ref(false)
const selectedTransaction = ref<Transaction | null>(null)

async function resolveMonthId() {
  const monthStr = route.params.month as string
  const months = await monthsApi.list()
  const found = months.find((m: Month) => m.month === monthStr)
  if (found) {
    monthId.value = found.id
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
  await resolveMonthId()
  await loadData()
})

onMounted(async () => {
  await resolveMonthId()
  await loadData()
})
</script>
