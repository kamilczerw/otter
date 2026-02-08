<template>
  <Bar :data="chartData" :options="chartOptions" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js'
import type { CategoryBudgetSummary } from '@/api/types'
import { formatCurrency } from '@/utils/currency'

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = defineProps<{
  categories: CategoryBudgetSummary[]
}>()

const chartData = computed(() => ({
  labels: props.categories.map(c => c.category.name),
  datasets: [
    {
      label: 'Budgeted',
      data: props.categories.map(c => c.budgeted / 100),
      backgroundColor: '#2E7D32',
    },
    {
      label: 'Paid',
      data: props.categories.map(c => c.paid / 100),
      backgroundColor: props.categories.map(c =>
        c.status === 'overspent' ? '#C62828' : '#1565C0'
      ),
    },
  ],
}))

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { position: 'top' as const },
  },
  scales: {
    y: {
      beginAtZero: true,
      ticks: {
        callback: (value: string | number) => formatCurrency(Number(value) * 100),
      },
    },
  },
}))
</script>

<style scoped>
div { height: 300px; }
</style>
