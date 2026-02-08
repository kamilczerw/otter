<template>
  <v-app>
    <v-app-bar density="compact" flat>
      <v-spacer />
      <v-btn :icon="isDark ? 'mdi-weather-sunny' : 'mdi-weather-night'" @click="toggleTheme" />
    </v-app-bar>
    <v-main>
      <router-view />
    </v-main>
    <BottomNav />
  </v-app>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useTheme } from 'vuetify'
import BottomNav from './components/layout/BottomNav.vue'

const theme = useTheme()

const isDark = computed(() => theme.global.current.value.dark)

function toggleTheme() {
  theme.global.name.value = isDark.value ? 'customLight' : 'customDark'
}
</script>
