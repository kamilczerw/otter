<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('months.title') }}</h1>

    <!-- Create Month -->
    <div class="mb-6" style="max-width: 320px;">
      <v-text-field
        v-model="newMonth"
        :label="$t('months.newMonth')"
        placeholder="YYYY-MM"
        append-inner-icon="mdi-plus"
        @click:append-inner="createMonth"
        @keyup.enter="createMonth"
        :error-messages="createError"
      />
    </div>

    <!-- Month List -->
    <div class="glass-card">
      <v-list>
        <v-list-item
          v-for="month in months"
          :key="month.id"
          :title="month.month"
          :to="`/months/${month.month}/budget`"
        >
          <template #append>
            <v-icon color="secondary">mdi-chevron-right</v-icon>
          </template>
        </v-list-item>
      </v-list>
      <v-alert v-if="months.length === 0 && !loading" type="info" variant="tonal" class="ma-4">
        {{ $t('common.noData') }}
      </v-alert>
    </div>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mt-4" />
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { monthsApi } from '@/api/months'
import type { Month } from '@/api/types'
import { ApiError } from '@/api/types'

const { t } = useI18n()

const months = ref<Month[]>([])
const loading = ref(false)
const newMonth = ref('')
const createError = ref('')

async function loadMonths() {
  loading.value = true
  try {
    months.value = await monthsApi.list()
  } catch (e) {
    console.error('Failed to load months', e)
  } finally {
    loading.value = false
  }
}

async function createMonth() {
  if (!newMonth.value.trim()) return
  createError.value = ''
  try {
    await monthsApi.create({ month: newMonth.value.trim() })
    newMonth.value = ''
    await loadMonths()
  } catch (e) {
    if (e instanceof ApiError) {
      createError.value = t(`errors.${e.code}`, e.details || {})
    }
  }
}

onMounted(loadMonths)
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
