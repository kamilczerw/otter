<!--
  Bottom-sheet form for creating or editing a user-stored categorization
  rule: a list of match-criterion rows (reusing MatchCriterionRow) plus a
  target category. Used for both create (rule=null) and edit (rule set).
-->
<template>
  <v-bottom-sheet v-model="isOpen">
    <v-card>
      <v-card-title>
        {{ rule ? $t('rules.editRule') : $t('rules.newRule') }}
      </v-card-title>
      <v-card-text>
        <MatchCriterionRow
          v-for="row in rows"
          :key="row.id"
          :field="row.field"
          :match-type="row.matchType"
          :pattern="row.pattern"
          @update="(val) => updateRow(row.id, val)"
          @remove="() => removeRow(row.id)"
        />
        <v-btn variant="text" color="primary" size="small" prepend-icon="mdi-plus" class="mb-4" @click="addRow">
          {{ $t('uncategorized.addCriterion') }}
        </v-btn>

        <v-text-field
          v-model="category"
          :label="$t('rules.category')"
          autofocus
        />

        <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mt-2">
          {{ error }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn class="btn-secondary-glass" @click="isOpen = false">
          {{ $t('common.cancel') }}
        </v-btn>
        <v-btn class="btn-primary-glass" :loading="saving" :disabled="!canSave" @click="save">
          {{ $t('common.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-bottom-sheet>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { MatchCriterion, MatchField, MatchingRule } from '@/api/types'
import { ApiError } from '@/api/types'
import { rulesApi } from '@/api/rules'
import MatchCriterionRow from '@/components/uncategorized/MatchCriterionRow.vue'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  rule: MatchingRule | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
})

/** A single editable criterion row, with a stable id for v-for keys. */
interface CriterionRow {
  id: number
  field: MatchField
  matchType: 'contains' | 'regex'
  pattern: string
}

const rows = ref<CriterionRow[]>([])
const category = ref('')
const error = ref('')
const saving = ref(false)
let nextRowId = 0

/** Builds a fresh criterion row with an empty pattern, assigning it a unique id. */
function makeEmptyRow(): CriterionRow {
  return { id: nextRowId++, field: 'merchant', matchType: 'contains', pattern: '' }
}

/** Converts a rule's match criteria into editable rows. */
function rowsFromCriteria(criteria: MatchCriterion[]): CriterionRow[] {
  return criteria.map((c) => ({
    id: nextRowId++,
    field: c.field,
    matchType: c.regex != null ? 'regex' : 'contains',
    pattern: (c.contains ?? c.regex) || '',
  }))
}

// Resets the form whenever the sheet opens, for either edit (pre-filled from `rule`) or create (one empty row).
watch(() => [props.rule, props.modelValue], () => {
  if (!props.modelValue) return
  rows.value = props.rule ? rowsFromCriteria(props.rule.criteria) : [makeEmptyRow()]
  category.value = props.rule?.category ?? ''
  error.value = ''
}, { immediate: true })

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

/** Converts editable rows into the MatchCriterion[] shape the API expects. */
function buildCriteria(): MatchCriterion[] {
  return rows.value.map((r) => ({
    field: r.field,
    contains: r.matchType === 'contains' ? r.pattern : null,
    regex: r.matchType === 'regex' ? r.pattern : null,
  }))
}

/** True once there's at least one criterion row with a non-empty pattern and a category. */
const canSave = computed(() => {
  return category.value.trim() !== '' && rows.value.length > 0 && rows.value.every((r) => r.pattern.trim() !== '')
})

/** Creates or updates the rule depending on whether `rule` prop is set, then closes and notifies the parent. */
async function save(): Promise<void> {
  if (!canSave.value) return
  error.value = ''
  saving.value = true
  try {
    const data = { criteria: buildCriteria(), category: category.value.trim() }
    if (props.rule?.id) {
      await rulesApi.update(props.rule.id, data)
    } else {
      await rulesApi.create(data)
    }
    isOpen.value = false
    emit('saved')
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  } finally {
    saving.value = false
  }
}
</script>
