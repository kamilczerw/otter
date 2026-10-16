<!--
  Single editable row of a bull-categorization rule: picks which transaction
  field to match, whether the pattern is a substring ("contains") or a regex,
  and the pattern text itself. Purely presentational — the parent view owns
  the list of rows and decides when to run previews/saves.
-->
<template>
  <div class="d-flex align-center ga-2 mb-2">
    <v-select
      :model-value="field"
      @update:model-value="onFieldChange"
      :items="fieldOptions"
      :label="$t('uncategorized.field')"
      density="compact"
      style="max-width: 160px;"
    />
    <v-btn-toggle
      :model-value="matchType"
      @update:model-value="onMatchTypeChange"
      density="compact"
      mandatory
      color="primary"
    >
      <v-btn value="contains" size="small">{{ $t('uncategorized.contains') }}</v-btn>
      <v-btn value="regex" size="small">{{ $t('uncategorized.regex') }}</v-btn>
    </v-btn-toggle>
    <v-text-field
      :model-value="pattern"
      @update:model-value="onPatternChange"
      :label="$t('uncategorized.pattern')"
      density="compact"
      hide-details
    />
    <v-btn icon size="small" variant="text" @click="$emit('remove')">
      <v-icon>mdi-close</v-icon>
    </v-btn>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { MatchField } from '@/api/types'

const { t } = useI18n()

const props = defineProps<{
  field: MatchField
  matchType: 'contains' | 'regex'
  pattern: string
}>()

const emit = defineEmits<{
  update: [value: { field: MatchField; matchType: 'contains' | 'regex'; pattern: string }]
  remove: []
}>()

// Options for the field selector, labeled via i18n rather than raw field names.
const fieldOptions: { title: string; value: MatchField }[] = [
  { title: t('uncategorized.fields.merchant'), value: 'merchant' },
  { title: t('uncategorized.fields.raw_type'), value: 'raw_type' },
  { title: t('uncategorized.fields.city'), value: 'city' },
  { title: t('uncategorized.fields.country'), value: 'country' },
  { title: t('uncategorized.fields.description'), value: 'description' },
  { title: t('uncategorized.fields.raw_fragments'), value: 'raw_fragments' },
]

/** Relays a field change, merging it with the row's current matchType/pattern. */
function onFieldChange(value: unknown): void {
  emit('update', { field: value as MatchField, matchType: props.matchType, pattern: props.pattern })
}

/** Relays a match-type change, merging it with the row's current field/pattern. */
function onMatchTypeChange(value: unknown): void {
  emit('update', { field: props.field, matchType: value as 'contains' | 'regex', pattern: props.pattern })
}

/** Relays a pattern-text change, merging it with the row's current field/matchType. */
function onPatternChange(value: unknown): void {
  emit('update', { field: props.field, matchType: props.matchType, pattern: value as string })
}
</script>
