import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

const customLight = {
  dark: false,
  colors: {
    primary: '#2E7D32',
    secondary: '#1565C0',
    accent: '#FF8F00',
    success: '#2E7D32',
    warning: '#F57F17',
    error: '#C62828',
    background: '#FAFAFA',
    surface: '#FFFFFF',
    'on-primary': '#FFFFFF',
    'on-secondary': '#FFFFFF',
    'on-background': '#212121',
    'on-surface': '#212121',
  },
}

const customDark = {
  dark: true,
  colors: {
    primary: '#66BB6A',
    secondary: '#42A5F5',
    accent: '#FFB300',
    success: '#66BB6A',
    warning: '#FFB300',
    error: '#EF5350',
    background: '#121212',
    surface: '#1E1E1E',
    'on-primary': '#000000',
    'on-secondary': '#000000',
    'on-background': '#E0E0E0',
    'on-surface': '#E0E0E0',
  },
}

const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches

export const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: prefersDark ? 'customDark' : 'customLight',
    themes: {
      customLight,
      customDark,
    },
  },
})
