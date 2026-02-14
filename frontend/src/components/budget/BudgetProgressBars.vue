<template>
  <div class="budget-progress-bars">
    <div
      v-for="item in categories"
      :key="item.category.id"
      class="budget-bar-wrapper"
    >
      <div
        class="budget-bar"
        :class="{ 'budget-bar--expanded': expandedEntryId === item.entry_id }"
        role="progressbar"
        :aria-valuenow="item.paid"
        :aria-valuemin="0"
        :aria-valuemax="item.budgeted"
        :aria-label="getAriaLabel(item)"
        @click="toggleExpand(item.entry_id)"
      >
        <v-progress-linear
          :model-value="getFillWidth(item)"
          :color="getBarColor(item)"
          :height="barHeight"
          :rounded="expandedEntryId === item.entry_id ? false : 'lg'"
          bg-color="#424242"
          class="budget-bar__progress"
          :class="{
            'budget-bar__progress--overspent': isOverspent(item),
            'budget-bar__progress--expanded': expandedEntryId === item.entry_id
          }"
        >
          <template v-slot:default>
            <div class="budget-bar__text">
              <span class="budget-bar__label">{{ getCategoryDisplayName(item.category) }}</span>
              <span class="budget-bar__amounts">
                {{ formatAmount(item.paid) }}/{{ formatAmount(item.budgeted) }}
                <v-icon
                  size="18"
                  class="budget-bar__chevron"
                  :class="{ 'budget-bar__chevron--expanded': expandedEntryId === item.entry_id }"
                >
                  mdi-chevron-down
                </v-icon>
              </span>
            </div>
          </template>
        </v-progress-linear>

        <!-- Overspend indicator line -->
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

      <!-- Expanded panel -->
      <v-expand-transition>
        <div v-if="expandedEntryId === item.entry_id" class="budget-panel-wrap">
          <BudgetCategoryPanel
            :entry-id="item.entry_id"
            @edit-budget="$emit('edit-budget', item)"
            @add-transaction="$emit('add-transaction', item)"
            @edit-transaction="(tx) => $emit('edit-transaction', tx)"
          />
        </div>
      </v-expand-transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CategoryBudgetSummary } from '@/api/types'
import type { Transaction } from '@/api/types'
import { getCategoryDisplayName } from '@/utils/category'
import { formatCurrency } from '@/utils/currency'
import BudgetCategoryPanel from './BudgetCategoryPanel.vue'

const { t } = useI18n()

interface Props {
  categories: CategoryBudgetSummary[]
  barSize?: 'compact' | 'spacious'
  expandedEntryId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  barSize: 'compact',
  expandedEntryId: null,
})

const emit = defineEmits<{
  'update:expandedEntryId': [id: string | null]
  'edit-budget': [item: CategoryBudgetSummary]
  'add-transaction': [item: CategoryBudgetSummary]
  'edit-transaction': [tx: Transaction]
}>()

const barHeight = computed(() => props.barSize === 'compact' ? 48 : 72)

function toggleExpand(entryId: string) {
  emit('update:expandedEntryId', props.expandedEntryId === entryId ? null : entryId)
}

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

.budget-bar-wrapper {
  width: 100%;
  will-change: contents;
}

.budget-bar {
  position: relative;
  width: 100%;
  cursor: pointer;
}

.budget-bar__progress >>> .v-progress-linear__determinate {
  opacity: 0.50;
}

.budget-bar__progress--overspent >>> .v-progress-linear__determinate {
  opacity: 0.65;
}

.budget-bar__progress {
  transition: border-radius 0.25s ease !important;
}

.budget-bar__progress--expanded {
  border-radius: 8px 8px 0 0 !important;
}

.budget-bar__progress >>> .v-progress-linear__background {
  transition: border-radius 0.25s ease !important;
}

.budget-bar__progress--expanded >>> .v-progress-linear__background {
  border-radius: 8px 8px 0 0 !important;
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
  display: flex;
  align-items: center;
  gap: 6px;
}

.budget-bar__chevron {
  transition: transform 0.25s ease;
  opacity: 0.6;
}

.budget-bar__chevron--expanded {
  transform: rotate(180deg);
  opacity: 1;
}

.budget-panel-wrap {
  overflow: hidden;
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
