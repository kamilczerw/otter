<template>
  <v-bottom-navigation grow>
    <v-btn :to="budgetRoute" value="budget">
      <v-icon>mdi-chart-bar</v-icon>
      <span>{{ $t('nav.budget') }}</span>
    </v-btn>
    <v-btn to="/months" value="months">
      <v-icon>mdi-calendar</v-icon>
      <span>{{ $t('nav.months') }}</span>
    </v-btn>
    <v-btn to="/categories" value="categories">
      <v-icon>mdi-tag-multiple</v-icon>
      <span>{{ $t('nav.categories') }}</span>
    </v-btn>
  </v-bottom-navigation>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const budgetRoute = computed(() => {
  const now = new Date()
  const year = now.getFullYear()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  return `/months/${year}-${month}/budget`
})
</script>
