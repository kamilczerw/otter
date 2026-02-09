<template>
  <v-menu offset-y>
    <template v-slot:activator="{ props }">
      <v-btn
        v-bind="props"
        icon
        size="small"
        class="language-btn"
        :aria-label="$t('language.label')"
      >
        <span class="flag-icon">{{ currentFlag }}</span>
      </v-btn>
    </template>
    <v-list class="language-menu" density="compact">
      <v-list-item
        v-for="lang in availableLanguages"
        :key="lang.code"
        :class="{ active: currentLocale === lang.code }"
        @click="changeLanguage(lang.code)"
      >
        <template v-slot:prepend>
          <span class="flag-icon-menu">{{ lang.flag }}</span>
        </template>
        <v-list-item-title>{{ lang.name }}</v-list-item-title>
        <template v-slot:append v-if="currentLocale === lang.code">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </template>
      </v-list-item>
    </v-list>
  </v-menu>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { locale, t } = useI18n()

const currentLocale = computed(() => locale.value)

const languageFlags: Record<string, string> = {
  en: '🇬🇧',
  pl: '🇵🇱',
}

const currentFlag = computed(() => languageFlags[currentLocale.value] || '🌐')

const availableLanguages = computed(() => [
  { code: 'en', name: t('language.en'), flag: '🇬🇧' },
  { code: 'pl', name: t('language.pl'), flag: '🇵🇱' },
])

function changeLanguage(newLocale: string) {
  locale.value = newLocale
  localStorage.setItem('language', newLocale)
}
</script>

<style scoped>
.language-btn {
  background: rgba(255, 255, 255, 0.06) !important;
  border: 1px solid rgba(255, 255, 255, 0.10) !important;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  transition: all 0.2s ease;
}

.language-btn:hover {
  background: rgba(255, 255, 255, 0.10) !important;
  border-color: var(--magenta) !important;
}

.flag-icon {
  font-size: 1.25rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.flag-icon-menu {
  font-size: 1.25rem;
  line-height: 1;
  margin-right: 8px;
}

.language-menu {
  background: rgba(14, 18, 40, 0.95) !important;
  backdrop-filter: blur(24px) !important;
  -webkit-backdrop-filter: blur(24px) !important;
  border: 1px solid var(--border-glass) !important;
  border-radius: 10px !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3) !important;
  padding: 4px !important;
}

.v-list-item {
  border-radius: 6px !important;
  margin: 2px 0 !important;
  min-height: 36px !important;
}

.v-list-item:hover {
  background: var(--bg-card-hover) !important;
}

.v-list-item.active {
  background: rgba(224, 64, 160, 0.15) !important;
  color: var(--magenta) !important;
}

.v-list-item.active .v-list-item-title {
  color: var(--magenta) !important;
  font-weight: 600;
}

.v-list-item.active svg {
  color: var(--magenta);
}
</style>
