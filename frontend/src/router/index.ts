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

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
