<template>
  <div class="budget-progress-bars">
    <div
      v-for="item in categories"
      :key="item.category.id"
      class="budget-bar"
      role="progressbar"
      :aria-valuenow="item.paid"
      :aria-valuemin="0"
      :aria-valuemax="item.budgeted"
      :aria-label="getAriaLabel(item)"
    >
      <v-progress-linear
        :model-value="getFillWidth(item)"
        :color="getBarColor(item)"
        :height="barHeight"
        rounded="lg"
        bg-color="#424242"
        class="budget-bar__progress"
        :class="{ 'budget-bar__progress--overspent': isOverspent(item) }"
      >
        <template v-slot:default>
          <div class="budget-bar__text">
            <span class="budget-bar__label">{{ getCategoryDisplayName(item.category) }}</span>
            <span class="budget-bar__amounts">{{ formatAmount(item.paid) }}/{{ formatAmount(item.budgeted) }}</span>
          </div>
        </template>
      </v-progress-linear>

      <!-- Overspend indicator line (only for non-overspent partial fills) -->
      <div
        v-if="item.paid > item.budgeted && item.budgeted > 0 && !isOverspent(item)"
        class="budget-bar__overspend-line"
        :style="{ left: getOverspendLinePosition(item) + '%' }"
      />

      <!-- Overspend badge -->
      <div
        v-if="isOverspent(item)"
        class="budget-bar__overspend-badge"
      >
        {{ t('budget.overBudget', { amount: formatAmount(item.paid - item.budgeted) }) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CategoryBudgetSummary } from '@/api/types'
import { getCategoryDisplayName } from '@/utils/category'
import { formatCurrency } from '@/utils/currency'

const { t } = useI18n()

interface Props {
  categories: CategoryBudgetSummary[]
  barSize?: 'compact' | 'spacious'
}

const props = withDefaults(defineProps<Props>(), {
  barSize: 'compact',
})

const barHeight = computed(() => props.barSize === 'compact' ? 48 : 72)

function formatAmount(minorUnits: number): string {
  return formatCurrency(minorUnits)
}

function isOverspent(item: CategoryBudgetSummary): boolean {
  return item.paid > item.budgeted
}

function getFillWidth(item: CategoryBudgetSummary): number {
  if (isOverspent(item)) return 100
  if (item.budgeted === 0) return 0
  return Math.min((item.paid / item.budgeted) * 100, 100)
}

function getBarColor(item: CategoryBudgetSummary): string {
  if (isOverspent(item)) return 'var(--color-danger)'
  if (item.paid === 0) return '#4CAF50'
  return 'var(--color-success)'
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

<style scoped>
.budget-progress-bars {
  --budget-bar-gap: 12px;
  --budget-bar-padding: 16px;

  display: flex;
  flex-direction: column;
  gap: var(--budget-bar-gap);
}

.budget-bar {
  position: relative;
  width: 100%;
}

.budget-bar__progress >>> .v-progress-linear__determinate {
  opacity: 0.50;
}

.budget-bar__progress--overspent >>> .v-progress-linear__determinate {
  opacity: 0.65;
}

.budget-bar__overspend-line {
  position: absolute;
  top: 0;
  height: 100%;
  width: 3px;
  background-color: var(--color-danger);
  z-index: 2;
  pointer-events: none;
  border-radius: 8px;
}

.budget-bar__text {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--budget-bar-padding);
  color: #E8EAF0;
  font-size: 14px;
  pointer-events: none;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
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

.budget-bar__overspend-badge {
  position: absolute;
  top: -8px;
  right: -4px;
  background-color: var(--color-danger);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  padding: 4px 8px;
  border-radius: 10px;
  white-space: nowrap;
  z-index: 3;
  pointer-events: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
}
</style>
