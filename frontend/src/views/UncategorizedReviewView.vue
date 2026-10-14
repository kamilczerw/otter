<!--
  Review queue for bulk-categorizing CSV-imported transactions that have no
  budget entry yet. Shows one "head" transaction at a time, lets the user
  build a match rule (pre-filled from the backend's derived suggestion),
  previews how many transactions it would match, and saves it — which
  categorizes all matches and advances the queue.
-->
<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('uncategorized.title') }}</h1>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />

    <template v-else>
      <v-alert v-if="!head" type="success" variant="tonal">
        {{ $t('uncategorized.allCaughtUp') }}
      </v-alert>

      <template v-else>
        <p class="text-body-2 text-medium-emphasis mb-4">
          {{ $t('uncategorized.remaining', { count: remaining }) }}
          <template v-if="skip > 0">&middot; {{ $t('uncategorized.skippedCount', { count: skip }) }}</template>
        </p>

        <!-- Head transaction details -->
        <v-card variant="tonal" class="mb-4 pa-3">
          <div class="text-caption text-medium-emphasis mb-2">{{ $t('uncategorized.headTransaction') }}</div>
          <div>{{ $t('transactions.title') }}: {{ head.transaction.title || $t('transactions.noTitle') }}</div>
          <div>{{ $t('transactions.amount') }}: {{ formatCurrency(head.transaction.amount) }}</div>
          <div>{{ $t('transactions.date') }}: {{ head.transaction.date }}</div>
          <template v-if="head.transaction.metadata">
            <div v-if="head.transaction.metadata.merchant">
              {{ $t('transactions.merchant') }}: {{ head.transaction.metadata.merchant }}
            </div>
            <div v-if="head.transaction.metadata.city">
              {{ $t('transactions.city') }}: {{ head.transaction.metadata.city }}
            </div>
            <div v-if="head.transaction.metadata.country">
              {{ $t('transactions.country') }}: {{ head.transaction.metadata.country }}
            </div>
            <div v-if="head.transaction.metadata.raw_type">
              {{ $t('transactions.rawType') }}: {{ head.transaction.metadata.raw_type }}
            </div>
            <div v-if="head.transaction.metadata.card_last4">
              {{ $t('transactions.cardLast4') }}: {{ head.transaction.metadata.card_last4 }}
            </div>
            <div v-if="head.transaction.metadata.currency">
              {{ $t('transactions.currency') }}: {{ head.transaction.metadata.currency }}
            </div>
            <div
              v-if="head.transaction.metadata.raw_description"
              :class="showFullDescription ? 'description-full' : 'description-truncated'"
              @click="showFullDescription = !showFullDescription"
            >
              {{ $t('transactions.rawDescription') }}: {{ head.transaction.metadata.raw_description }}
            </div>
          </template>
          <v-alert v-else type="info" variant="tonal" density="compact" class="mt-2">
            {{ $t('uncategorized.noMetadata') }}
          </v-alert>
        </v-card>

        <!-- Criteria editor -->
        <div class="glass-card pa-3 mb-4">
          <div class="text-caption text-medium-emphasis mb-2">{{ $t('uncategorized.criteria') }}</div>
          <MatchCriterionRow
            v-for="row in rows"
            :key="row.id"
            :field="row.field"
            :match-type="row.matchType"
            :pattern="row.pattern"
            @update="(val) => updateRow(row.id, val)"
            @remove="() => removeRow(row.id)"
          />
          <v-btn variant="text" color="primary" size="small" prepend-icon="mdi-plus" @click="addRow">
            {{ $t('uncategorized.addCriterion') }}
          </v-btn>

          <p class="text-body-2 mt-3">
            <template v-if="previewLoading">{{ $t('common.loading') }}</template>
            <template v-else-if="previewCount !== null">
              {{ $t('uncategorized.matchCount', { count: previewCount }) }}
            </template>
          </p>
        </div>

        <!-- Category + save -->
        <v-combobox
          v-model="category"
          :items="categoryNames"
          :label="$t('uncategorized.category')"
          class="mb-4"
        />

        <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mb-4">
          {{ error }}
        </v-alert>

        <div class="d-flex justify-end ga-2">
          <v-btn variant="text" :loading="loading" @click="handleSkip">
            {{ $t('uncategorized.skip') }}
          </v-btn>
          <v-btn class="btn-primary-glass" :loading="saving" :disabled="!category" @click="handleSave">
            {{ $t('uncategorized.saveAndApply') }}
          </v-btn>
        </div>
      </template>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { uncategorizedApi } from '@/api/uncategorized'
import { categoriesApi } from '@/api/categories'
import { ApiError } from '@/api/types'
import type { MatchCriterion, MatchField, UncategorizedHead } from '@/api/types'
import { formatCurrency } from '@/utils/currency'
import MatchCriterionRow from '@/components/uncategorized/MatchCriterionRow.vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

/** A single editable criterion row, with a stable id for v-for keys. */
interface CriterionRow {
  id: number
  field: MatchField
  matchType: 'contains' | 'regex'
  pattern: string
}

const loading = ref(false)
const saving = ref(false)
const previewLoading = ref(false)
const error = ref('')

const remaining = ref(0)
const head = ref<UncategorizedHead | null>(null)
const rows = ref<CriterionRow[]>([])
const category = ref<string>('')
const categoryNames = ref<string[]>([])
const previewCount = ref<number | null>(null)
const showFullDescription = ref(false)

let nextRowId = 0
let previewTimer: ReturnType<typeof setTimeout> | undefined

