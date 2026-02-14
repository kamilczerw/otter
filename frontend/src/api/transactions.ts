import { client } from './client'
import type { Transaction, CreateTransactionRequest, UpdateTransactionRequest, PaginatedTransactionsResponse } from './types'

export const transactionsApi = {
  list: (monthId: string) => client.get<Transaction[]>('/transactions', { month: monthId }),
  listByEntry: (entryId: string, limit: number, offset: number) =>
    client.get<PaginatedTransactionsResponse>('/transactions', {
      entry_id: entryId,
      limit: String(limit),
      offset: String(offset),
    }),
  create: (data: CreateTransactionRequest) => client.post<Transaction>('/transactions', data),
  update: (id: string, data: UpdateTransactionRequest) =>
    client.patch<Transaction>(`/transactions/${id}`, data),
  delete: (id: string) => client.delete(`/transactions/${id}`),
}
