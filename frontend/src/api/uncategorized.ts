/**
 * API client for the uncategorized-transactions bulk-categorization workflow:
 * fetching the next uncategorized transaction, previewing how many transactions
 * a candidate rule would match, and saving a rule to bulk-categorize matches.
 */
import { client } from './client'
import type {
  UncategorizedQueue,
  PreviewRuleRequest,
  PreviewRuleResponse,
  SaveRuleRequest,
  SaveRuleResponse,
} from './types'

export const uncategorizedApi = {
  /** Fetches the current uncategorized-transaction queue (count remaining + next transaction to review). */
  getQueue: () => client.get<UncategorizedQueue>('/uncategorized-transactions'),
  /** Previews how many transactions a candidate set of match criteria would select, without saving anything. */
  preview: (data: PreviewRuleRequest) =>
    client.post<PreviewRuleResponse>('/uncategorized-transactions/preview', data),
  /** Saves a categorization rule, applying it to all currently matching transactions. */
  saveRule: (data: SaveRuleRequest) =>
    client.post<SaveRuleResponse>('/uncategorized-transactions/rules', data),
}
