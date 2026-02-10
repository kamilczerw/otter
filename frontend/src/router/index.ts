import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: () => {
      const now = new Date()
      const year = now.getFullYear()
      const month = String(now.getMonth() + 1).padStart(2, '0')
      return `/months/${year}-${month}/budget`
    },
  },
  {
    path: '/months',
    name: 'month-list',
    component: () => import('@/views/MonthListView.vue'),
  },
  {
    path: '/months/:month/budget',
    name: 'month-budget',
    component: () => import('@/views/MonthBudgetView.vue'),
  },
  {
    path: '/months/:month/transactions',
    name: 'month-transactions',
    component: () => import('@/views/MonthTransactionsView.vue'),
  },
  {
    path: '/categories',
    name: 'categories',
    component: () => import('@/views/CategoryListView.vue'),
  },
]

// Detect base path dynamically to support HA Ingress subpaths.
// In Ingress, the app is served under e.g. /api/hassio_ingress/<token>/ui/
// We detect the base by finding everything up to and including "/ui/" in the
// current URL path.
function detectBase(): string {
  const path = window.location.pathname
  const uiIndex = path.indexOf('/ui/')
  if (uiIndex !== -1) {
    return path.substring(0, uiIndex + '/ui/'.length)
  }
  // Fallback: if path ends with /ui
  if (path.endsWith('/ui')) {
    return path + '/'
  }
  return '/ui/'
}

export const router = createRouter({
  history: createWebHistory(detectBase()),
  routes,
})
