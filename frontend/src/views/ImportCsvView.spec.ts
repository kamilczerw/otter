/**
 * Tests for ImportCsvView: charset selection, uploading a CSV via the native
 * file input, rendering the import result (stat chips + optional failed-rows
 * panel), and surfacing an ApiError as an alert.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import { importsApi } from '@/api/imports'
import { ApiError } from '@/api/types'
import type { ImportResponse } from '@/api/types'
import ImportCsvView from './ImportCsvView.vue'

vi.mock('@/api/imports', () => ({
  importsApi: {
    importCsv: vi.fn(),
  },
}))

const vuetify = createVuetify({ components, directives })

// ponytail: minimal stub matching the existing convention in UncategorizedReviewView.spec.ts.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver

// jsdom has no visualViewport; Vuetify's overlay positioning reads it.
// ponytail: minimal stub, matches the existing convention in UncategorizedReviewView.spec.ts.
globalThis.visualViewport ??= {
  addEventListener() {},
  removeEventListener() {},
} as unknown as VisualViewport

function makeResult(overrides: Partial<ImportResponse> = {}): ImportResponse {
  return {
    imported: 3,
    duplicates: 1,
    uncategorized: 2,
    created_months: [],
    created_categories: [],
    failed: [],
    ...overrides,
  }
}

function mountView() {
  return mount(ImportCsvView, {
    global: { plugins: [i18n, vuetify] },
  })
}

async function selectFile(wrapper: ReturnType<typeof mountView>, filename = 'bank.csv') {
  const file = new File(['content'], filename)
  const input = wrapper.find('[data-testid="import-file-input"]').element as HTMLInputElement
  Object.defineProperty(input, 'files', { value: [file], configurable: true })
  await wrapper.find('[data-testid="import-file-input"]').trigger('change')
  await flushPromises()
  return file
}

describe('ImportCsvView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('defaults the charset to UTF-8', () => {
    const wrapper = mountView()
    const select = wrapper.findComponent({ name: 'VSelect' })
    expect(select.props('modelValue')).toBe('utf-8')
  })

  it('calls importCsv with the selected file and the currently selected charset', async () => {
    vi.mocked(importsApi.importCsv).mockResolvedValue(makeResult())
    const wrapper = mountView()

    const file = await selectFile(wrapper)

    expect(importsApi.importCsv).toHaveBeenCalledWith(file, 'utf-8')
  })

  it('renders the three stat chips on a successful response', async () => {
    vi.mocked(importsApi.importCsv).mockResolvedValue(makeResult({ imported: 5, duplicates: 2, uncategorized: 1 }))
    const wrapper = mountView()

    await selectFile(wrapper)

    const text = wrapper.text()
    expect(text).toContain('5')
    expect(text).toContain('2')
    expect(text).toContain('1')
  })

  it('renders the failed-rows expansion panel with the correct count and row content', async () => {
    vi.mocked(importsApi.importCsv).mockResolvedValue(
      makeResult({
        failed: [{ row: 3, reason: 'bad amount', raw: '2026-01-01;oops;x' }],
      }),
    )
    const wrapper = mountView()

    await selectFile(wrapper)

    expect(wrapper.findComponent({ name: 'VExpansionPanels' }).exists()).toBe(true)
    expect(wrapper.text()).toContain('1 rows failed')

    await wrapper.findComponent({ name: 'VExpansionPanelTitle' }).trigger('click')
    await flushPromises()

    const text = wrapper.text()
    expect(text).toContain('Row 3: bad amount')
    expect(text).toContain('2026-01-01;oops;x')
  })

  it('renders no expansion panel when there are no failed rows', async () => {
    vi.mocked(importsApi.importCsv).mockResolvedValue(makeResult({ failed: [] }))
    const wrapper = mountView()

    await selectFile(wrapper)

    expect(wrapper.findComponent({ name: 'VExpansionPanels' }).exists()).toBe(false)
  })

  it('renders the error alert and no result panel when importCsv throws an ApiError', async () => {
    vi.mocked(importsApi.importCsv).mockRejectedValue(new ApiError('IMPORT_FAILED'))
    const wrapper = mountView()

    await selectFile(wrapper)

    expect(wrapper.findComponent({ name: 'VAlert' }).exists()).toBe(true)
    expect(wrapper.findComponent({ name: 'VExpansionPanels' }).exists()).toBe(false)
    expect(wrapper.find('.glass-card').exists()).toBe(false)
  })

  it('re-triggers the change handler when selecting the same filename twice', async () => {
    vi.mocked(importsApi.importCsv).mockResolvedValue(makeResult())
    const wrapper = mountView()

    await selectFile(wrapper, 'same.csv')
    await selectFile(wrapper, 'same.csv')

    expect(importsApi.importCsv).toHaveBeenCalledTimes(2)
  })
})
