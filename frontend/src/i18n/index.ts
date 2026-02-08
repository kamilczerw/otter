import { createI18n } from 'vue-i18n'
import en from './en.json'
import pl from './pl.json'

export const i18n = createI18n({
  legacy: false,
  locale: navigator.language.startsWith('pl') ? 'pl' : 'en',
  fallbackLocale: 'en',
  messages: { en, pl },
})
