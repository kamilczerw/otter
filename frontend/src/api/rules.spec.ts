/**
 * Tests for the rules API module: verifies each function calls the right
 * HTTP client method/path, and that `importFile` wraps the given `File` in
 * a multipart `FormData` under the backend's expected "file" field name.
 */
import { describe, it, expect, vi } from 'vitest'
import { client } from './client'
import { rulesApi } from './rules'
import type { MatchingRule, MatchingRuleRequest } from './types'

vi.mock('./client', () => ({
  client: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    postForm: vi.fn(),
  },
}))

describe('rulesApi', () => {
  it('list() GETs /rules', () => {
    rulesApi.list()
    expect(client.get).toHaveBeenCalledWith('/rules')
  })

  it('create() POSTs the rule to /rules', () => {
    const data: MatchingRuleRequest = { criteria: [], category: 'groceries' }
    rulesApi.create(data)
    expect(client.post).toHaveBeenCalledWith('/rules', data)
  })

  it('update() PUTs the rule to /rules/{id}', () => {
    const data: MatchingRuleRequest = { criteria: [], category: 'groceries' }
    rulesApi.update('r1', data)
    expect(client.put).toHaveBeenCalledWith('/rules/r1', data)
  })

  it('remove() DELETEs /rules/{id}', () => {
    rulesApi.remove('r1')
    expect(client.delete).toHaveBeenCalledWith('/rules/r1')
  })

  it('importFile() posts a FormData with the file under the "file" field', () => {
    const file = new File(['content'], 'rules.toml')
    rulesApi.importFile(file)

    expect(client.postForm).toHaveBeenCalledTimes(1)
    const [path, formData] = vi.mocked(client.postForm).mock.calls[0]
    expect(path).toBe('/rules/import')
    expect(formData).toBeInstanceOf(FormData)
    expect((formData as FormData).get('file')).toBe(file)
  })

  it('list() resolves with the typed rule array returned by the client', async () => {
    const rules: MatchingRule[] = [{ id: null, criteria: [], category: 'rent', editable: false }]
    vi.mocked(client.get).mockResolvedValueOnce(rules)
    await expect(rulesApi.list()).resolves.toEqual(rules)
  })
})
