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
import type { PreviewRuleResponse, Transaction, UncategorizedQueue } from '@/api/types'
import MatchCriterionRow from '@/components/uncategorized/MatchCriterionRow.vue'
import MatchPreviewTable from '@/components/uncategorized/MatchPreviewTable.vue'
import TransactionDetailCard from '@/components/transactions/TransactionDetailCard.vue'
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

// jsdom has no visualViewport; Vuetify's overlay positioning (used by v-bottom-sheet) reads it.
// ponytail: minimal stub, upgrade to a shared test-setup file if more specs need it.
globalThis.visualViewport ??= {
  addEventListener() {},
  removeEventListener() {},
} as unknown as VisualViewport

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

describe('UncategorizedReviewView match preview table', () => {
  /** A matched "other" transaction fixture for the preview table's paginated responses. */
  function makeMatchedTransaction(id: string): Transaction {
    return {
      id,
      entry_id: 'e1',
      amount: 500,
      date: '2026-02-02',
      title: `Matched ${id}`,
      created_at: '2026-02-02T00:00:00Z',
      updated_at: '2026-02-02T00:00:00Z',
      tags: [],
      metadata: null,
    }
  }

  /** Clicks "Add criterion" and sets the new row's pattern, triggering the rows watcher. */
  async function addCriterionWithPattern(wrapper: ReturnType<typeof mountView>, pattern: string): Promise<void> {
    const addButton = wrapper.findAll('button').find((b) => b.text() === i18n.global.t('uncategorized.addCriterion'))
    expect(addButton).toBeTruthy()
    await addButton!.trigger('click')
    const row = wrapper.findComponent(MatchCriterionRow)
    await row.vm.$emit('update', { field: 'merchant', matchType: 'contains', pattern })
  }

  /** Edits an already-present criterion row's pattern, triggering the rows watcher. */
  async function editCriterionPattern(wrapper: ReturnType<typeof mountView>, pattern: string): Promise<void> {
    const row = wrapper.findComponent(MatchCriterionRow)
    await row.vm.$emit('update', { field: 'merchant', matchType: 'contains', pattern })
  }

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(makeQueue())
    vi.mocked(categoriesApi.list).mockResolvedValue([])
  })

  it('does not request a page while the table is collapsed, even across multiple criteria edits', async () => {
    vi.mocked(uncategorizedApi.preview).mockResolvedValue({ count: 3 })
    const router = await makeRouter('/uncategorized')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()

    await addCriterionWithPattern(wrapper, 'foo')
    await vi.waitFor(() => expect(uncategorizedApi.preview).toHaveBeenCalledTimes(1))

    await editCriterionPattern(wrapper, 'foobar')
    await vi.waitFor(() => expect(uncategorizedApi.preview).toHaveBeenCalledTimes(2))

    const calls = vi.mocked(uncategorizedApi.preview).mock.calls
    expect(calls.every(([req]) => req.page === undefined)).toBe(true)
  })

  it('fetches a page of matches when the table is expanded', async () => {
    const matched = [makeMatchedTransaction('m1'), makeMatchedTransaction('m2')]
    vi.mocked(uncategorizedApi.preview).mockImplementation(
      async (req): Promise<PreviewRuleResponse> =>
        req.page
          ? { count: 2, other_count: 2, transactions: matched }
          : { count: 2 },
    )
    const router = await makeRouter('/uncategorized')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()

    await wrapper.findComponent(MatchPreviewTable).vm.$emit('toggle')

    await vi.waitFor(() => expect(uncategorizedApi.preview).toHaveBeenCalledTimes(1))
    const [req] = vi.mocked(uncategorizedApi.preview).mock.calls[0]
    expect(req.page).toEqual({ offset: 0, limit: 20 })
    expect(req.exclude_transaction_id).toBe('t1')

    await vi.waitFor(() => expect(wrapper.findComponent(MatchPreviewTable).props('transactions')).toEqual(matched))
  })

  it('re-fetches the expanded table on a criteria change', async () => {
    const firstPage = [makeMatchedTransaction('m1')]
    const secondPage = [makeMatchedTransaction('m2'), makeMatchedTransaction('m3')]
    vi.mocked(uncategorizedApi.preview).mockImplementation(async (req): Promise<PreviewRuleResponse> => {
      if (!req.page) return { count: 1 }
      return req.criteria.length === 0
        ? { count: 1, other_count: 1, transactions: firstPage }
        : { count: 2, other_count: 2, transactions: secondPage }
    })
    const router = await makeRouter('/uncategorized')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()

    await wrapper.findComponent(MatchPreviewTable).vm.$emit('toggle')
    await vi.waitFor(() => expect(wrapper.findComponent(MatchPreviewTable).props('transactions')).toEqual(firstPage))

    await addCriterionWithPattern(wrapper, 'foo')

    await vi.waitFor(() => expect(wrapper.findComponent(MatchPreviewTable).props('transactions')).toEqual(secondPage))
    const pagedCalls = vi.mocked(uncategorizedApi.preview).mock.calls.filter(([req]) => req.page !== undefined)
    expect(pagedCalls.length).toBeGreaterThanOrEqual(2)
  })

  it('opens the peek with the clicked transaction, not the head transaction', async () => {
    const matched = [makeMatchedTransaction('m1'), makeMatchedTransaction('m2')]
    vi.mocked(uncategorizedApi.preview).mockImplementation(
      async (req): Promise<PreviewRuleResponse> =>
        req.page
          ? { count: 2, other_count: 2, transactions: matched }
          : { count: 2 },
    )
    const router = await makeRouter('/uncategorized')
    const wrapper = mountView(router)
    await vi.waitFor(() => expect(uncategorizedApi.getQueue).toHaveBeenCalledTimes(1))
    await flushPromises()

    const table = wrapper.findComponent(MatchPreviewTable)
    await table.vm.$emit('toggle')
    await vi.waitFor(() => expect(table.props('transactions')).toEqual(matched))

    await table.vm.$emit('row-click', matched[1])
    await flushPromises()

    const sheet = wrapper.findComponent({ name: 'VBottomSheet' })
    expect(sheet.props('modelValue')).toBe(true)
    const peekCard = sheet.findComponent(TransactionDetailCard)
    expect(peekCard.props('transaction')).toEqual(matched[1])
  })
})
