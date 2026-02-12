<template>
  <div class="budget-progress-bars">
    <div
      v-for="item in categories"
      :key="item.category.id"
      class="budget-bar"
      :class="barSizeClass"
      role="progressbar"
      :aria-valuenow="item.paid"
      :aria-valuemin="0"
      :aria-valuemax="item.budgeted"
      :aria-label="getAriaLabel(item)"
    >
      <div class="budget-bar__background">
        <div
          class="budget-bar__fill"
          :style="{ width: getFillWidth(item) + '%', backgroundColor: getBarColor(item) }"
        />
        <div
          v-if="item.paid > item.budgeted && item.budgeted > 0"
          class="budget-bar__overspend-line"
          :style="{ left: getOverspendLinePosition(item) + '%' }"
        />
        <div class="budget-bar__text">
          <span class="budget-bar__label">{{ getCategoryDisplayName(item.category) }}</span>
          <span class="budget-bar__amounts">{{ formatAmount(item.paid) }}/{{ formatAmount(item.budgeted) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CategoryBudgetSummary } from '@/api/types'
import { getCategoryDisplayName } from '@/utils/category'
import { formatCurrency } from '@/utils/currency'

interface Props {
  categories: CategoryBudgetSummary[]
  barSize?: 'compact' | 'spacious'
}

const props = withDefaults(defineProps<Props>(), {
  barSize: 'compact',
})

const barSizeClass = computed(() => `budget-bar--${props.barSize}`)

function formatAmount(minorUnits: number): string {
  return formatCurrency(minorUnits)
}

function getFillWidth(item: CategoryBudgetSummary): number {
  if (item.budgeted === 0) return item.paid > 0 ? 100 : 0
  return Math.min((item.paid / item.budgeted) * 100, 100)
}

function getBarColor(item: CategoryBudgetSummary): string {
  const { paid, budgeted } = item
  if (budgeted === 0 && paid > 0) return 'var(--budget-bar-color-red)'
  if (paid === 0) return 'var(--budget-bar-color-green)'
  const percentage = (paid / budgeted) * 100
  if (percentage > 100) return 'var(--budget-bar-color-red)'
  if (percentage >= 80) return 'var(--budget-bar-color-yellow)'
  return 'var(--budget-bar-color-green)'
}

function getOverspendLinePosition(item: CategoryBudgetSummary): number {
  const overspend = item.paid - item.budgeted
  return (1 - overspend / item.budgeted) * 100
}

function getAriaLabel(item: CategoryBudgetSummary): string {
  const label = getCategoryDisplayName(item.category)
  return `Category: ${label}, Spent: ${formatAmount(item.paid)}, Budgeted: ${formatAmount(item.budgeted)}, Status: ${item.status}`
}
</script>

<script lang="ts">
import { computed } from 'vue'
</script>

<style scoped>
.budget-progress-bars {
  --budget-bar-height-compact: 48px;
  --budget-bar-height-spacious: 72px;
  --budget-bar-gap: 12px;
  --budget-bar-padding: 16px;
  --budget-bar-color-green: #4CAF50;
  --budget-bar-color-yellow: #FFC107;
  --budget-bar-color-red: #F44336;
  --budget-bar-color-bg: #424242;

  display: flex;
  flex-direction: column;
  gap: var(--budget-bar-gap);
}

.budget-bar {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
}

.budget-bar--compact {
  height: var(--budget-bar-height-compact);
}

.budget-bar--spacious {
  height: var(--budget-bar-height-spacious);
}

.budget-bar__background {
  position: relative;
  width: 100%;
  height: 100%;
  background-color: var(--budget-bar-color-bg);
  border-radius: 8px;
  overflow: hidden;
}

.budget-bar__fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  border-radius: 8px 0 0 8px;
  transition: none;
}

.budget-bar__overspend-line {
  position: absolute;
  top: 0;
  height: 100%;
  width: 3px;
  background-color: #FFFFFF;
  z-index: 1;
}

.budget-bar__text {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--budget-bar-padding);
  z-index: 2;
  color: #E8EAF0;
  font-size: 14px;
  pointer-events: none;
}

.budget-bar__label {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
  flex-shrink: 1;
  margin-right: 12px;
}

.budget-bar__amounts {
  flex-shrink: 0;
  white-space: nowrap;
}
</style>
