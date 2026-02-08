<template>
  <Doughnut :data="chartData" :options="chartOptions" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Doughnut } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js'

ChartJS.register(ArcElement, Tooltip, Legend)

const props = defineProps<{
  totalBudgeted: number
  totalPaid: number
}>()

const chartData = computed(() => {
  const paid = props.totalPaid / 100
  const remaining = Math.max(0, (props.totalBudgeted - props.totalPaid)) / 100
  return {
    labels: ['Paid', 'Remaining'],
    datasets: [{
      data: [paid, remaining],
      backgroundColor: ['#2E7D32', '#E0E0E0'],
      borderWidth: 0,
    }],
  }
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: true,
  cutout: '70%',
  plugins: {
    legend: { position: 'bottom' as const },
  },
}
</script>
