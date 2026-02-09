import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

const cosmicDark = {
  dark: true,
  colors: {
    primary: '#E040A0',
    secondary: '#8890A8',
    accent: '#E040A0',
    success: '#5AD8A0',
    warning: '#FFB74D',
    error: '#FF5070',
    background: '#0B0D1A',
    surface: '#121630',
    'on-primary': '#FFFFFF',
    'on-secondary': '#E8EAF0',
    'on-background': '#E8EAF0',
    'on-surface': '#E8EAF0',
    'surface-variant': '#121630',
  },
}

export const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'cosmicDark',
    themes: {
      cosmicDark,
    },
  },
  defaults: {
    VCard: {
      rounded: 'lg',
      elevation: 0,
    },
    VBtn: {
      rounded: 'lg',
    },
    VTextField: {
      variant: 'outlined',
      density: 'compact',
    },
    VSelect: {
      variant: 'outlined',
      density: 'compact',
    },
    VAutocomplete: {
      variant: 'outlined',
      density: 'compact',
    },
  },
})
