<!--
  Lists the full categorization rule set: read-only config-default rules
  first, then the user's own rules (editable/deletable), with header actions
  to create a new rule or import rules from a `.toml` file.
-->
<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('rules.title') }}</h1>

    <div class="d-flex ga-2 mb-6 flex-wrap">
      <v-btn variant="text" color="primary" size="small" prepend-icon="mdi-plus" @click="openCreateSheet">
        {{ $t('rules.newRule') }}
      </v-btn>
      <v-btn variant="text" color="primary" size="small" prepend-icon="mdi-upload" @click="triggerImportPicker">
        {{ $t('rules.importFile') }}
      </v-btn>
      <input
        ref="importInput"
        type="file"
        accept=".toml"
        class="d-none"
        data-testid="import-file-input"
        @change="handleImportFileSelected"
      >
    </div>

    <v-alert v-if="importMessage" type="info" variant="tonal" density="compact" class="mb-4">
      {{ importMessage }}
    </v-alert>
    <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mb-4">
      {{ error }}
    </v-alert>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />

    <template v-else>
      <div class="glass-card mb-6">
        <div class="text-caption text-medium-emphasis pa-3 pb-0">{{ $t('rules.defaultRules') }}</div>
        <v-list>
          <v-list-item
            v-for="(rule, index) in defaultRules"
            :key="`default-${index}`"
            :title="rule.category"
            :subtitle="formatCriteria(rule.criteria)"
            class="text-medium-emphasis"
          />
          <v-alert v-if="defaultRules.length === 0" type="info" variant="tonal" density="compact" class="ma-4">
            {{ $t('common.noData') }}
          </v-alert>
        </v-list>
      </div>

      <div class="glass-card">
        <div class="text-caption text-medium-emphasis pa-3 pb-0">{{ $t('rules.yourRules') }}</div>
        <v-list>
          <v-list-item
            v-for="rule in userRules"
            :key="rule.id!"
            :title="rule.category"
            :subtitle="formatCriteria(rule.criteria)"
          >
            <template #append>
              <v-btn icon size="small" variant="text" @click="openEditSheet(rule)">
                <v-icon>mdi-pencil</v-icon>
              </v-btn>
              <v-btn icon size="small" variant="text" @click="handleDelete(rule)">
                <v-icon>mdi-delete</v-icon>
              </v-btn>
            </template>
          </v-list-item>
        </v-list>
        <v-alert v-if="userRules.length === 0" type="info" variant="tonal" density="compact" class="ma-4">
          {{ $t('rules.noUserRules') }}
        </v-alert>
      </div>
    </template>

    <RuleEditorSheet v-model="sheetOpen" :rule="selectedRule" @saved="loadRules" />
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { rulesApi } from '@/api/rules'
import { ApiError } from '@/api/types'
import type { MatchCriterion, MatchingRule } from '@/api/types'
import RuleEditorSheet from '@/components/rules/RuleEditorSheet.vue'

const { t } = useI18n()

const rules = ref<MatchingRule[]>([])
const loading = ref(false)
const error = ref('')
const importMessage = ref('')
const sheetOpen = ref(false)
const selectedRule = ref<MatchingRule | null>(null)
const importInput = ref<HTMLInputElement | null>(null)

/** Config-defined default rules: read-only, listed first. */
const defaultRules = computed(() => rules.value.filter((r) => !r.editable))
/** The user's own stored rules: editable/deletable, listed after the defaults. */
const userRules = computed(() => rules.value.filter((r) => r.editable))

/** Renders a rule's match criteria as a compact, human-readable summary string. */
function formatCriteria(criteria: MatchCriterion[]): string {
  return criteria
    .map((c) => `${c.field}: ${c.contains ?? c.regex ?? ''}`)
    .join(' & ')
}

/** Loads the full merged rule set from the API. */
async function loadRules(): Promise<void> {
  loading.value = true
  error.value = ''
  try {
    rules.value = await rulesApi.list()
  } catch (e) {
    console.error('Failed to load rules', e)
  } finally {
    loading.value = false
  }
}

/** Opens the editor sheet pre-filled for creating a new rule. */
function openCreateSheet(): void {
  selectedRule.value = null
  sheetOpen.value = true
}

/**
 * Opens the editor sheet pre-filled with an existing user rule.
 *
 * @param rule - The rule to edit.
 */
function openEditSheet(rule: MatchingRule): void {
  selectedRule.value = rule
  sheetOpen.value = true
}

/**
 * Deletes a user rule after the browser's native confirmation, then reloads the list.
 *
 * @param rule - The rule to delete.
 */
async function handleDelete(rule: MatchingRule): Promise<void> {
  if (!rule.id) return
  if (!window.confirm(t('rules.deleteConfirmText'))) return
  error.value = ''
  try {
    await rulesApi.remove(rule.id)
    await loadRules()
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  }
}

/** Opens the native file picker for importing a rules file. */
function triggerImportPicker(): void {
  importInput.value?.click()
}

/** Uploads the file selected from the native file picker and reports the import summary. */
async function handleImportFileSelected(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  error.value = ''
  importMessage.value = ''
  try {
    const result = await rulesApi.importFile(file)
    importMessage.value = t('rules.importSummary', {
      imported: result.imported,
      skipped: result.skipped,
      failed: result.failed.length,
    })
    await loadRules()
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  }
}

onMounted(loadRules)
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
</style>
