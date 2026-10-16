/**
 * Tests for TransactionDetailCard: renders a transaction's own fields (metadata
 * when present, a fallback alert when absent), and toggles the raw-description
 * row between truncated and full display on click.
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import type { Transaction } from '@/api/types'
import TransactionDetailCard from './TransactionDetailCard.vue'

const vuetify = createVuetify({ components, directives })

function baseTransaction(metadata: Transaction['metadata'] = null): Transaction {
  return {
    id: 't1',
    entry_id: 'e1',
    amount: 1000,
    date: '2026-01-01',
    title: 'Groceries',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    tags: [],
    metadata,
  }
}

function mountCard(transaction: Transaction) {
  return mount(TransactionDetailCard, {
    props: { transaction },
    global: { plugins: [i18n, vuetify] },
  })
}

describe('TransactionDetailCard', () => {
  it('renders metadata fields when present', () => {
    const wrapper = mountCard(
      baseTransaction({
        source: 'csv',
        merchant: 'Walmart',
        city: 'Anytown',
        country: 'US',
        card_last4: '1234',
        raw_type: 'POS',
        currency: 'USD',
        raw_description: 'WALMART SUPERCENTER #1234 ANYTOWN',
        raw_fragments: null,
        original_amount_fragment: null,
      }),
    )

    expect(wrapper.text()).toContain('Walmart')
    expect(wrapper.text()).toContain('Anytown')
    expect(wrapper.text()).toContain('US')
    expect(wrapper.text()).toContain('POS')
    expect(wrapper.text()).toContain('1234')
    expect(wrapper.text()).toContain('USD')
    expect(wrapper.find('.description-truncated').exists()).toBe(true)
  })

  it('renders the no-metadata fallback alert when metadata is absent', () => {
    const wrapper = mountCard(baseTransaction(null))

    expect(wrapper.text()).toContain(i18n.global.t('uncategorized.noMetadata'))
    expect(wrapper.find('.description-truncated').exists()).toBe(false)
  })

  it('toggles the raw-description row between truncated and full on click', async () => {
    const wrapper = mountCard(
      baseTransaction({
        source: 'csv',
        merchant: null,
        city: null,
        country: null,
        card_last4: null,
        raw_type: null,
        currency: null,
        raw_description: 'WALMART SUPERCENTER #1234 ANYTOWN',
        raw_fragments: null,
        original_amount_fragment: null,
      }),
    )

    const truncated = wrapper.find('.description-truncated')
    expect(truncated.exists()).toBe(true)
    await truncated.trigger('click')

    expect(wrapper.find('.description-truncated').exists()).toBe(false)
    const full = wrapper.find('.description-full')
    expect(full.exists()).toBe(true)
    expect(full.text()).toContain('WALMART SUPERCENTER #1234 ANYTOWN')
  })

  it('renders the raw-fragments row when present, and omits it when null', () => {
    const withFragments = mountCard(
      baseTransaction({
        source: 'csv',
        merchant: null,
        city: null,
        country: null,
        card_last4: null,
        raw_type: null,
        currency: null,
        raw_description: null,
        raw_fragments: 'NETFLIX.COM | AMSTERDAM | NLD',
        original_amount_fragment: null,
      }),
    )
    expect(withFragments.text()).toContain('NETFLIX.COM | AMSTERDAM | NLD')

    const withoutFragments = mountCard(
      baseTransaction({
        source: 'csv',
        merchant: null,
        city: null,
        country: null,
        card_last4: null,
        raw_type: null,
        currency: null,
        raw_description: null,
        raw_fragments: null,
        original_amount_fragment: null,
      }),
    )
    expect(withoutFragments.text()).not.toContain('Raw fragments')
  })
})
