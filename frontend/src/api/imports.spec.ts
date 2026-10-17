/**
 * Tests for the imports API module: verifies `withCharset` produces the
 * correct MIME type per charset while leaving the file's name and content
 * unchanged.
 */
import { describe, it, expect } from 'vitest'
import { withCharset } from './imports'

describe('withCharset', () => {
  it('produces plain text/csv for utf-8', () => {
    const file = new File(['a,b,c'], 'transactions.csv')
    const result = withCharset(file, 'utf-8')
    expect(result.type).toBe('text/csv')
  })

  it('produces text/csv with charset param for windows-1250', () => {
    const file = new File(['a,b,c'], 'transactions.csv')
    const result = withCharset(file, 'windows-1250')
    expect(result.type).toBe('text/csv; charset=windows-1250')
  })

  it('preserves the file name and content', async () => {
    const file = new File(['a,b,c'], 'transactions.csv')
    const result = withCharset(file, 'utf-8')
    expect(result.name).toBe('transactions.csv')
    await expect(result.text()).resolves.toBe('a,b,c')
  })
})
