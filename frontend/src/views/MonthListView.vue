<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <h1 class="text-h5 mb-4">{{ $t('months.title') }}</h1>
      </v-col>
    </v-row>

    <!-- Create Month -->
    <v-row>
      <v-col cols="12" sm="6" md="4">
        <v-text-field
          v-model="newMonth"
          :label="$t('months.newMonth')"
          placeholder="YYYY-MM"
          variant="outlined"
          density="compact"
          append-inner-icon="mdi-plus"
          @click:append-inner="createMonth"
          @keyup.enter="createMonth"
          :error-messages="createError"
        />
      </v-col>
    </v-row>

    <!-- Month List -->
    <v-row>
      <v-col cols="12">
        <v-list>
          <v-list-item
            v-for="month in months"
            :key="month.id"
            :title="month.month"
            :to="`/months/${month.month}/budget`"
          >
            <template #append>
              <v-icon>mdi-chevron-right</v-icon>
            </template>
          </v-list-item>
        </v-list>
        <v-alert v-if="months.length === 0 && !loading" type="info" variant="tonal">
          {{ $t('common.noData') }}
        </v-alert>
      </v-col>
    </v-row>

    <v-progress-linear v-if="loading" indeterminate />
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
