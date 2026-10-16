/**
 * Tests for RuleEditorSheet: pre-fills from the `rule` prop for edit vs.
 * starts with one empty row for create, calls the right rulesApi method on
 * save, and surfaces a regex-validation error from the API.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import { rulesApi } from '@/api/rules'
import { ApiError } from '@/api/types'
import type { MatchingRule } from '@/api/types'
import MatchCriterionRow from '@/components/uncategorized/MatchCriterionRow.vue'
import RuleEditorSheet from './RuleEditorSheet.vue'

vi.mock('@/api/rules', () => ({
  rulesApi: {
    create: vi.fn(),
    update: vi.fn(),
  },
}))

const vuetify = createVuetify({ components, directives })

// jsdom has no ResizeObserver; Vuetify's loading-spinner button needs one to mount.
// ponytail: minimal stub, matches the existing convention in UncategorizedReviewView.spec.ts.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver

// jsdom has no visualViewport; Vuetify's overlay positioning (used by v-bottom-sheet) reads it.
// ponytail: minimal stub, matches the existing convention in UncategorizedReviewView.spec.ts.
globalThis.visualViewport ??= {
  addEventListener() {},
  removeEventListener() {},
} as unknown as VisualViewport

let mountedWrapper: ReturnType<typeof mount> | undefined

// VBottomSheet teleports its content to document.body; attaching the wrapper
// there (rather than leaving it detached) keeps that teleported content
// reachable via plain `document` queries in the save-button tests below.
// Tracked in `mountedWrapper` so `afterEach` can unmount it, since an
// attached wrapper otherwise leaves its DOM (and teleported buttons) behind
// for the next test to find.
function mountSheet(rule: MatchingRule | null) {
  mountedWrapper = mount(RuleEditorSheet, {
    props: { modelValue: true, rule },
    global: { plugins: [i18n, vuetify] },
    attachTo: document.body,
  })
  return mountedWrapper
}

/** Finds a teleported `v-btn` by its exact rendered text, via `document` since VTU's DOM-based lookups don't see teleported content. */
function findButtonByText(text: string): HTMLButtonElement {
  const button = Array.from(document.querySelectorAll('button')).find((b) => b.textContent?.trim() === text)
  if (!button) throw new Error(`no button with text "${text}" found`)
  return button
}

function userRule(): MatchingRule {
  return {
    id: 'r1',
    criteria: [{ field: 'merchant', contains: 'acme', regex: null }],
    category: 'groceries',
    editable: true,
  }
}

describe('RuleEditorSheet', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    mountedWrapper?.unmount()
    mountedWrapper = undefined
  })

  it('starts with one empty row and empty category when creating', () => {
    const wrapper = mountSheet(null)
    expect(wrapper.findAllComponents(MatchCriterionRow)).toHaveLength(1)
    expect(wrapper.findComponent(MatchCriterionRow).props('pattern')).toBe('')
  })

  it('pre-fills rows and category from the rule prop when editing', () => {
    const wrapper = mountSheet(userRule())
    const row = wrapper.findComponent(MatchCriterionRow)
    expect(row.props('field')).toBe('merchant')
    expect(row.props('matchType')).toBe('contains')
    expect(row.props('pattern')).toBe('acme')

    const categoryFields = wrapper.findAllComponents({ name: 'VTextField' })
    const categoryField = categoryFields[categoryFields.length - 1]
    expect(categoryField?.props('modelValue')).toBe('groceries')
  })

  it('calls rulesApi.create and emits saved when creating', async () => {
    vi.mocked(rulesApi.create).mockResolvedValue(userRule())
    const wrapper = mountSheet(null)

    await wrapper.findComponent(MatchCriterionRow).vm.$emit('update', {
      field: 'merchant',
      matchType: 'contains',
      pattern: 'acme',
    })
    const categoryFields = wrapper.findAllComponents({ name: 'VTextField' })
    const categoryField = categoryFields[categoryFields.length - 1]
    await categoryField?.setValue('groceries')

    findButtonByText(i18n.global.t('common.save')).click()
    await flushPromises()

    expect(rulesApi.create).toHaveBeenCalledWith({
      criteria: [{ field: 'merchant', contains: 'acme', regex: null }],
      category: 'groceries',
    })
    expect(wrapper.emitted('saved')).toHaveLength(1)
  })

  it('calls rulesApi.update with the rule id when editing', async () => {
    vi.mocked(rulesApi.update).mockResolvedValue(userRule())
    const wrapper = mountSheet(userRule())

    findButtonByText(i18n.global.t('common.save')).click()
    await flushPromises()

    expect(rulesApi.update).toHaveBeenCalledWith('r1', {
      criteria: [{ field: 'merchant', contains: 'acme', regex: null }],
      category: 'groceries',
    })
  })

  it('shows a translated error message when the API rejects an invalid regex', async () => {
    vi.mocked(rulesApi.update).mockRejectedValue(
      new ApiError('MATCHING_RULE_INVALID_REGEX', { pattern: '(', reason: 'unclosed group' }),
    )
    const wrapper = mountSheet(userRule())

    findButtonByText(i18n.global.t('common.save')).click()
    await vi.waitFor(() => expect(document.body.textContent).toContain('unclosed group'))
  })
})
