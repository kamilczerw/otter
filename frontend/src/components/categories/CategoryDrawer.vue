<template>
  <v-bottom-sheet v-model="isOpen">
    <v-card>
      <v-card-title>
        {{ category ? $t('categories.rename') : $t('categories.newCategory') }}
      </v-card-title>
      <v-card-text>
        <v-text-field
          v-model="categoryName"
          :label="$t('categories.name')"
          placeholder="e.g., bills/electricity"
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
import type { Category } from '@/api/types'
import { categoriesApi } from '@/api/categories'
import { ApiError } from '@/api/types'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  category: Category | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  saved: []
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
})

const categoryName = ref('')
const error = ref('')
const saving = ref(false)

// Initialize form when category changes or drawer opens
watch(() => [props.category, props.modelValue], () => {
  if (props.modelValue) {
    if (props.category) {
      categoryName.value = props.category.name
    } else {
      categoryName.value = ''
    }
    error.value = ''
  }
}, { immediate: true })

async function save() {
  if (!categoryName.value.trim()) return
  error.value = ''
  saving.value = true
  try {
    if (props.category) {
      await categoriesApi.update(props.category.id, { name: categoryName.value.trim() })
    } else {
      await categoriesApi.create({ name: categoryName.value.trim() })
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
