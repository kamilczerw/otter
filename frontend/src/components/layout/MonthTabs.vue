<template>
  <v-tabs v-model="activeTab" grow>
    <v-tab :value="'budget'" @click="navigateTo('budget')">
      <v-icon start>mdi-calculator</v-icon>
      {{ $t('months.budget') }}
    </v-tab>
    <v-tab :value="'transactions'" @click="navigateTo('transactions')">
      <v-icon start>mdi-cash-multiple</v-icon>
      {{ $t('months.transactions') }}
    </v-tab>
  </v-tabs>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const activeTab = computed(() => {
  if (route.name === 'month-transactions') return 'transactions'
  return 'budget'
})

const monthParam = computed(() => route.params.month as string)

function navigateTo(tab: string) {
  router.push(`/months/${monthParam.value}/${tab}`)
}
</script>
