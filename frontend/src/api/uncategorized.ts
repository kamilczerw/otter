/**
 * API client for the transactions-review bulk-categorization workflow:
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
  /**
   * Fetches the current uncategorized-transaction queue (count remaining + next transaction to review).
   *
   * @param skip - How many positions to advance past the queue's natural head, wrapping once it
   *   reaches the end. Omitted or 0 fetches the natural head.
   */
  getQueue: (skip?: number) =>
    client.get<UncategorizedQueue>('/transactions-review', skip ? { skip: String(skip) } : undefined),
  /** Previews how many transactions a candidate set of match criteria would select, without saving anything. */
  preview: (data: PreviewRuleRequest) =>
    client.post<PreviewRuleResponse>('/transactions-review/preview', data),
  /** Saves a categorization rule, applying it to all currently matching transactions. */
  saveRule: (data: SaveRuleRequest) =>
    client.post<SaveRuleResponse>('/transactions-review/rules', data),
}
