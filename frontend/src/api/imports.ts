/**
 * API client for CSV transaction import: wraps the user-selected file so its
 * MIME type carries the chosen charset, then uploads it.
 */
import { client } from './client'
import type { ImportResponse } from './types'

export type Charset = 'utf-8' | 'windows-1250'

/** Re-wraps `file` in a new File whose MIME type encodes `charset`, since the backend reads charset from the multipart part's own Content-Type (see backend/crates/api/src/import/decode.rs::parse_charset). */
export function withCharset(file: File, charset: Charset): File {
  const type = charset === 'windows-1250' ? 'text/csv; charset=windows-1250' : 'text/csv'
  return new File([file], file.name, { type })
}

export const importsApi = {
  /** Uploads a CSV file for transaction import, declaring `charset` via the file part's Content-Type. */
  importCsv: (file: File, charset: Charset) => {
    const formData = new FormData()
    formData.set('file', withCharset(file, charset))
    return client.postForm<ImportResponse>('/imports', formData)
  },
}
