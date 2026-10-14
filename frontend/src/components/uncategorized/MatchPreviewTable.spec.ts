/**
 * Tests for MatchPreviewTable: toggling collapsed/expanded state, emitting
 * load-more when scrolled near the bottom of the expanded table (but only
 * when there's more to load and it's not already loading), and emitting
 * row-click with the clicked transaction.
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import type { Transaction } from '@/api/types'
import MatchPreviewTable from './MatchPreviewTable.vue'

const vuetify = createVuetify({ components, directives })

function makeTransaction(id: string): Transaction {
  return {
    id,
    entry_id: 'e1',
    amount: 1000,
    date: '2026-01-01',
    title: `Transaction ${id}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    tags: [],
    metadata: null,
  }
}

function mountTable(props: {
  transactions?: Transaction[]
  expanded?: boolean
  otherCount?: number
  loading?: boolean
  hasMore?: boolean
}) {
  return mount(MatchPreviewTable, {
    props: {
      transactions: props.transactions ?? [],
      expanded: props.expanded ?? false,
      otherCount: props.otherCount ?? 0,
      loading: props.loading ?? false,
      hasMore: props.hasMore ?? false,
    },
    global: { plugins: [i18n, vuetify] },
  })
}

/** Sets scroll geometry on an element so a scroll event reports "near the bottom". */
function stubNearBottomScroll(el: HTMLElement): void {
  Object.defineProperty(el, 'scrollTop', { value: 90, configurable: true })
  Object.defineProperty(el, 'clientHeight', { value: 100, configurable: true })
  Object.defineProperty(el, 'scrollHeight', { value: 200, configurable: true })
}

describe('MatchPreviewTable', () => {
  it('emits toggle when clicking the collapsed toggle row', async () => {
    const wrapper = mountTable({ expanded: false, otherCount: 5 })

    expect(wrapper.text()).toContain('5')
    await wrapper.find('[data-testid="match-preview-toggle"]').trigger('click')

    expect(wrapper.emitted('toggle')).toHaveLength(1)
  })

  it('emits toggle when clicking the expanded toggle row', async () => {
    const wrapper = mountTable({ expanded: true, transactions: [makeTransaction('t1')] })

    await wrapper.find('[data-testid="match-preview-toggle"]').trigger('click')

    expect(wrapper.emitted('toggle')).toHaveLength(1)
  })

  it('emits load-more when scrolled near the bottom and hasMore is true and not loading', async () => {
    const wrapper = mountTable({
      expanded: true,
      transactions: [makeTransaction('t1')],
      hasMore: true,
      loading: false,
    })

    const container = wrapper.find('[data-testid="match-preview-scroll"]').element as HTMLElement
    stubNearBottomScroll(container)
    await wrapper.find('[data-testid="match-preview-scroll"]').trigger('scroll')

    expect(wrapper.emitted('load-more')).toHaveLength(1)
  })

  it('does not emit load-more when hasMore is false', async () => {
    const wrapper = mountTable({
      expanded: true,
      transactions: [makeTransaction('t1')],
      hasMore: false,
      loading: false,
    })

    const container = wrapper.find('[data-testid="match-preview-scroll"]').element as HTMLElement
    stubNearBottomScroll(container)
    await wrapper.find('[data-testid="match-preview-scroll"]').trigger('scroll')

    expect(wrapper.emitted('load-more')).toBeUndefined()
  })

  it('does not emit load-more when already loading', async () => {
    const wrapper = mountTable({
      expanded: true,
      transactions: [makeTransaction('t1')],
      hasMore: true,
      loading: true,
    })

    const container = wrapper.find('[data-testid="match-preview-scroll"]').element as HTMLElement
    stubNearBottomScroll(container)
    await wrapper.find('[data-testid="match-preview-scroll"]').trigger('scroll')

    expect(wrapper.emitted('load-more')).toBeUndefined()
  })

  it('emits row-click with the clicked transaction', async () => {
    const t1 = makeTransaction('t1')
    const t2 = makeTransaction('t2')
    const wrapper = mountTable({ expanded: true, transactions: [t1, t2] })

    const rows = wrapper.findAll('[data-testid="match-preview-row"]')
    expect(rows).toHaveLength(2)
    await rows[1].trigger('click')

    expect(wrapper.emitted('row-click')).toHaveLength(1)
    expect(wrapper.emitted('row-click')![0]).toEqual([t2])
  })
})
