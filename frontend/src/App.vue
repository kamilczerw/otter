<template>
  <v-app>
    <div class="language-switcher-container">
      <LanguageSwitcher />
    </div>
    <v-main>
      <router-view />
    </v-main>
    <BottomNav />
  </v-app>
</template>

<script setup lang="ts">
import BottomNav from './components/layout/BottomNav.vue'
import LanguageSwitcher from './components/layout/LanguageSwitcher.vue'
</script>

<style>
:root {
  --bg-deep: #0B0D1A;
  --bg-card: rgba(18, 22, 48, 0.55);
  --bg-card-hover: rgba(24, 28, 58, 0.65);
  --bg-stat: rgba(255, 255, 255, 0.04);
  --bg-nav: rgba(18, 22, 48, 0.60);
  --magenta: #E040A0;
  --magenta-glass: rgba(224, 64, 160, 0.70);
  --magenta-border: rgba(224, 64, 160, 0.25);
  --magenta-glow: rgba(224, 64, 160, 0.15);
  --text-primary: #E8EAF0;
  --text-secondary: #8890A8;
  --color-success: #5AD8A0;
  --color-danger: #FF5070;
  --color-warning: #FFB74D;
  --border-glass: rgba(255, 255, 255, 0.08);
  --border-row: rgba(255, 255, 255, 0.05);
  --glass-highlight: rgba(255, 255, 255, 0.04);
}

html, body {
  margin: 0;
  padding: 0;
  background: var(--bg-deep);
  font-family: 'Nunito', sans-serif;
  color: var(--text-primary);
  min-height: 100vh;
  overflow-x: hidden;
}

/* Night sky background */
body::before {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
  background:
    /* Stars - small */
    radial-gradient(1px 1px at 10% 15%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 25% 8%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1.5px 1.5px at 40% 22%, rgba(255,255,255,0.8), transparent),
    radial-gradient(1px 1px at 55% 5%, rgba(255,255,255,0.6), transparent),
    radial-gradient(2px 2px at 70% 18%, rgba(255,255,255,0.9), transparent),
    radial-gradient(1px 1px at 85% 12%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1px 1px at 15% 35%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1.5px 1.5px at 30% 42%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 50% 38%, rgba(255,255,255,0.5), transparent),
    radial-gradient(2px 2px at 65% 30%, rgba(255,255,255,0.8), transparent),
    radial-gradient(1px 1px at 80% 45%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1px 1px at 92% 35%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1.5px 1.5px at 8% 55%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 22% 62%, rgba(255,255,255,0.5), transparent),
    radial-gradient(2px 2px at 38% 58%, rgba(255,255,255,0.8), transparent),
    radial-gradient(1px 1px at 48% 52%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1px 1px at 62% 65%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1.5px 1.5px at 78% 55%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 88% 60%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1px 1px at 5% 75%, rgba(255,255,255,0.5), transparent),
    radial-gradient(2px 2px at 18% 80%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 35% 72%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1.5px 1.5px at 52% 78%, rgba(255,255,255,0.8), transparent),
    radial-gradient(1px 1px at 68% 82%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1px 1px at 82% 75%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 95% 85%, rgba(255,255,255,0.5), transparent),
    radial-gradient(1.5px 1.5px at 12% 92%, rgba(255,255,255,0.6), transparent),
    radial-gradient(1px 1px at 42% 88%, rgba(255,255,255,0.5), transparent),
    radial-gradient(2px 2px at 72% 95%, rgba(255,255,255,0.7), transparent),
    radial-gradient(1px 1px at 58% 92%, rgba(255,255,255,0.6), transparent),
    /* Nebula glow */
    radial-gradient(ellipse 600px 400px at 20% 30%, rgba(224, 64, 160, 0.06), transparent),
    radial-gradient(ellipse 500px 350px at 75% 60%, rgba(50, 70, 200, 0.05), transparent),
    radial-gradient(ellipse 400px 300px at 50% 80%, rgba(224, 64, 160, 0.04), transparent);
}

/* Vuetify overrides for cosmic theme */
.v-application {
  font-family: 'Nunito', sans-serif !important;
  background: transparent !important;
}

.v-main {
  padding-bottom: 72px !important;
}

/* Glass card styling */
.glass-card {
  background: var(--bg-card) !important;
  border: 1px solid var(--border-glass) !important;
  backdrop-filter: blur(16px) !important;
  -webkit-backdrop-filter: blur(16px) !important;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2), inset 0 1px 0 var(--glass-highlight) !important;
  border-radius: 14px !important;
}

