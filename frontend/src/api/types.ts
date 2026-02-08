export interface Category {
  id: string
  name: string
  created_at: string
  updated_at: string
}

export interface Month {
  id: string
  month: string  // "YYYY-MM"
  created_at: string
  updated_at: string
}

export interface CategorySummary {
  id: string
  name: string
}

export interface Entry {
  id: string
  category: CategorySummary
  budgeted: number
  due_day: number | null
  created_at: string
  updated_at: string
}

export interface Transaction {
  id: string
  entry_id: string
  amount: number
  date: string  // "YYYY-MM-DD"
  created_at: string
  updated_at: string
}

export interface MonthSummary {
  month: string
  total_budgeted: number
  total_paid: number
  remaining: number
  categories: CategoryBudgetSummary[]
}

export interface CategoryBudgetSummary {
  category: CategorySummary
  budgeted: number
  paid: number
  remaining: number
  status: 'unpaid' | 'underspent' | 'on_budget' | 'overspent'
}

// Request types
export interface CreateCategoryRequest {
  name: string
}

export interface UpdateCategoryRequest {
  name: string
}

export interface CreateMonthRequest {
  month: string
}

export interface CreateEntryRequest {
  category_id: string
  budgeted: number
  due_day?: number | null
}

export interface UpdateEntryRequest {
  budgeted?: number
  due_day?: number | null
}

export interface CreateTransactionRequest {
  entry_id: string
  amount: number
  date: string
}

export interface UpdateTransactionRequest {
  entry_id?: string
  amount?: number
  date?: string
}

// Error types
export interface ApiErrorResponse {
  error: {
    code: string
    details?: Record<string, unknown>
  }
}

export class ApiError extends Error {
  code: string
  details?: Record<string, unknown>

  constructor(code: string, details?: Record<string, unknown>) {
    super(code)
    this.code = code
    this.details = details
  }
}
