import { client } from './client'
import type { Entry, CreateEntryRequest, UpdateEntryRequest } from './types'

export const entriesApi = {
  list: (monthId: string) => client.get<Entry[]>(`/months/${monthId}/entries`),
  create: (monthId: string, data: CreateEntryRequest) => client.post<Entry>(`/months/${monthId}/entries`, data),
  update: (monthId: string, entryId: string, data: UpdateEntryRequest) =>
    client.patch<Entry>(`/months/${monthId}/entries/${entryId}`, data),
  delete: (monthId: string, entryId: string) =>
    client.delete(`/months/${monthId}/entries/${entryId}`),
}
