<template>
  <div>
    <v-table v-if="entries.length > 0" density="compact">
      <thead>
        <tr>
          <th>{{ $t('entries.category') }}</th>
          <th class="text-right">{{ $t('entries.budgeted') }}</th>
          <th class="text-center">{{ $t('entries.dueDay') }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="entry in entries" :key="entry.id">
          <td>{{ entry.category.name }}</td>
          <td class="text-right">{{ formatCurrency(entry.budgeted) }}</td>
          <td class="text-center">{{ entry.due_day ?? '-' }}</td>
          <td class="text-right">
            <v-btn icon size="x-small" variant="text" @click="startEdit(entry)">
              <v-icon size="small">mdi-pencil</v-icon>
            </v-btn>
            <v-btn icon size="x-small" variant="text" color="error" @click="tryDelete(entry)">
              <v-icon size="small">mdi-delete</v-icon>
            </v-btn>
          </td>
        </tr>
      </tbody>
    </v-table>
    <v-alert v-else-if="!loading" type="info" variant="tonal">
      {{ $t('entries.noEntries') }}
    </v-alert>

    <!-- Edit Dialog -->
    <v-dialog v-model="editDialog" max-width="400">
      <v-card v-if="editingEntry">
        <v-card-title>{{ editingEntry.category.name }}</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="editBudgeted"
            :label="$t('entries.budgeted')"
            type="number"
            variant="outlined"
            density="compact"
          />
          <v-text-field
            v-model="editDueDay"
            :label="$t('entries.dueDay')"
            type="number"
            variant="outlined"
            density="compact"
            :placeholder="'1-31'"
            clearable
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="editDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn color="primary" @click="saveEdit">{{ $t('common.save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Error Snackbar -->
    <v-snackbar v-model="showError" color="error" timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Entry } from '@/api/types'
import { ApiError } from '@/api/types'
import { entriesApi } from '@/api/entries'
import { formatCurrency, parseCurrencyToMinor } from '@/utils/currency'

const { t } = useI18n()

const props = defineProps<{
  entries: Entry[]
  monthId: string
  loading: boolean
}>()

const emit = defineEmits<{
  refresh: []
}>()

const editDialog = ref(false)
const editingEntry = ref<Entry | null>(null)
const editBudgeted = ref('')
const editDueDay = ref<string | null>(null)
const showError = ref(false)
const errorMessage = ref('')

function startEdit(entry: Entry) {
  editingEntry.value = entry
  editBudgeted.value = (entry.budgeted / 100).toFixed(2)
  editDueDay.value = entry.due_day?.toString() ?? null
  editDialog.value = true
}

async function saveEdit() {
  if (!editingEntry.value) return
  try {
    const budgeted = parseCurrencyToMinor(editBudgeted.value)
    const dueDay = editDueDay.value ? parseInt(editDueDay.value) : null
    await entriesApi.update(props.monthId, editingEntry.value.id, {
      budgeted,
      due_day: dueDay,
    })
    editDialog.value = false
    emit('refresh')
  } catch (e) {
    if (e instanceof ApiError) {
      errorMessage.value = t(`errors.${e.code}`, e.details || {})
      showError.value = true
    }
  }
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
</script>
