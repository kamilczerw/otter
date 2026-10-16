import { defineConfig, mergeConfig } from 'vite'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'jsdom',
      // Vuetify's component barrel pulls in per-component .css imports that only
      // Vite's pipeline (not vitest's default SSR externalization) knows how to handle.
      server: { deps: { inline: ['vuetify'] } },
    },
  }),
)
