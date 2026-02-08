<template>
  <v-app>
    <v-main>
      <router-view />
    </v-main>
    <BottomNav />
  </v-app>
</template>

<script setup lang="ts">
import BottomNav from './components/layout/BottomNav.vue'
</script>
