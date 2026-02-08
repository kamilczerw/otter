import { client } from './client'
import type { MonthSummary } from './types'

export const summaryApi = {
  get: (monthId: string) => client.get<MonthSummary>(`/months/${monthId}/summary`),
}
