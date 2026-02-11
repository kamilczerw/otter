<template>
  <v-bottom-sheet v-model="isOpen">
    <v-card>
      <v-card-title>
        {{ entry ? $t('entries.editEntry') : $t('entries.addEntry') }}
      </v-card-title>
      <v-card-text>
        <CategoryAutocomplete
          v-model="categoryId"
          class="mb-2"
          :disabled="!!entry"
        />
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
        <v-btn
          v-if="entry"
          class="btn-secondary-glass"
          color="error"
          @click="handleDelete"
        >
          {{ $t('common.delete') }}
        </v-btn>
        <v-spacer />
        <v-btn class="btn-secondary-glass" @click="isOpen = false">
          {{ $t('common.cancel') }}
        </v-btn>
        <v-btn class="btn-primary-glass" @click="save" :loading="saving">
          {{ $t('common.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-bottom-sheet>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CategoryAutocomplete from './CategoryAutocomplete.vue'
import type { Entry } from '@/api/types'
import { entriesApi } from '@/api/entries'
import { ApiError } from '@/api/types'
import { parseCurrencyToMinor } from '@/utils/currency'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  monthId: string
  entry: Entry | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
  deleted: []
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
})

const categoryId = ref('')
const budgeted = ref('')
const dueDay = ref<string | null>(null)
const error = ref('')
const saving = ref(false)

// Initialize form when entry changes or drawer opens
watch(() => [props.entry, props.modelValue], () => {
  if (props.modelValue) {
    if (props.entry) {
      categoryId.value = props.entry.category.id
      budgeted.value = (props.entry.budgeted / 100).toFixed(2)
      dueDay.value = props.entry.due_day?.toString() ?? null
    } else {
      categoryId.value = ''
      budgeted.value = ''
      dueDay.value = null
    }
    error.value = ''
  }
}, { immediate: true })

async function save() {
  if (!categoryId.value || !budgeted.value) return
  error.value = ''
  saving.value = true
  try {
    const payload = {
      category_id: categoryId.value,
      budgeted: parseCurrencyToMinor(budgeted.value),
      due_day: dueDay.value ? parseInt(dueDay.value) : undefined,
    }

    if (props.entry) {
      // For updates, only send changed fields
      const updatePayload: { budgeted?: number; due_day?: number | null } = {}
      if (payload.budgeted !== props.entry.budgeted) {
        updatePayload.budgeted = payload.budgeted
      }
      if ((payload.due_day ?? null) !== props.entry.due_day) {
        updatePayload.due_day = payload.due_day ?? null
      }

      if (Object.keys(updatePayload).length > 0) {
        await entriesApi.update(props.monthId, props.entry.id, updatePayload)
      }
    } else {
      await entriesApi.create(props.monthId, payload)
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

async function handleDelete() {
  if (!props.entry) return
  error.value = ''
  try {
    await entriesApi.delete(props.monthId, props.entry.id)
    isOpen.value = false
    emit('deleted')
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = t(`errors.${e.code}`, e.details || {})
    }
  }
}
</script>
