<template>
  <div>
    <v-table v-if="entries.length > 0" density="compact">
      <thead>
        <tr>
          <th>{{ $t('entries.category') }}</th>
          <th class="text-right">{{ $t('entries.paidBudgeted') }}</th>
          <th class="text-center">{{ $t('entries.dueDay') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="entry in entries"
          :key="entry.id"
          class="cursor-pointer"
          :class="{ 'overspent-row': isOverspent(entry) }"
          @click="openEditDrawer(entry)"
        >
          <td>{{ getCategoryDisplayName(entry.category) }}</td>
          <td class="text-right">
            <span :class="{ 'overspent-text': isOverspent(entry) }">
              {{ formatCurrency(getPaid(entry)) }}/{{ formatCurrency(entry.budgeted) }}
            </span>
          </td>
          <td class="text-center">{{ entry.due_day ?? '-' }}</td>
        </tr>
      </tbody>
    </v-table>

    <v-alert v-if="entries.length === 0 && !loading" type="info" variant="tonal">
      {{ $t('entries.noEntries') }}
    </v-alert>

    <div class="mt-3">
      <v-btn
        variant="text"
        color="primary"
        size="small"
        prepend-icon="mdi-plus"
        @click="openAddDrawer"
      >
        {{ $t('entries.addEntry') }}
      </v-btn>
    </div>

    <!-- Entry Drawer -->
    <EntryDrawer
      v-model="drawerOpen"
      :month-id="monthId"
      :entry="selectedEntry"
      @saved="handleSaved"
      @deleted="handleDeleted"
    />

    <!-- Error Snackbar -->
    <v-snackbar v-model="showError" color="error" timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Entry, CategoryBudgetSummary } from '@/api/types'
import { formatCurrency } from '@/utils/currency'
import { getCategoryDisplayName } from '@/utils/category'
import EntryDrawer from './EntryDrawer.vue'

const props = defineProps<{
  entries: Entry[]
  monthId: string
  loading: boolean
  categorySummaries?: CategoryBudgetSummary[]
}>()

// Create a map of category ID to summary for quick lookup
const summaryMap = computed(() => {
  const map = new Map<string, CategoryBudgetSummary>()
  if (props.categorySummaries) {
    props.categorySummaries.forEach(summary => {
      map.set(summary.category.id, summary)
    })
  }
  return map
})

// Get paid amount for an entry
function getPaid(entry: Entry): number {
  const summary = summaryMap.value.get(entry.category.id)
  return summary?.paid ?? 0
}

// Check if entry is overspent
function isOverspent(entry: Entry): boolean {
  const summary = summaryMap.value.get(entry.category.id)
  return summary ? summary.remaining < 0 : false
}

const emit = defineEmits<{
  refresh: []
}>()

const drawerOpen = ref(false)
const selectedEntry = ref<Entry | null>(null)
const showError = ref(false)
const errorMessage = ref('')

function openAddDrawer() {
  selectedEntry.value = null
  drawerOpen.value = true
}

function openEditDrawer(entry: Entry) {
  selectedEntry.value = entry
  drawerOpen.value = true
}

function handleSaved() {
  emit('refresh')
}

function handleDeleted() {
  emit('refresh')
}
</script>

<style scoped>
.cursor-pointer {
  cursor: pointer;
}

.cursor-pointer:hover {
  background: rgba(255, 255, 255, 0.05);
}

.overspent-row {
  background: rgba(244, 67, 54, 0.15) !important;
  border-left: 3px solid rgb(244, 67, 54);
}

.overspent-row:hover {
  background: rgba(244, 67, 54, 0.25) !important;
}

.overspent-text {
  color: rgb(244, 67, 54);
  font-weight: 600;
}

.mt-3 {
  margin-top: 12px;
}
</style>
