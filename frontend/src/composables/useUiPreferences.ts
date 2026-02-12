import { ref, watch } from 'vue'

export type BudgetBarSize = 'compact' | 'spacious'

const BUDGET_BAR_SIZE_KEY = 'budgetBarSize'
const DEFAULT_BUDGET_BAR_SIZE: BudgetBarSize = 'compact'

// Reactive state shared across all instances
const budgetBarSize = ref<BudgetBarSize>(loadBudgetBarSize())

function loadBudgetBarSize(): BudgetBarSize {
  const stored = localStorage.getItem(BUDGET_BAR_SIZE_KEY)
  if (stored === 'compact' || stored === 'spacious') {
    return stored
  }
  return DEFAULT_BUDGET_BAR_SIZE
}

function saveBudgetBarSize(size: BudgetBarSize) {
  localStorage.setItem(BUDGET_BAR_SIZE_KEY, size)
}

// Watch for changes and persist to localStorage
watch(budgetBarSize, (newSize) => {
  saveBudgetBarSize(newSize)
})

export function useUiPreferences() {
  return {
    budgetBarSize,
  }
}
