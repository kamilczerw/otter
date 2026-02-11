<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('categories.title') }}</h1>

    <!-- Add Category Button -->
    <div class="mb-6">
      <v-btn
        variant="text"
        color="primary"
        size="small"
        prepend-icon="mdi-plus"
        @click="openAddDrawer"
      >
        {{ $t('categories.newCategory') }}
      </v-btn>
    </div>

    <!-- Category List -->
    <div class="glass-card">
      <v-list>
        <v-list-item
          v-for="category in categories"
          :key="category.id"
          :title="category.name"
        >
          <template #append>
            <v-btn icon size="small" variant="text" @click="openEditDrawer(category)">
              <v-icon>mdi-pencil</v-icon>
            </v-btn>
          </template>
        </v-list-item>
      </v-list>
      <v-alert v-if="categories.length === 0 && !loading" type="info" variant="tonal" class="ma-4">
        {{ $t('common.noData') }}
      </v-alert>
    </div>

    <!-- Category Drawer -->
    <CategoryDrawer
      v-model="drawerOpen"
      :category="selectedCategory"
      @saved="handleSaved"
    />

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mt-4" />
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { categoriesApi } from '@/api/categories'
import type { Category } from '@/api/types'
import CategoryDrawer from '@/components/categories/CategoryDrawer.vue'

const categories = ref<Category[]>([])
const loading = ref(false)
const drawerOpen = ref(false)
const selectedCategory = ref<Category | null>(null)

async function loadCategories() {
  loading.value = true
  try {
    categories.value = await categoriesApi.list()
  } catch (e) {
    console.error('Failed to load categories', e)
  } finally {
    loading.value = false
  }
}

function openAddDrawer() {
  selectedCategory.value = null
  drawerOpen.value = true
}

function openEditDrawer(category: Category) {
  selectedCategory.value = category
  drawerOpen.value = true
}

function handleSaved() {
  loadCategories()
}

onMounted(loadCategories)
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
