<template>
  <v-autocomplete
    ref="autocompleteRef"
    v-model="selected"
    :items="items"
    :label="hideLabel ? undefined : $t('entries.category')"
    :placeholder="hideLabel ? $t('entries.category') : undefined"
    item-title="name"
    item-value="id"
    variant="outlined"
    density="compact"
    :search="search"
    @update:search="onSearch"
    :no-data-text="search ? undefined : $t('common.noData')"
    bg-color="surface"
  >
    <template #no-data>
      <v-list-item v-if="search" @click="createCategory">
        <v-list-item-title>
          {{ $t('common.create') }}: "{{ search }}"
        </v-list-item-title>
      </v-list-item>
    </template>
  </v-autocomplete>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { categoriesApi } from '@/api/categories'
import { ApiError } from '@/api/types'
import type { Category } from '@/api/types'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  autofocus?: boolean
  hideLabel?: boolean
}>(), {
  autofocus: false,
  hideLabel: false,
})

const emit = defineEmits<{
  error: [message: string]
}>()

const model = defineModel<string>()

const autocompleteRef = ref()
const items = ref<Category[]>([])
const selected = ref<string>('')
const search = ref('')

watch(selected, (val) => {
  model.value = val
})

watch(() => model.value, (val) => {
  if (val !== selected.value) {
    selected.value = val || ''
  }
})

function onSearch(val: string) {
  search.value = val || ''
}

async function loadCategories() {
  try {
    items.value = await categoriesApi.list()
  } catch (e) {
    console.error('Failed to load categories', e)
  }
}

async function createCategory() {
  if (!search.value.trim()) return
  try {
    const cat = await categoriesApi.create({ name: search.value.trim() })
    items.value.push(cat)
    selected.value = cat.id
    search.value = ''
  } catch (e) {
    if (e instanceof ApiError) {
      emit('error', t(`errors.${e.code}`, e.details || {}))
    }
  }
}

onMounted(async () => {
  await loadCategories()
  if (props.autofocus) {
    await nextTick()
    autocompleteRef.value?.focus()
  }
})
</script>
