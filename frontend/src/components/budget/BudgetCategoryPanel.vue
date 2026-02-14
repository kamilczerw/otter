<template>
  <div class="budget-panel">
    <PanelActionRow
      @edit-budget="$emit('edit-budget')"
      @add-transaction="$emit('add-transaction')"
    />
    <PanelTransactionList
      :entry-id="entryId"
      @edit-transaction="(tx) => $emit('edit-transaction', tx)"
    />
  </div>
</template>

<script setup lang="ts">
import PanelActionRow from './PanelActionRow.vue'
import PanelTransactionList from './PanelTransactionList.vue'
import type { Transaction } from '@/api/types'

defineProps<{
  entryId: string
}>()

defineEmits<{
  'edit-budget': []
  'add-transaction': []
  'edit-transaction': [tx: Transaction]
}>()
</script>

<style scoped>
.budget-panel {
  background-color: #151f2e;
  border-radius: 0 0 8px 8px;
  padding: 8px 12px 12px;
  margin-top: -2px;
}
</style>
