import { client } from './client'
import type { Category, CreateCategoryRequest, UpdateCategoryRequest } from './types'

export const categoriesApi = {
  list: () => client.get<Category[]>('/categories'),
  create: (data: CreateCategoryRequest) => client.post<Category>('/categories', data),
  update: (id: string, data: UpdateCategoryRequest) => client.patch<Category>(`/categories/${id}`, data),
}
