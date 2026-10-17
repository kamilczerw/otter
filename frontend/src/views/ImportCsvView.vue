<!--
  CSV transaction import screen: pick a charset, upload a bank CSV export,
  and see the import result (counts + any per-row failures) without leaving
  the application. Mirrors RulesView.vue's upload/result pattern.
-->
<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('import.title') }}</h1>

    <v-select
      v-model="charset"
      :items="charsetOptions"
      :label="$t('import.charset')"
      class="mb-4"
      style="max-width: 320px"
    />

    <div class="d-flex ga-2 mb-6 flex-wrap">
      <v-btn variant="text" color="primary" size="small" prepend-icon="mdi-upload" @click="triggerFilePicker">
        {{ $t('import.uploadButton') }}
      </v-btn>
      <input
        ref="fileInput"
        type="file"
        accept=".csv"
        class="d-none"
        data-testid="import-file-input"
        @change="handleFileSelected"
      >
    </div>

    <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mb-4">
      {{ error }}
    </v-alert>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />

    <div v-if="result" class="glass-card pa-4">
      <div class="d-flex ga-4 flex-wrap mb-2">
        <v-chip color="primary" variant="tonal">{{ $t('import.imported') }}: {{ result.imported }}</v-chip>
        <v-chip color="secondary" variant="tonal">{{ $t('import.duplicates') }}: {{ result.duplicates }}</v-chip>
        <v-chip color="warning" variant="tonal">{{ $t('import.uncategorized') }}: {{ result.uncategorized }}</v-chip>
      </div>

      <v-expansion-panels v-if="result.failed.length > 0">
        <v-expansion-panel>
          <v-expansion-panel-title>
            {{ $t('import.failedRows', { count: result.failed.length }) }}
          </v-expansion-panel-title>
          <v-expansion-panel-text>
            <v-list>
              <v-list-item v-for="(row, index) in result.failed" :key="index">
                <v-list-item-title>{{ `Row ${row.row}: ${row.reason}` }}</v-list-item-title>
                <v-list-item-subtitle>{{ row.raw }}</v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { importsApi, type Charset } from '@/api/imports'
import { ApiError } from '@/api/types'
import type { ImportResponse } from '@/api/types'

const { t } = useI18n()

/** The two charsets the backend's CSV decoder supports; UTF-8 is the default. */
const charsetOptions: { title: string; value: Charset }[] = [
  { title: 'UTF-8', value: 'utf-8' },
  { title: 'Windows-1250', value: 'windows-1250' },
]

const charset = ref<Charset>('utf-8')
const loading = ref(false)
const error = ref('')
const result = ref<ImportResponse | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)

/** Opens the native file picker for selecting a CSV file to import. */
function triggerFilePicker(): void {
  fileInput.value?.click()
}

/** Uploads the file selected from the native file picker and reports the import result. */
async function handleFileSelected(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  error.value = ''
  result.value = null
  loading.value = true
  try {
    result.value = await importsApi.importCsv(file, charset.value)
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  } finally {
    loading.value = false
  }
}
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
