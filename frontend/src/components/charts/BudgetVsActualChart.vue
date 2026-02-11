<template>
  <div class="chart-container">
    <Bar :data="chartData" :options="chartOptions" />
  </div>
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
import { getCategoryDisplayName } from '@/utils/category'

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = defineProps<{
  categories: CategoryBudgetSummary[]
}>()

const chartData = computed(() => ({
  labels: props.categories.map(c => getCategoryDisplayName(c.category)),
  datasets: [
    {
      label: 'Budgeted',
      data: props.categories.map(c => c.budgeted / 100),
      backgroundColor: 'rgba(255, 255, 255, 0.10)',
      borderRadius: 4,
    },
    {
      label: 'Paid',
      data: props.categories.map(c => c.paid / 100),
      backgroundColor: props.categories.map(c =>
        c.status === 'overspent' ? '#FF5070' : '#E040A0'
      ),
      borderRadius: 4,
    },
  ],
}))

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'top' as const,
      labels: {
        color: '#8890A8',
        font: { family: 'Nunito' },
      },
    },
  },
  scales: {
    x: {
      ticks: {
        color: '#8890A8',
        font: { family: 'Nunito', size: 11 },
      },
      grid: { color: 'rgba(255, 255, 255, 0.05)' },
    },
    y: {
      beginAtZero: true,
      ticks: {
        color: '#8890A8',
        font: { family: 'Nunito', size: 11 },
        callback: (value: string | number) => formatCurrency(Number(value) * 100),
      },
      grid: { color: 'rgba(255, 255, 255, 0.05)' },
    },
  },
}))
</script>

<style scoped>
.chart-container {
  position: relative;
  height: 300px;
}
</style>
