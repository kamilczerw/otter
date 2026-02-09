<template>
  <div ref="containerRef">
    <v-table v-if="entries.length > 0 || isAddingNew" density="compact">
      <thead>
        <tr>
          <th>{{ $t('entries.category') }}</th>
          <th class="text-right">{{ $t('entries.budgeted') }}</th>
          <th class="text-center">{{ $t('entries.dueDay') }}</th>
          <th class="entry-actions-col"></th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="entry in entries"
          :key="entry.id"
          :class="{ 'cursor-pointer': editingEntryId !== entry.id }"
          @click="onRowClick(entry)"
        >
          <!-- Display mode -->
          <template v-if="editingEntryId !== entry.id">
            <td>{{ entry.category.name }}</td>
            <td class="text-right">{{ formatCurrency(entry.budgeted) }}</td>
            <td class="text-center">{{ entry.due_day ?? '-' }}</td>
            <td class="text-right entry-actions-col">
              <v-btn icon size="x-small" variant="text" color="error" @click.stop="tryDelete(entry)">
                <v-icon size="small">mdi-delete</v-icon>
              </v-btn>
            </td>
          </template>

          <!-- Edit mode -->
          <template v-else>
            <td @click.stop>{{ entry.category.name }}</td>
            <td @click.stop>
              <v-text-field
                v-model="editForm.budgeted"
                type="number"
                density="compact"
                variant="underlined"
                hide-details
                single-line
                class="inline-input inline-input--right"
              />
            </td>
            <td @click.stop>
              <v-text-field
                v-model="editForm.dueDay"
                type="number"
                density="compact"
                variant="underlined"
                hide-details
                single-line
                clearable
                placeholder="1-31"
                class="inline-input inline-input--center"
              />
            </td>
            <td class="entry-actions-col" @click.stop>
              <v-btn icon size="x-small" variant="text" color="success" :loading="saving" @click="confirmEdit">
                <v-icon size="small">mdi-check</v-icon>
              </v-btn>
              <v-btn icon size="x-small" variant="text" @click="cancelEdit">
                <v-icon size="small">mdi-close</v-icon>
              </v-btn>
            </td>
          </template>
        </tr>

        <!-- New entry row -->
        <tr v-if="isAddingNew" @click.stop>
          <td>
            <CategoryAutocomplete
              v-model="newForm.categoryId"
              :autofocus="true"
              hide-label
              @error="onError"
            />
          </td>
          <td>
            <v-text-field
              v-model="newForm.budgeted"
              type="number"
              density="compact"
              variant="underlined"
              hide-details
              single-line
              class="inline-input inline-input--right"
            />
          </td>
          <td>
            <v-text-field
              v-model="newForm.dueDay"
              type="number"
              density="compact"
              variant="underlined"
              hide-details
              single-line
              clearable
              placeholder="1-31"
              class="inline-input inline-input--center"
            />
          </td>
          <td class="entry-actions-col">
            <v-btn icon size="x-small" variant="text" color="success" :loading="saving" @click="confirmAdd">
              <v-icon size="small">mdi-check</v-icon>
            </v-btn>
            <v-btn icon size="x-small" variant="text" @click="cancelAdd">
              <v-icon size="small">mdi-close</v-icon>
            </v-btn>
          </td>
        </tr>
      </tbody>
    </v-table>

    <v-alert v-if="entries.length === 0 && !isAddingNew && !loading" type="info" variant="tonal">
      {{ $t('entries.noEntries') }}
    </v-alert>

    <div class="mt-3">
      <v-btn
        v-if="!isAddingNew"
        variant="text"
        color="primary"
        size="small"
        prepend-icon="mdi-plus"
        @click="startAdd"
      >
        {{ $t('entries.addEntry') }}
      </v-btn>
    </div>

    <!-- Error Snackbar -->
    <v-snackbar v-model="showError" color="error" timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Entry } from '@/api/types'
import { ApiError } from '@/api/types'
import { entriesApi } from '@/api/entries'
import { formatCurrency, parseCurrencyToMinor } from '@/utils/currency'
import CategoryAutocomplete from './CategoryAutocomplete.vue'

const { t } = useI18n()

const props = defineProps<{
  entries: Entry[]
  monthId: string
  loading: boolean
}>()

const emit = defineEmits<{
  refresh: []
}>()

// Edit state
const editingEntryId = ref<string | null>(null)
const editingEntry = ref<Entry | null>(null)
const editForm = ref({ budgeted: '', dueDay: null as string | null })
const editOriginal = ref({ budgeted: '', dueDay: null as string | null })

// Add state
const isAddingNew = ref(false)
const newForm = ref({ categoryId: '', budgeted: '', dueDay: null as string | null })

