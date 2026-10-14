/**
 * Regression tests for the `skip` query param reactivity in UncategorizedReviewView.
 *
 * Before the fix, `skip` changes only reloaded the queue through two ad hoc explicit
 * `loadQueue()` calls (one in `handleSkip`, one in `handleSave`). A single
 * `watch(skip, loadQueue)` now covers every way `skip` can change — including direct
 * route-query navigation (back/forward) that isn't driven by the Skip button — while
 * the explicit calls were removed so skip/save don't double-fetch.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createRouter, createMemoryHistory } from 'vue-router'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import { uncategorizedApi } from '@/api/uncategorized'
import { categoriesApi } from '@/api/categories'
import type { UncategorizedQueue } from '@/api/types'
import UncategorizedReviewView from './UncategorizedReviewView.vue'

vi.mock('@/api/uncategorized', () => ({
  uncategorizedApi: {
    getQueue: vi.fn(),
    preview: vi.fn(),
    saveRule: vi.fn(),
  },
}))

vi.mock('@/api/categories', () => ({
  categoriesApi: {
    list: vi.fn(),
  },
}))

const vuetify = createVuetify({ components, directives })

// jsdom has no ResizeObserver; Vuetify's loading-spinner button needs one to mount.
// ponytail: minimal stub, upgrade to a shared test-setup file if more specs need it.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver

/** Minimal fake queue response with a head transaction, enough to render the form. */
function makeQueue(): UncategorizedQueue {
  return {
    remaining: 3,
    head: {
      transaction: {
        id: 't1',
        entry_id: '',
        amount: 1000,
        date: '2026-01-01',
        title: 'Test transaction',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        tags: [],
        metadata: null,
      },
      derived_rule: null,
    },
  }
}

async function makeRouter(initialPath: string) {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/uncategorized', name: 'uncategorized', component: UncategorizedReviewView }],
  })
  router.push(initialPath)
  await router.isReady()
  return router
}

function mountView(router: Awaited<ReturnType<typeof makeRouter>>) {
  return mount(UncategorizedReviewView, {
    global: {
      plugins: [router, i18n, vuetify],
    },
  })
}

describe('UncategorizedReviewView skip reactivity', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(makeQueue())
    vi.mocked(uncategorizedApi.saveRule).mockResolvedValue({ remaining: 2 })
    vi.mocked(categoriesApi.list).mockResolvedValue([])
  })

  it('loads the queue at the skip offset from the initial route query', async () => {
    const router = await makeRouter('/uncategorized?skip=5')
    mountView(router)

    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledWith(5))
  })

  it('reloads the queue when skip changes via route navigation, not just the Skip button', async () => {
    const router = await makeRouter('/uncategorized')
    mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))

    await router.push({ query: { skip: '3' } })

    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(2))
    expect(uncategorizedApi.getQueue).toHaveBeenNthCalledWith(2, 3)
  })

  it('reloads exactly once more when the Skip button is clicked', async () => {
    const router = await makeRouter('/uncategorized')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()

    const skipButton = wrapper.findAll('button').find((b) => b.text() === i18n.global.t('uncategorized.skip'))
    expect(skipButton).toBeTruthy()
    await skipButton!.trigger('click')

    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(2))
    expect(uncategorizedApi.getQueue).toHaveBeenNthCalledWith(2, 1)
  })

  it('reloads the queue at the same skip offset after a successful save', async () => {
    const router = await makeRouter('/uncategorized?skip=4')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()
    expect(uncategorizedApi.getQueue).toHaveBeenNthCalledWith(1, 4)

    // Minimum state for the Save button to be enabled: a non-empty category.
    // No criteria rows are required (an empty rows array builds valid, empty criteria).
    const combobox = wrapper.findComponent({ name: 'VCombobox' })
    await combobox.setValue('groceries')
    await flushPromises()

    const saveButton = wrapper.findAll('button').find((b) => b.text() === i18n.global.t('uncategorized.saveAndApply'))
    expect(saveButton).toBeTruthy()
    await saveButton!.trigger('click')

    await vi.waitFor(() => expect(uncategorizedApi.saveRule).toHaveBeenCalledTimes(1))
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(2))
    expect(uncategorizedApi.getQueue).toHaveBeenNthCalledWith(2, 4)
  })
})
