<template>
  <v-container>
    <h1 class="page-title mb-6">{{ $t('categories.title') }}</h1>

    <!-- Add Category -->
    <div class="mb-6" style="max-width: 400px;">
      <v-text-field
        v-model="newCategoryName"
        :label="$t('categories.newCategory')"
        append-inner-icon="mdi-plus"
        @click:append-inner="addCategory"
        @keyup.enter="addCategory"
        :error-messages="createError"
      />
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
            <v-btn icon size="small" variant="text" @click="startEdit(category)">
              <v-icon>mdi-pencil</v-icon>
            </v-btn>
          </template>
        </v-list-item>
      </v-list>
      <v-alert v-if="categories.length === 0 && !loading" type="info" variant="tonal" class="ma-4">
        {{ $t('common.noData') }}
      </v-alert>
    </div>

    <!-- Edit Dialog -->
    <v-dialog v-model="editDialog" max-width="400">
      <v-card>
        <v-card-title>{{ $t('categories.rename') }}</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="editName"
            :label="$t('categories.name')"
            :error-messages="editError"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn class="btn-secondary-glass" @click="editDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn class="btn-primary-glass" @click="saveEdit">{{ $t('common.save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mt-4" />
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { categoriesApi } from '@/api/categories'
import type { Category } from '@/api/types'
import { ApiError } from '@/api/types'

const { t } = useI18n()

const categories = ref<Category[]>([])
const loading = ref(false)
const newCategoryName = ref('')
const createError = ref('')
const editDialog = ref(false)
const editName = ref('')
const editError = ref('')
const editingCategory = ref<Category | null>(null)

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

async function addCategory() {
  if (!newCategoryName.value.trim()) return
  createError.value = ''
  try {
    await categoriesApi.create({ name: newCategoryName.value.trim() })
    newCategoryName.value = ''
    await loadCategories()
  } catch (e) {
    if (e instanceof ApiError) {
      createError.value = t(`errors.${e.code}`, e.details || {})
    }
  }
}

function startEdit(category: Category) {
  editingCategory.value = category
  editName.value = category.name
  editError.value = ''
  editDialog.value = true
}

async function saveEdit() {
  if (!editingCategory.value || !editName.value.trim()) return
  editError.value = ''
  try {
    await categoriesApi.update(editingCategory.value.id, { name: editName.value.trim() })
    editDialog.value = false
    await loadCategories()
  } catch (e) {
    if (e instanceof ApiError) {
      editError.value = t(`errors.${e.code}`, e.details || {})
    }
  }
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
