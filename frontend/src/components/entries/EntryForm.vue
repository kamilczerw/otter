<template>
  <v-card>
    <v-card-title>{{ $t('entries.addEntry') }}</v-card-title>
    <v-card-text>
      <CategoryAutocomplete v-model="selectedCategoryId" class="mb-2" />
      <v-text-field
        v-model="budgeted"
        :label="$t('entries.budgeted')"
        type="number"
        class="mb-2"
      />
      <v-text-field
        v-model="dueDay"
        :label="$t('entries.dueDay')"
        type="number"
        placeholder="1-31"
        clearable
      />
      <v-alert v-if="error" type="error" variant="tonal" density="compact" class="mt-2">
        {{ error }}
      </v-alert>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn class="btn-secondary-glass" @click="$emit('cancel')">{{ $t('common.cancel') }}</v-btn>
      <v-btn class="btn-primary-glass" @click="save" :loading="saving">{{ $t('common.save') }}</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CategoryAutocomplete from './CategoryAutocomplete.vue'
import { entriesApi } from '@/api/entries'
import { ApiError } from '@/api/types'
import { parseCurrencyToMinor } from '@/utils/currency'

const { t } = useI18n()

const props = defineProps<{
  monthId: string
}>()

const emit = defineEmits<{
  saved: []
  cancel: []
}>()

const selectedCategoryId = ref('')
const budgeted = ref('')
const dueDay = ref<string | null>(null)
const error = ref('')
const saving = ref(false)

async function save() {
  if (!selectedCategoryId.value || !budgeted.value) return
  error.value = ''
  saving.value = true
  try {
    await entriesApi.create(props.monthId, {
      category_id: selectedCategoryId.value,
      budgeted: parseCurrencyToMinor(budgeted.value),
      due_day: dueDay.value ? parseInt(dueDay.value) : undefined,
    })
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
