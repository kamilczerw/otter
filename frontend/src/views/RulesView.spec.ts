/**
 * Tests for RulesView: renders config-default rules (non-editable) before
 * user rules (editable, with Edit/Delete actions), opens the editor sheet
 * for create/edit, deletes a user rule, and imports a file via the native
 * file input.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { i18n } from '@/i18n'
import { rulesApi } from '@/api/rules'
import type { MatchingRule } from '@/api/types'
import RuleEditorSheet from '@/components/rules/RuleEditorSheet.vue'
import RulesView from './RulesView.vue'

vi.mock('@/api/rules', () => ({
  rulesApi: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
    importFile: vi.fn(),
  },
}))

const vuetify = createVuetify({ components, directives })

// ponytail: minimal stub matching the existing convention in UncategorizedReviewView.spec.ts.
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

function makeRules(): MatchingRule[] {
  return [
    { id: null, criteria: [{ field: 'merchant', contains: 'rent-co', regex: null }], category: 'rent', editable: false },
    { id: 'u1', criteria: [{ field: 'merchant', contains: 'acme', regex: null }], category: 'groceries', editable: true },
  ]
}

function mountView() {
  return mount(RulesView, {
    global: { plugins: [i18n, vuetify] },
  })
}

describe('RulesView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(rulesApi.list).mockResolvedValue(makeRules())
  })

  it('renders default rules before user rules', async () => {
    const wrapper = mountView()
    await flushPromises()

    const titles = wrapper.findAll('.v-list-item-title').map((n) => n.text())
    expect(titles).toEqual(['rent', 'groceries'])
  })

  it('does not show Edit/Delete actions for a default rule, but does for a user rule', async () => {
    const wrapper = mountView()
    await flushPromises()

    const items = wrapper.findAllComponents({ name: 'VListItem' })
    const defaultItem = items.find((i) => i.text().includes('rent'))
    const userItem = items.find((i) => i.text().includes('groceries'))
    expect(defaultItem?.findAll('button')).toHaveLength(0)
    expect(userItem?.findAll('button')).toHaveLength(2)
  })

  it('opens the editor sheet for creating a new rule', async () => {
    const wrapper = mountView()
    await flushPromises()

    const newRuleButton = wrapper.findAll('button').find((b) => b.text() === i18n.global.t('rules.newRule'))
    await newRuleButton!.trigger('click')

    const sheet = wrapper.findComponent(RuleEditorSheet)
    expect(sheet.props('modelValue')).toBe(true)
    expect(sheet.props('rule')).toBeNull()
  })

  it('opens the editor sheet pre-filled when editing a user rule', async () => {
    const wrapper = mountView()
    await flushPromises()

    const userItem = wrapper.findAllComponents({ name: 'VListItem' }).find((i) => i.text().includes('groceries'))
    await userItem!.findAll('button')[0].trigger('click')

    const sheet = wrapper.findComponent(RuleEditorSheet)
    expect(sheet.props('rule')?.id).toBe('u1')
  })

  it('reloads the list when the editor sheet emits saved', async () => {
    const wrapper = mountView()
    await flushPromises()
    expect(rulesApi.list).toHaveBeenCalledTimes(1)

    await wrapper.findComponent(RuleEditorSheet).vm.$emit('saved')
    await flushPromises()

    expect(rulesApi.list).toHaveBeenCalledTimes(2)
  })

  it('deletes a user rule after confirmation and reloads the list', async () => {
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    vi.mocked(rulesApi.remove).mockResolvedValue(undefined)
    const wrapper = mountView()
    await flushPromises()

    const userItem = wrapper.findAllComponents({ name: 'VListItem' }).find((i) => i.text().includes('groceries'))
    await userItem!.findAll('button')[1].trigger('click')
    await flushPromises()

    expect(rulesApi.remove).toHaveBeenCalledWith('u1')
    expect(rulesApi.list).toHaveBeenCalledTimes(2)
  })

  it('does not delete when the confirmation is cancelled', async () => {
    vi.spyOn(window, 'confirm').mockReturnValue(false)
    const wrapper = mountView()
    await flushPromises()

    const userItem = wrapper.findAllComponents({ name: 'VListItem' }).find((i) => i.text().includes('groceries'))
    await userItem!.findAll('button')[1].trigger('click')
    await flushPromises()

    expect(rulesApi.remove).not.toHaveBeenCalled()
  })

  it('imports a selected file and shows the import summary', async () => {
    vi.mocked(rulesApi.importFile).mockResolvedValue({ imported: 2, skipped: 1, failed: [] })
    const wrapper = mountView()
    await flushPromises()

    const file = new File(['content'], 'rules.toml')
    const input = wrapper.find('[data-testid="import-file-input"]').element as HTMLInputElement
    Object.defineProperty(input, 'files', { value: [file] })
    await wrapper.find('[data-testid="import-file-input"]').trigger('change')
    await flushPromises()

    expect(rulesApi.importFile).toHaveBeenCalledWith(file)
    expect(wrapper.text()).toContain('2')
    expect(rulesApi.list).toHaveBeenCalledTimes(2)
  })
})
