import { ref } from 'vue'
import { monthsApi } from '@/api/months'
import type { Month } from '@/api/types'

const months = ref<Month[]>([])
const loaded = ref(false)

async function fetchMonths(): Promise<Month[]> {
  const data = await monthsApi.list()
  months.value = data
  loaded.value = true
  return data
}

export function useMonths() {
  async function getMonths(): Promise<Month[]> {
    if (loaded.value) {
      return months.value
    }
    return fetchMonths()
  }

  async function refreshMonths(): Promise<Month[]> {
    return fetchMonths()
  }

  async function resolveMonthId(monthStr: string): Promise<string> {
    let list = await getMonths()
    let found = list.find((m: Month) => m.month === monthStr)
    if (found) {
      return found.id
    }

    // Refresh and retry before giving up
    list = await refreshMonths()
    found = list.find((m: Month) => m.month === monthStr)
    if (found) {
      return found.id
    }

    throw new Error(`Month not found: ${monthStr}`)
  }

  return {
    months,
    getMonths,
    refreshMonths,
    resolveMonthId,
  }
}
