import type { Category, CategorySummary } from '@/api/types'

/**
 * Returns the display name for a category.
 * If the category has a label, it returns the label.
 * Otherwise, it returns the name.
 */
export function getCategoryDisplayName(category: Category | CategorySummary): string {
  return category.label || category.name
}
