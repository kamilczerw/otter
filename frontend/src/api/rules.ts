/**
 * API client for the categorization rule editor: listing the merged
 * config-default + user rule set, creating/editing/deleting user rules, and
 * importing rules from an uploaded `.toml` file.
 */
import { client } from './client'
import type { MatchingRule, MatchingRuleRequest, ImportRulesResponse } from './types'

export const rulesApi = {
  /** Fetches the full active rule set: config defaults first, then user rules, in evaluation order. */
  list: () => client.get<MatchingRule[]>('/rules'),
  /** Creates a new user rule, appended after every existing user rule. */
  create: (data: MatchingRuleRequest) => client.post<MatchingRule>('/rules', data),
  /** Replaces an existing user rule's criteria/category in place, keeping its evaluation position. */
  update: (id: string, data: MatchingRuleRequest) => client.put<MatchingRule>(`/rules/${id}`, data),
  /** Permanently deletes a user rule. */
  remove: (id: string) => client.delete(`/rules/${id}`),
  /** Uploads a `.toml` rules file, importing every non-duplicate rule it contains. */
  importFile: (file: File) => {
    const formData = new FormData()
    formData.set('file', file)
    return client.postForm<ImportRulesResponse>('/rules/import', formData)
  },
}
