<template>
  <v-autocomplete
    v-model="selected"
    :items="items"
    :label="$t('entries.category')"
    item-title="name"
    item-value="id"
    variant="outlined"
    density="compact"
    :search="search"
    @update:search="onSearch"
    :no-data-text="search ? undefined : $t('common.noData')"
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
import { ref, onMounted, watch } from 'vue'
import { categoriesApi } from '@/api/categories'
import type { Category } from '@/api/types'

const model = defineModel<string>()

const items = ref<Category[]>([])
const selected = ref<string>('')
const search = ref('')

watch(selected, (val) => {
  model.value = val
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
    console.error('Failed to create category', e)
  }
}

onMounted(loadCategories)
</script>
