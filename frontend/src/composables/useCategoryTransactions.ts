import { reactive } from 'vue'
import { transactionsApi } from '@/api/transactions'
import type { Transaction } from '@/api/types'
import { INITIAL_TRANSACTION_COUNT, TRANSACTION_BATCH_SIZE } from '@/constants'

interface CacheEntry {
  transactions: Transaction[]
  offset: number
  hasMore: boolean
  loading: boolean
  initialLoaded: boolean
  showMoreClicked: boolean
}

const cache = reactive(new Map<string, CacheEntry>())

function getOrCreate(entryId: string): CacheEntry {
  if (!cache.has(entryId)) {
    cache.set(entryId, {
      transactions: [],
      offset: 0,
      hasMore: false,
      loading: false,
      initialLoaded: false,
      showMoreClicked: false,
    })
  }
  return cache.get(entryId)!
}

export function useCategoryTransactions() {
  async function load(entryId: string): Promise<void> {
    const entry = getOrCreate(entryId)
    if (entry.initialLoaded) return
    entry.loading = true
    try {
      const result = await transactionsApi.listByEntry(entryId, INITIAL_TRANSACTION_COUNT, 0)
      entry.transactions = result.items
      entry.offset = result.items.length
      entry.hasMore = result.has_more
      entry.initialLoaded = true
    } finally {
      entry.loading = false
    }
  }

  async function loadMore(entryId: string): Promise<void> {
    const entry = getOrCreate(entryId)
    if (entry.loading || !entry.hasMore) return
    entry.loading = true
    entry.showMoreClicked = true
    try {
      const result = await transactionsApi.listByEntry(entryId, TRANSACTION_BATCH_SIZE, entry.offset)
      entry.transactions = [...entry.transactions, ...result.items]
      entry.offset += result.items.length
      entry.hasMore = result.has_more
    } finally {
      entry.loading = false
    }
  }

  async function invalidate(entryId: string): Promise<void> {
    const entry = cache.get(entryId)
    const previousOffset = entry ? entry.offset : INITIAL_TRANSACTION_COUNT
    const wasShowMoreClicked = entry ? entry.showMoreClicked : false

    // Clear the cache entry
    cache.delete(entryId)

    // Re-fetch up to the previously loaded depth
    const newEntry = getOrCreate(entryId)
    newEntry.loading = true
    try {
      const result = await transactionsApi.listByEntry(entryId, previousOffset, 0)
      newEntry.transactions = result.items
      newEntry.offset = result.items.length
      newEntry.hasMore = result.has_more
      newEntry.initialLoaded = true
      newEntry.showMoreClicked = wasShowMoreClicked
    } finally {
      newEntry.loading = false
    }
  }

  function invalidateAll(): void {
    cache.clear()
  }

  function getTransactions(entryId: string): Transaction[] {
    return cache.get(entryId)?.transactions ?? []
  }

  function getHasMore(entryId: string): boolean {
    return cache.get(entryId)?.hasMore ?? false
  }

  function getIsLoading(entryId: string): boolean {
    return cache.get(entryId)?.loading ?? false
  }

  function getShowMoreClicked(entryId: string): boolean {
    return cache.get(entryId)?.showMoreClicked ?? false
  }

  return {
    load,
    loadMore,
    invalidate,
    invalidateAll,
    getTransactions,
    getHasMore,
    getIsLoading,
    getShowMoreClicked,
  }
}