.glass-card:hover {
  background: var(--bg-card-hover) !important;
}

/* Stat blocks */
.stat-block {
  background: var(--bg-stat);
  border-radius: 10px;
  padding: 10px;
  text-align: center;
}

.stat-block .stat-label {
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 1.5px;
}

.stat-block .stat-value {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

/* Glass primary button */
.btn-primary-glass {
  background: var(--magenta-glass) !important;
  backdrop-filter: blur(8px) !important;
  -webkit-backdrop-filter: blur(8px) !important;
  border: 1px solid var(--magenta-border) !important;
  box-shadow: 0 2px 12px var(--magenta-glow) !important;
  color: #fff !important;
}

.btn-primary-glass:hover {
  opacity: 0.85;
}

/* Glass secondary button */
.btn-secondary-glass {
  background: rgba(255, 255, 255, 0.08) !important;
  backdrop-filter: blur(8px) !important;
  -webkit-backdrop-filter: blur(8px) !important;
  border: 1px solid rgba(255, 255, 255, 0.10) !important;
  color: var(--text-primary) !important;
}

/* Section labels */
.section-label {
  font-size: 0.6875rem;
  font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 1.5px;
}

/* Text utilities */
.text-magenta { color: var(--magenta) !important; }
.text-cosmic-primary { color: var(--text-primary) !important; }
.text-cosmic-secondary { color: var(--text-secondary) !important; }
.text-success-cosmic { color: var(--color-success) !important; }
.text-danger-cosmic { color: var(--color-danger) !important; }

/* Focus ring */
*:focus-visible {
  outline: 2px solid rgba(224, 64, 160, 0.5) !important;
  outline-offset: 2px !important;
}

/* Glass table */
.v-table {
  background: transparent !important;
  color: var(--text-primary) !important;
}

.v-table thead th {
  color: var(--text-secondary) !important;
  font-size: 0.6875rem !important;
  font-weight: 700 !important;
  text-transform: uppercase !important;
  letter-spacing: 1.5px !important;
  border-bottom: 1px solid var(--border-row) !important;
}

.v-table tbody td {
  color: var(--text-primary) !important;
  font-size: 0.875rem !important;
  border-bottom: 1px solid var(--border-row) !important;
}

/* Glass list items */
.v-list {
  background: transparent !important;
}

.v-list-item {
  border-bottom: 1px solid var(--border-row);
}

.v-list-item:hover {
  background: var(--bg-card-hover) !important;
}

.v-list-item-title {
  color: var(--text-primary) !important;
}

/* Text field cosmic styling */
.v-field--variant-outlined .v-field__outline {
  color: var(--border-glass) !important;
}

.v-field--focused .v-field__outline {
  color: var(--magenta) !important;
}

.v-field__input {
  color: var(--text-primary) !important;
}

.v-label {
  color: var(--text-secondary) !important;
}

/* Tabs cosmic styling */
.v-tabs {
  background: transparent !important;
}

.v-tab {
  color: var(--text-secondary) !important;
  font-family: 'Nunito', sans-serif !important;
  font-weight: 600 !important;
  text-transform: none !important;
}

.v-tab--selected {
  color: var(--magenta) !important;
}

.v-tab-slider {
  color: var(--magenta) !important;
}

/* Alert styling */
.v-alert {
  border-radius: 10px !important;
}

/* Dialog/card glass */
.v-dialog .v-card,
.v-bottom-sheet .v-card {
  background: rgba(14, 18, 40, 0.90) !important;
  backdrop-filter: blur(24px) !important;
  -webkit-backdrop-filter: blur(24px) !important;
  border: 1px solid var(--border-glass) !important;
  color: var(--text-primary) !important;
}

.v-card-title {
  color: var(--text-primary) !important;
  font-family: 'Nunito', sans-serif !important;
  font-weight: 700 !important;
}

.v-card-text {
  color: var(--text-primary) !important;
}

/* Bottom sheet glass treatment */
.v-bottom-sheet .v-card {
  border-radius: 20px 20px 0 0 !important;
  border-top: 1px solid var(--border-glass) !important;
}

/* Progress linear */
.v-progress-linear {
  border-radius: 2px !important;
}

/* Snackbar */
.v-snackbar__wrapper {
  border-radius: 10px !important;
}

/* Container spacing */
.v-container {
  padding: 16px !important;
}

/* Language switcher positioning */
.language-switcher-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 999;
}
</style>