// Shared state
const saving = ref(false)
const showError = ref(false)
const errorMessage = ref('')
const containerRef = ref<HTMLElement | null>(null)

function toFormBudgeted(minorUnits: number): string {
  return (minorUnits / 100).toFixed(2)
}

function hasEditChanges(): boolean {
  return editForm.value.budgeted !== editOriginal.value.budgeted
    || (editForm.value.dueDay ?? '') !== (editOriginal.value.dueDay ?? '')
}

function onRowClick(entry: Entry) {
  // Already editing this row
  if (editingEntryId.value === entry.id) return
  // Currently editing another row with changes — block
  if (editingEntryId.value && hasEditChanges()) return
  // Currently adding — block
  if (isAddingNew.value) return

  // Close current edit (no changes)
  if (editingEntryId.value) {
    cancelEdit()
  }

  startEdit(entry)
}

function startEdit(entry: Entry) {
  editingEntryId.value = entry.id
  editingEntry.value = entry
  const budgeted = toFormBudgeted(entry.budgeted)
  const dueDay = entry.due_day?.toString() ?? null
  editForm.value = { budgeted, dueDay }
  editOriginal.value = { budgeted, dueDay }
}

function cancelEdit() {
  editingEntryId.value = null
  editingEntry.value = null
}

async function confirmEdit() {
  if (!editingEntry.value) return
  saving.value = true
  try {
    const budgeted = parseCurrencyToMinor(editForm.value.budgeted)
    const dueDay = editForm.value.dueDay ? parseInt(editForm.value.dueDay) : null
    const payload: { budgeted?: number; due_day?: number | null } = {}

    if (budgeted !== editingEntry.value.budgeted) {
      payload.budgeted = budgeted
    }
    if (dueDay !== editingEntry.value.due_day) {
      payload.due_day = dueDay
    }

    if (Object.keys(payload).length > 0) {
      await entriesApi.update(props.monthId, editingEntry.value.id, payload)
    }
    cancelEdit()
    emit('refresh')
  } catch (e) {
    if (e instanceof ApiError) {
      errorMessage.value = t(`errors.${e.code}`, e.details || {})
      showError.value = true
    }
  } finally {
    saving.value = false
  }
}

function startAdd() {
  // Close current edit if no changes
  if (editingEntryId.value && !hasEditChanges()) {
    cancelEdit()
  }
  if (editingEntryId.value) return
  isAddingNew.value = true
  newForm.value = { categoryId: '', budgeted: '', dueDay: null }
}

function cancelAdd() {
  isAddingNew.value = false
  newForm.value = { categoryId: '', budgeted: '', dueDay: null }
}

async function confirmAdd() {
  if (!newForm.value.categoryId || !newForm.value.budgeted) return
  saving.value = true
  try {
    await entriesApi.create(props.monthId, {
      category_id: newForm.value.categoryId,
      budgeted: parseCurrencyToMinor(newForm.value.budgeted),
      due_day: newForm.value.dueDay ? parseInt(newForm.value.dueDay) : undefined,
    })
    cancelAdd()
    emit('refresh')
  } catch (e) {
    if (e instanceof ApiError) {
      errorMessage.value = t(`errors.${e.code}`, e.details || {})
      showError.value = true
    }
  } finally {
    saving.value = false
  }
}

function onError(msg: string) {
  errorMessage.value = msg
  showError.value = true
}

async function tryDelete(entry: Entry) {
  try {
    await entriesApi.delete(props.monthId, entry.id)
    emit('refresh')
  } catch (e) {
    if (e instanceof ApiError) {
      errorMessage.value = t(`errors.${e.code}`, e.details || {})
      showError.value = true
    }
  }
}

// Click-outside handler for dismissing edit mode when no changes
function onDocumentClick(e: MouseEvent) {
  if (!editingEntryId.value) return
  if (!containerRef.value) return

  // Check if click is inside the table (any row)
  const table = containerRef.value.querySelector('table')
  if (table && table.contains(e.target as Node)) return

  // Also ignore clicks inside Vuetify overlay containers (dropdowns, menus)
  const overlay = (e.target as HTMLElement)?.closest?.('.v-overlay__content')
  if (overlay) return

  // Clicked outside — dismiss if no changes
  if (!hasEditChanges()) {
    cancelEdit()
  }
}

onMounted(() => {
  document.addEventListener('click', onDocumentClick, true)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick, true)
})
</script>

<style scoped>
.entry-actions-col {
  width: 80px;
  white-space: nowrap;
}

.inline-input {
  max-width: 120px;
}

.inline-input--right :deep(input) {
  text-align: right;
}

.inline-input--center :deep(input) {
  text-align: center;
}

.cursor-pointer {
  cursor: pointer;
}

.mt-3 {
  margin-top: 12px;
}
</style>
