import { createI18n } from 'vue-i18n'
import en from './en.json'
import pl from './pl.json'

function getInitialLocale(): string {
  const saved = localStorage.getItem('language')
  if (saved && ['en', 'pl'].includes(saved)) {
    return saved
  }
  return navigator.language.startsWith('pl') ? 'pl' : 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: 'en',
  messages: { en, pl },
})
