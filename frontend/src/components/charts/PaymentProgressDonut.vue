<template>
  <div class="donut-wrapper">
    <Doughnut :data="chartData" :options="chartOptions" />
    <div class="donut-center">
      <span class="donut-percent">{{ percentPaid }}%</span>
    </div>
  </div>
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

const percentPaid = computed(() => {
  if (props.totalBudgeted === 0) return 0
  return Math.round((props.totalPaid / props.totalBudgeted) * 100)
})

const chartData = computed(() => {
  const paid = props.totalPaid / 100
  const remaining = Math.max(0, (props.totalBudgeted - props.totalPaid)) / 100
  return {
    labels: ['Paid', 'Remaining'],
    datasets: [{
      data: [paid, remaining],
      backgroundColor: ['#E040A0', 'rgba(255, 255, 255, 0.06)'],
      borderWidth: 0,
      hoverBackgroundColor: ['#E040A0', 'rgba(255, 255, 255, 0.10)'],
    }],
  }
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: true,
  cutout: '70%',
  plugins: {
    legend: {
      position: 'bottom' as const,
      labels: {
        color: '#8890A8',
        font: { family: 'Nunito' },
      },
    },
  },
}
</script>

<style scoped>
.donut-wrapper {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  max-width: 300px;
  max-height: 300px;
  margin: 0 auto;
}

.donut-center {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
}

.donut-percent {
  font-size: 1.375rem;
  font-weight: 700;
  color: #E040A0;
  font-variant-numeric: tabular-nums;
}
</style>
