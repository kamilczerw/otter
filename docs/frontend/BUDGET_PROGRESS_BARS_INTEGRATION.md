# BudgetProgressBars Integration Guide

This document shows how to integrate the BudgetProgressBars component with the new budget bar size setting.

## Example Integration

To use BudgetProgressBars with the user's preferred size setting:

```vue
<script setup lang="ts">
import { useUiPreferences } from '@/composables/useUiPreferences'
import BudgetProgressBars from '@/components/budget/BudgetProgressBars.vue'
import type { CategoryBudgetSummary } from '@/api/types'

const { budgetBarSize } = useUiPreferences()

// Your categories data
const categories = ref<CategoryBudgetSummary[]>([])
</script>

<template>
  <BudgetProgressBars
    :categories="categories"
    :bar-size="budgetBarSize"
  />
</template>
```

## Integration Points

The BudgetProgressBars component can be added to:

1. **MonthBudgetView** - Display budget progress bars in the main budget view
2. **Summary sections** - Show visual budget status alongside or instead of tables
3. **Dashboard views** - Quick overview of category spending

## Reactivity

The `budgetBarSize` from `useUiPreferences()` is reactive, so any changes made in the Settings will automatically update all BudgetProgressBars components across the application without requiring page refreshes.

## User Flow

1. User opens Settings (from bottom navigation "More" tab)
2. User finds "Budget bar size" under "UI Settings"
3. User selects between "Compact" (48px height) or "Spacious" (72px height)
4. Change is immediately reflected in all budget progress bars
5. Preference is persisted in localStorage

## Technical Details

- **Storage key**: `budgetBarSize`
- **Default value**: `'compact'`
- **Possible values**: `'compact'` | `'spacious'`
- **Composable**: `useUiPreferences` in `/src/composables/useUiPreferences.ts`
