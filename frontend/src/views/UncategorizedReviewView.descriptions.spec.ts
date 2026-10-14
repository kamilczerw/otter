/**
 * Tests for the raw-description row on the transactions-review head card:
 * it shows a truncated single-line preview of `metadata.raw_description` when
 * present, hides entirely when absent, and expands to full text on click.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createRouter, createMemoryHistory } from 'vue-router'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import type { Transaction, UncategorizedQueue } from '@/api/types'
import UncategorizedReviewView from './UncategorizedReviewView.vue'

vi.mock('@/api/uncategorized', () => ({
  uncategorizedApi: {
    getQueue: vi.fn(),
    preview: vi.fn().mockResolvedValue({ count: 0 }),
    saveRule: vi.fn(),
  },
}))

vi.mock('@/api/categories', () => ({
  categoriesApi: {
    list: vi.fn().mockResolvedValue([]),
  },
}))

import { uncategorizedApi } from '@/api/uncategorized'

const vuetify = createVuetify({ components, directives })

function baseTransaction(overrides: Partial<Transaction['metadata']> = {}): Transaction {
  return {
    id: 't1',
    entry_id: 'e1',
    amount: 1000,
    date: '2026-01-01',
    title: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    tags: [],
    metadata:
      overrides === null
        ? null
        : {
            source: 'csv',
            merchant: 'Walmart',
            city: null,
            country: null,
            card_last4: null,
            raw_type: null,
            currency: null,
            raw_description: null,
            original_amount_fragment: null,
            ...overrides,
          },
  }
}

function queueWith(transaction: Transaction): UncategorizedQueue {
  return { remaining: 1, head: { transaction, derived_rule: null } }
}

async function mountView() {
  const router = createRouter({ history: createMemoryHistory(), routes: [] })
  const wrapper = mount(UncategorizedReviewView, {
    global: { plugins: [router, i18n, vuetify] },
  })
  await flushPromises()
  await wrapper.vm.$nextTick()
  return wrapper
}

describe('UncategorizedReviewView raw description', () => {
  beforeEach(() => {
    vi.mocked(uncategorizedApi.getQueue).mockReset()
  })

  it('8.1 shows the raw description truncated when present', async () => {
    const tx = baseTransaction({ raw_description: 'WALMART SUPERCENTER #1234 ANYTOWN' })
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(queueWith(tx))

    const wrapper = await mountView()

    const el = wrapper.find('.description-truncated')
    expect(el.exists()).toBe(true)
    expect(el.text()).toContain('WALMART SUPERCENTER #1234 ANYTOWN')
  })

  it('8.2 renders no description row when metadata is null', async () => {
    const tx = baseTransaction(null as unknown as Partial<Transaction['metadata']>)
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(queueWith(tx))

    const wrapper = await mountView()

    expect(wrapper.find('.description-truncated').exists()).toBe(false)
    expect(wrapper.find('.description-full').exists()).toBe(false)
  })

  it('8.2 renders no description row when raw_description is null', async () => {
    const tx = baseTransaction({ raw_description: null })
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(queueWith(tx))

    const wrapper = await mountView()

    expect(wrapper.find('.description-truncated').exists()).toBe(false)
    expect(wrapper.find('.description-full').exists()).toBe(false)
  })

  it('8.3 toggles to full description on click', async () => {
    const tx = baseTransaction({ raw_description: 'WALMART SUPERCENTER #1234 ANYTOWN' })
    vi.mocked(uncategorizedApi.getQueue).mockResolvedValue(queueWith(tx))

    const wrapper = await mountView()

    const truncated = wrapper.find('.description-truncated')
    expect(truncated.exists()).toBe(true)
    await truncated.trigger('click')

    expect(wrapper.find('.description-truncated').exists()).toBe(false)
    const full = wrapper.find('.description-full')
    expect(full.exists()).toBe(true)
    expect(full.text()).toContain('WALMART SUPERCENTER #1234 ANYTOWN')
  })
})
