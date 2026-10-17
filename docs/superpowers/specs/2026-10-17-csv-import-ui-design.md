# CSV Import UI (Settings → Data)

## Context

The backend CSV import endpoint (`POST /api/v1/imports`, `backend/crates/api/src/handlers/imports.rs`) is fully implemented: it accepts a `multipart/form-data` upload with the file in a part named `"file"`, decodes it per the part's `Content-Type` charset (UTF-8 default, Windows-1250 also supported), parses it via the `csv_import` crate, auto-creates needed months/categories, applies matching rules, persists transactions, and returns an `ImportResponse` with counts and per-row failures. There is currently no way to reach this endpoint from the UI.

The Settings drawer (`frontend/src/components/layout/SettingsDrawer.vue`) already has a "Data" section with two nav items (Uncategorized, Rules), and `RulesView.vue` already implements an equivalent upload flow for importing a `.toml` rules file (hidden file input → `client.postForm()` → inline result alert). This change reuses that exact pattern for CSV transaction import.

## Goals

- Let the user upload a bank CSV export from the UI and see the import result (counts + any row failures), without a terminal/CLI.
- Support the same two charsets the backend already decodes (UTF-8, Windows-1250), since some Polish bank exports aren't UTF-8.
- Follow the existing Settings → Data navigation pattern and the existing Rules-import visual/interaction pattern, so this doesn't introduce a new UI idiom.

## Non-Goals

- Drag-and-drop upload, upload progress percentage, or CSV preview before submitting — none of these exist for the Rules import either.
- Surfacing `created_months` / `created_categories` by name in the result — that state is already visible elsewhere in the app (month list, category list).
- Undo/rollback of a completed import.
- Any backend changes. The endpoint, charset decoding, and response shape are unchanged.

## Design

### 1. Settings entry point

`SettingsDrawer.vue`'s existing "Data" section gets a third item, after Uncategorized and Rules:

```html
<div class="settings-item mt-2" @click="goToImport">
  <div class="settings-item-left">
    <v-icon class="settings-item-icon" size="large">mdi-file-upload-outline</v-icon>
    <div class="settings-item-label">{{ $t('import.title') }}</div>
  </div>
  <v-icon size="small" class="text-cosmic-secondary">mdi-chevron-right</v-icon>
</div>
```

`goToImport()` mirrors `goToRules()`: closes the drawer, `router.push('/import')`.

### 2. Route and view

New route `/import` → new view `frontend/src/views/ImportCsvView.vue`, registered in the router alongside the existing `/rules` and `/uncategorized` routes.

Structure (mirrors `RulesView.vue`):

- Page title (`$t('import.title')`).
- A `v-select` for charset, options `UTF-8` (value `utf-8`, default) / `Windows-1250` (value `windows-1250`), labeled `$t('import.charset')`.
- An upload button (`prepend-icon="mdi-upload"`) that triggers a hidden `<input type="file" accept=".csv">`.
- `@change` handler reads the selected `File`, calls the API function, and populates the result state.
- `v-progress-linear` while the request is in flight (same as Rules).
- On error: `v-alert type="error"` with the `ApiError` message (same pattern as Rules).
- On success: result panel (see below).

The file input is cleared (`input.value = ''`) after each selection so re-uploading the same filename re-triggers `@change`.

### 3. Charset wiring (no `client.ts` changes)

The backend reads the charset from the **file part's own** `Content-Type`, not the request's. The browser's `fetch`/`FormData` already lets a part carry its own MIME type via the `Blob`/`File` passed to `FormData.append`. So charset selection is handled entirely in the new API function, by re-wrapping the selected file:

```ts
// frontend/src/api/imports.ts
function withCharset(file: File, charset: Charset): File {
  const contentType = charset === 'utf-8' ? 'text/csv' : `text/csv; charset=${charset}`
  return new File([file], file.name, { type: contentType })
}

export async function importCsv(file: File, charset: Charset): Promise<ImportResponse> {
  const formData = new FormData()
  formData.append('file', withCharset(file, charset))
  return client.postForm<ImportResponse>('/imports', formData)
}
```

`client.postForm` is used unchanged — no new client method needed. `Charset = 'utf-8' | 'windows-1250'`, matching the two the backend's `decode.rs` supports.

### 4. Result display

Per `ImportResponse { imported, duplicates, uncategorized, created_months, created_categories, failed }`:

- Three stat chips: imported / duplicates / uncategorized counts (`$t('import.imported')`, `$t('import.duplicates')`, `$t('import.uncategorized')`).
- If `failed.length > 0`: an expandable list (`v-expansion-panels`, collapsed by default) titled `$t('import.failedRows', { count: failed.length })`, listing each failure as `Row {row}: {reason}` with the `raw` content as a secondary line.
- `created_months` / `created_categories` are not rendered (Non-Goals).

### 5. Types

`frontend/src/api/types.ts` gains:

```ts
export interface FailedRow {
  row: number
  raw: string
  reason: string
}

export interface ImportResponse {
  imported: number
  duplicates: number
  uncategorized: number
  created_months: string[]
  created_categories: string[]
  failed: FailedRow[]
}
```

### 6. i18n

New flat `import.*` keys added to `frontend/src/i18n/en.json` and `pl.json`, following the existing `rules.*` key shape: `title`, `charset`, `uploadButton`, `imported`, `duplicates`, `uncategorized`, `failedRows` (with `{count}` interpolation), plus reusing the existing generic error-display pattern (no new error-specific keys needed — `ApiError` messages are already shown as-is elsewhere, e.g. Rules).

## Testing

- Frontend: `ImportCsvView.spec.ts` covering — charset select defaults to UTF-8; selecting a file calls `importCsv` with the chosen charset; success response renders the three stat chips; a response with `failed` rows renders the expandable list with correct count and content; a response with no failures renders no expansion panel; an `ApiError` from the API call renders the error alert and no result panel; re-selecting the same filename re-triggers the change handler (input value reset).
- `imports.ts` unit test: `withCharset` produces `text/csv` for `'utf-8'` and `text/csv; charset=windows-1250` for `'windows-1250'`.
- No backend tests needed — endpoint and charset decoding are unchanged and already covered by existing backend tests (`imports_tests.rs`, `decode.rs` unit tests).