/**
 * The raw (un-wrapped) skip offset, sourced from the route's `skip` query param so that
 * reloading the page preserves the user's position in the queue. Defaults to 0 for an
 * absent or non-numeric value.
 */
const skip = computed<number>(() => {
  const parsed = Number(route.query.skip)
  return Number.isInteger(parsed) && parsed > 0 ? parsed : 0
})

/**
 * Updates the `skip` route query param without pushing a new history entry, since every
 * skip/save click shouldn't pollute browser back/forward navigation.
 *
 * @param value - The new skip offset. A value of 0 clears the query param entirely.
 */
async function setSkip(value: number): Promise<void> {
  const query = { ...route.query, skip: value > 0 ? String(value) : undefined }
  await router.replace({ query })
}

/**
 * Builds a fresh criterion row with an empty pattern, assigning it a unique id.
 */
function makeEmptyRow(): CriterionRow {
  return { id: nextRowId++, field: 'merchant', matchType: 'contains', pattern: '' }
}

/**
 * Converts a backend-derived rule (or null) into editable criterion rows.
 *
 * @param derivedRule - The rule the backend suggests for the head transaction, if any.
 */
function rowsFromDerivedRule(derivedRule: MatchCriterion[] | null): CriterionRow[] {
  if (!derivedRule) return []
  return derivedRule.map((c) => ({
    id: nextRowId++,
    field: c.field,
    matchType: c.regex != null ? 'regex' : 'contains',
    pattern: (c.contains ?? c.regex) || '',
  }))
}

/**
 * Resets all form state (criteria, category, preview) for a newly loaded head transaction.
 *
 * @param newHead - The new head transaction, or null if the queue is empty.
 */
function resetFormForHead(newHead: UncategorizedHead | null): void {
  head.value = newHead
  rows.value = rowsFromDerivedRule(newHead?.derived_rule ?? null)
  category.value = ''
  previewCount.value = null
  showFullDescription.value = false
}

/**
 * Loads the uncategorized-transaction queue and resets the form for the new head.
 */
async function loadQueue(): Promise<void> {
  loading.value = true
  error.value = ''
  try {
    const queue = await uncategorizedApi.getQueue(skip.value)
    remaining.value = queue.remaining
    resetFormForHead(queue.head)
  } catch (e) {
    console.error('Failed to load uncategorized queue', e)
  } finally {
    loading.value = false
  }
}

/**
 * Loads all category names, used as combobox suggestions for the category field.
 */
async function loadCategoryNames(): Promise<void> {
  try {
    const categories = await categoriesApi.list()
    categoryNames.value = categories.map((c) => c.name)
  } catch (e) {
    console.error('Failed to load categories', e)
  }
}

/**
 * Advances the queue past the current head without deciding on it, by incrementing the
 * `skip` route query param and reloading the queue at the new offset.
 */
async function handleSkip(): Promise<void> {
  await setSkip(skip.value + 1)
  await loadQueue()
}

/** Adds a new empty criterion row to the editor. */
function addRow(): void {
  rows.value.push(makeEmptyRow())
}

/**
 * Updates a criterion row's field/matchType/pattern in place.
 *
 * @param id - The row's stable id.
 * @param value - The updated field/matchType/pattern values.
 */
function updateRow(id: number, value: { field: MatchField; matchType: 'contains' | 'regex'; pattern: string }): void {
  const row = rows.value.find((r) => r.id === id)
  if (row) {
    row.field = value.field
    row.matchType = value.matchType
    row.pattern = value.pattern
  }
}

/**
 * Removes a criterion row by id.
 *
 * @param id - The row's stable id.
 */
function removeRow(id: number): void {
  rows.value = rows.value.filter((r) => r.id !== id)
}

/**
 * Converts editable rows into the MatchCriterion[] shape the API expects.
 */
function buildCriteria(): MatchCriterion[] {
  return rows.value.map((r) => ({
    field: r.field,
    contains: r.matchType === 'contains' ? r.pattern : null,
    regex: r.matchType === 'regex' ? r.pattern : null,
  }))
}

/** True if every row has a non-empty pattern (i.e. the criteria are complete enough to preview). */
function criteriaAreComplete(): boolean {
  return rows.value.every((r) => r.pattern.trim() !== '')
}

/**
 * Fetches a live match count for the current criteria from the API.
 * Skipped entirely when criteria are incomplete (empty pattern).
 */
async function runPreview(): Promise<void> {
  if (!criteriaAreComplete()) {
    previewCount.value = null
    return
  }
  previewLoading.value = true
  try {
    const result = await uncategorizedApi.preview({ criteria: buildCriteria() })
    previewCount.value = result.count
  } catch (e) {
    console.error('Failed to preview rule', e)
  } finally {
    previewLoading.value = false
  }
}

/** Debounces preview calls so they don't fire on every keystroke. */
function schedulePreview(): void {
  clearTimeout(previewTimer)
  previewTimer = setTimeout(runPreview, 350)
}

watch(rows, schedulePreview, { deep: true })

/**
 * Saves the current rule, applying it to all matching transactions, then
 * reloads the queue to show the next head transaction.
 */
async function handleSave(): Promise<void> {
  if (!category.value) return
  error.value = ''
  saving.value = true
  try {
    await uncategorizedApi.saveRule({ criteria: buildCriteria(), category: category.value })
    await setSkip(0)
    await loadQueue()
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  await loadQueue()
  await loadCategoryNames()
})
</script>

<style scoped>
.page-title {
  font-size: 1.375rem;
  font-weight: 700;
  color: var(--text-primary);
}

.mb-6 {
  margin-bottom: 24px;
}

.description-truncated {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.description-full {
  white-space: normal;
  cursor: pointer;
}
</style>
