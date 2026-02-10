import { ApiError, type ApiErrorResponse } from './types'

// Compute the API base URL relative to the current page.
// In standard deployment: /ui/ -> API at /api/v1
// In HA Ingress: /api/hassio_ingress/<token>/ui/ -> API at /api/hassio_ingress/<token>/api/v1
// We detect the ingress prefix by finding everything before "/ui/" in the pathname.
function getBaseUrl(): string {
  const path = window.location.pathname
  const uiIndex = path.indexOf('/ui/')
  if (uiIndex !== -1) {
    const prefix = path.substring(0, uiIndex)
    return `${prefix}/api/v1`
  }
  if (path.endsWith('/ui')) {
    const prefix = path.substring(0, path.length - '/ui'.length)
    return `${prefix}/api/v1`
  }
  return '/api/v1'
}

const BASE_URL = getBaseUrl()

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

async function fetchWithErrorHandling(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  try {
    return await fetch(input, init)
  } catch {
    throw new ApiError('NETWORK_ERROR')
  }
}

export const client = {
  async get<T>(path: string, params?: Record<string, string>): Promise<T> {
    const url = new URL(BASE_URL + path, window.location.origin)
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        url.searchParams.set(key, value)
      })
    }
    const response = await fetchWithErrorHandling(url.toString(), {
      headers: { 'Accept': 'application/json' },
    })
    return handleResponse<T>(response)
  },

  async post<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetchWithErrorHandling(BASE_URL + path, {
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
    const response = await fetchWithErrorHandling(BASE_URL + path, {
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
    const response = await fetchWithErrorHandling(BASE_URL + path, {
      method: 'DELETE',
      headers: { 'Accept': 'application/json' },
    })
    return handleResponse<void>(response)
  },
}
