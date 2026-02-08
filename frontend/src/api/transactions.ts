import { client } from './client'
import type { Transaction, CreateTransactionRequest, UpdateTransactionRequest } from './types'

export const transactionsApi = {
  list: (monthId: string) => client.get<Transaction[]>('/transactions', { month: monthId }),
  create: (data: CreateTransactionRequest) => client.post<Transaction>('/transactions', data),
  update: (id: string, data: UpdateTransactionRequest) =>
    client.patch<Transaction>(`/transactions/${id}`, data),
  delete: (id: string) => client.delete(`/transactions/${id}`),
}
