import { ApiError, type ApiErrorResponse } from './types'

const BASE_URL = '/api/v1'

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let errorData: ApiErrorResponse
    try {
      errorData = await response.json()
    } catch {
      throw new ApiError('NETWORK_ERROR')
    }
    throw new ApiError(errorData.error.code, errorData.error.details)
  }

  if (response.status === 204) {
    return undefined as T
  }

  return response.json()
}

export const client = {
  async get<T>(path: string, params?: Record<string, string>): Promise<T> {
    const url = new URL(BASE_URL + path, window.location.origin)
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        url.searchParams.set(key, value)
      })
    }
    const response = await fetch(url.toString(), {
      headers: { 'Accept': 'application/json' },
    })
    return handleResponse<T>(response)
  },

  async post<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(BASE_URL + path, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      body: body ? JSON.stringify(body) : undefined,
    })
    return handleResponse<T>(response)
  },

  async patch<T>(path: string, body: unknown): Promise<T> {
    const response = await fetch(BASE_URL + path, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      body: JSON.stringify(body),
    })
    return handleResponse<T>(response)
  },

  async delete(path: string): Promise<void> {
    const response = await fetch(BASE_URL + path, {
      method: 'DELETE',
      headers: { 'Accept': 'application/json' },
    })
    return handleResponse<void>(response)
  },
}
