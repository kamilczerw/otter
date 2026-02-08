import { client } from './client'
import type { Month, CreateMonthRequest } from './types'

export const monthsApi = {
  list: () => client.get<Month[]>('/months'),
  get: (id: string) => client.get<Month>(`/months/${id}`),
  create: (data: CreateMonthRequest) => client.post<Month>('/months', data),
}
