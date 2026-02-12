<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    fullscreen
    transition="dialog-bottom-transition"
    class="settings-drawer"
  >
    <div class="settings-fullscreen">
      <div class="settings-header">
        <h2 class="settings-title">{{ $t('settings.title') }}</h2>
        <v-btn icon variant="text" size="small" @click="$emit('update:modelValue', false)">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </div>

      <div class="settings-content">
        <!-- User Settings -->
        <div class="settings-section">
          <div class="section-label">{{ $t('settings.userSettings') }}</div>
          <div class="settings-placeholder">
            <span class="text-cosmic-secondary">{{ $t('settings.comingSoon') }}</span>
          </div>
        </div>

        <!-- UI Settings -->
        <div class="settings-section">
          <div class="section-label">{{ $t('settings.uiSettings') }}</div>
          <div class="settings-item" @click="showLanguageMenu = !showLanguageMenu">
            <div class="settings-item-left">
              <span class="settings-item-icon">{{ currentFlag }}</span>
              <div>
                <div class="settings-item-label">{{ $t('language.label') }}</div>
                <div class="settings-item-value">{{ currentLanguageName }}</div>
              </div>
            </div>
            <v-icon size="small" class="text-cosmic-secondary">
              {{ showLanguageMenu ? 'mdi-chevron-up' : 'mdi-chevron-down' }}
            </v-icon>
          </div>
          <div v-if="showLanguageMenu" class="language-options">
            <div
              v-for="lang in availableLanguages"
              :key="lang.code"
              class="language-option"
              :class="{ active: currentLocale === lang.code }"
              @click="changeLanguage(lang.code)"
            >
              <span class="language-option-flag">{{ lang.flag }}</span>
              <span class="language-option-name">{{ lang.name }}</span>
              <v-icon v-if="currentLocale === lang.code" size="small" color="primary" class="language-option-check">
                mdi-check
              </v-icon>
            </div>
          </div>

          <div class="settings-item mt-2" @click="showBudgetBarSizeMenu = !showBudgetBarSizeMenu">
            <div class="settings-item-left">
              <v-icon class="settings-item-icon" size="large">
                {{ budgetBarSize === 'compact' ? 'mdi-view-compact' : 'mdi-view-comfortable' }}
              </v-icon>
              <div>
                <div class="settings-item-label">{{ $t('budgetBarSize.label') }}</div>
                <div class="settings-item-value">{{ currentBudgetBarSizeName }}</div>
              </div>
            </div>
            <v-icon size="small" class="text-cosmic-secondary">
              {{ showBudgetBarSizeMenu ? 'mdi-chevron-up' : 'mdi-chevron-down' }}
            </v-icon>
          </div>
          <div v-if="showBudgetBarSizeMenu" class="language-options">
            <div
              v-for="size in availableBudgetBarSizes"
              :key="size.value"
              class="language-option"
              :class="{ active: budgetBarSize === size.value }"
              @click="changeBudgetBarSize(size.value)"
            >
              <v-icon class="language-option-flag">{{ size.icon }}</v-icon>
              <span class="language-option-name">{{ size.name }}</span>
              <v-icon v-if="budgetBarSize === size.value" size="small" color="primary" class="language-option-check">
                mdi-check
              </v-icon>
            </div>
          </div>
        </div>

        <!-- Data -->
        <div class="settings-section">
          <div class="section-label">{{ $t('settings.data') }}</div>
          <div class="settings-placeholder">
            <span class="text-cosmic-secondary">{{ $t('settings.comingSoon') }}</span>
          </div>
        </div>
      </div>
    </div>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUiPreferences, type BudgetBarSize } from '@/composables/useUiPreferences'

defineProps<{
  modelValue: boolean
}>()

defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { locale, t } = useI18n()
const { budgetBarSize } = useUiPreferences()

const showLanguageMenu = ref(false)
const showBudgetBarSizeMenu = ref(false)

const currentLocale = computed(() => locale.value)

const languageFlags: Record<string, string> = {
  en: '\u{1F1EC}\u{1F1E7}',
  pl: '\u{1F1F5}\u{1F1F1}',
}

const currentFlag = computed(() => languageFlags[currentLocale.value] || '\u{1F310}')

const currentLanguageName = computed(() => {
  const lang = availableLanguages.value.find(l => l.code === currentLocale.value)
  return lang?.name || currentLocale.value
})

const currentBudgetBarSizeName = computed(() => {
  return t(`budgetBarSize.${budgetBarSize.value}`)
})

const availableLanguages = computed(() => [
  { code: 'en', name: t('language.en'), flag: '\u{1F1EC}\u{1F1E7}' },
  { code: 'pl', name: t('language.pl'), flag: '\u{1F1F5}\u{1F1F1}' },
])

const availableBudgetBarSizes = computed(() => [
  { value: 'compact' as BudgetBarSize, name: t('budgetBarSize.compact'), icon: 'mdi-view-compact' },
  { value: 'spacious' as BudgetBarSize, name: t('budgetBarSize.spacious'), icon: 'mdi-view-comfortable' },
])

function changeLanguage(newLocale: string) {
  locale.value = newLocale
  localStorage.setItem('language', newLocale)
}

function changeBudgetBarSize(size: BudgetBarSize) {
  budgetBarSize.value = size
}
</script>

<style scoped>
.settings-fullscreen {
  background: var(--bg-deep);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-glass);
}

.settings-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--text-primary);
}

.settings-content {
  flex: 1;
  padding: 8px 16px;
  overflow-y: auto;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section .section-label {
  margin-bottom: 8px;
  padding: 0 4px;
}

.settings-placeholder {
  background: var(--bg-stat);
  border-radius: 10px;
  padding: 16px;
  text-align: center;
  font-size: 0.875rem;
}

.settings-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-stat);
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.settings-item:hover {
  background: var(--bg-card-hover);
}

.settings-item-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.settings-item-icon {
  font-size: 1.5rem;
  line-height: 1;
}

.settings-item-label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
}

.settings-item-value {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.language-options {
  margin-top: 4px;
  border-radius: 10px;
  overflow: hidden;
  background: var(--bg-stat);
}

.language-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.language-option:hover {
  background: var(--bg-card-hover);
}

.language-option.active {
  background: rgba(224, 64, 160, 0.10);
}

.language-option-flag {
  font-size: 1.25rem;
  line-height: 1;
}

.mt-2 {
  margin-top: 8px;
}

.language-option-name {
  flex: 1;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.language-option.active .language-option-name {
  color: var(--magenta);
  font-weight: 600;
}

.language-option-check {
  color: var(--magenta) !important;
}
</style>
