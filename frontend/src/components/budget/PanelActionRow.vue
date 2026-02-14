<template>
  <div class="panel-action-row">
    <v-btn
      variant="text"
      size="small"
      color="grey"
      prepend-icon="mdi-pencil"
      @click="$emit('edit-budget')"
    >
      {{ $t('budget.panel.editBudget') }}
    </v-btn>
    <v-btn
      variant="text"
      size="small"
      class="add-btn"
      prepend-icon="mdi-plus"
      @click="$emit('add-transaction')"
    >
      {{ $t('budget.panel.addTransaction') }}
    </v-btn>
  </div>
</template>

<script setup lang="ts">
defineEmits<{
  'edit-budget': []
  'add-transaction': []
}>()
</script>

<style scoped>
.panel-action-row {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 4px 0;
}

.add-btn {
  color: #3ddc84 !important;
}
</style>
