<template>
  <v-bottom-sheet v-model="isOpen">
    <v-card>
      <v-card-title>
        {{ transaction ? $t('common.edit') : $t('transactions.addTransaction') }}
      </v-card-title>
      <v-card-text>
        <TransactionForm
          :entries="entries"
          :transaction="transaction"
          @saved="$emit('saved')"
          @cancel="isOpen = false"
        />
      </v-card-text>
    </v-card>
  </v-bottom-sheet>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TransactionForm from './TransactionForm.vue'
import type { Transaction, Entry } from '@/api/types'

const props = defineProps<{
  modelValue: boolean
  entries: Entry[]
  transaction: Transaction | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
})
</script>
