<template>
  <div class="month-nav-bar">
    <v-btn
      :icon="leftIcon"
      variant="text"
      size="large"
      :loading="creatingPrev"
      @click="handleLeftClick"
    />
    <div class="month-display">{{ monthDisplay }}</div>
    <v-btn
      :icon="rightIcon"
      variant="text"
      size="large"
      :loading="creatingNext"
      @click="handleRightClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { monthsApi } from '@/api/months'
import { useMonths } from '@/composables/useMonths'
import type { Month } from '@/api/types'

interface Props {
  currentMonth: string // YYYY-MM
  currentMonthId: string // ULID
  allMonths: Month[]
}

const props = defineProps<Props>()
const router = useRouter()
const { locale } = useI18n()
const { refreshMonths } = useMonths()

const creatingPrev = ref(false)
const creatingNext = ref(false)

// Display formatted month name
const monthDisplay = computed(() => {
  const [year, month] = props.currentMonth.split('-')
  const date = new Date(parseInt(year), parseInt(month) - 1)
  return date.toLocaleDateString(locale.value, { month: 'long', year: 'numeric' })
})

// Calculate previous month string (YYYY-MM)
function getPreviousMonthString(monthStr: string): string {
  const [year, month] = monthStr.split('-').map(Number)
  if (month === 1) {
    return `${year - 1}-12`
  }
  return `${year}-${String(month - 1).padStart(2, '0')}`
}

// Calculate next month string (YYYY-MM)
function getNextMonthString(monthStr: string): string {
  const [year, month] = monthStr.split('-').map(Number)
  if (month === 12) {
    return `${year + 1}-01`
  }
  return `${year}-${String(month + 1).padStart(2, '0')}`
}

// Check if a month exists in the list
function monthExists(monthStr: string): boolean {
  return props.allMonths.some(m => m.month === monthStr)
}

// Previous month logic
const prevMonthStr = computed(() => getPreviousMonthString(props.currentMonth))
const prevMonthExists = computed(() => monthExists(prevMonthStr.value))
const leftIcon = computed(() => prevMonthExists.value ? 'mdi-chevron-left' : 'mdi-plus')

// Next month logic
const nextMonthStr = computed(() => getNextMonthString(props.currentMonth))
const nextMonthExists = computed(() => monthExists(nextMonthStr.value))
const rightIcon = computed(() => nextMonthExists.value ? 'mdi-chevron-right' : 'mdi-plus')

// Handle left button click
async function handleLeftClick() {
  if (prevMonthExists.value) {
    // Navigate to existing previous month
    router.push(`/months/${prevMonthStr.value}`)
  } else {
    // Create empty month
    creatingPrev.value = true
    try {
      await monthsApi.create({
        month: prevMonthStr.value,
        empty: true,
      })
      await refreshMonths()
      router.push(`/months/${prevMonthStr.value}`)
    } catch (error) {
      console.error('Failed to create previous month:', error)
    } finally {
      creatingPrev.value = false
    }
  }
}

// Handle right button click
async function handleRightClick() {
  if (nextMonthExists.value) {
    // Navigate to existing next month
    router.push(`/months/${nextMonthStr.value}`)
  } else {
    // Create new month copying from current month
    creatingNext.value = true
    try {
      await monthsApi.create({
        month: nextMonthStr.value,
        copy_from: props.currentMonthId,
      })
      await refreshMonths()
      router.push(`/months/${nextMonthStr.value}`)
    } catch (error) {
      console.error('Failed to create next month:', error)
    } finally {
      creatingNext.value = false
    }
  }
}
</script>

<style scoped>
.month-nav-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  margin-bottom: 8px;
}

.month-display {
  flex: 1;
  text-align: center;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  text-transform: capitalize;
}
</style>
