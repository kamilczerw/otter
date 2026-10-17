## 1. Types and API client

- [x] 1.1 Add `FailedRow` and `ImportResponse` interfaces to `frontend/src/api/types.ts` matching the backend's `ImportResponse`/`FailedRowResponse` shape, and verify `npm run type-check` passes
- [x] 1.2 Add `frontend/src/api/imports.ts` with a `withCharset(file, charset)` helper (returns a new `File` with MIME type `text/csv` or `text/csv; charset=windows-1250`) and `importCsv(file, charset)` calling `client.postForm<ImportResponse>('/imports', formData)`; verify with a unit test asserting `withCharset` produces the correct MIME string for both charsets

## 2. Import view

- [ ] 2.1 Add route `/import` → new `frontend/src/views/ImportCsvView.vue` in the router config, verified by navigating to it in a dev run
- [ ] 2.2 Build `ImportCsvView.vue`: charset `v-select` (UTF-8 default, Windows-1250), hidden file input + upload button (mirroring `RulesView.vue`'s pattern), `v-progress-linear` while in flight, calling `importCsv` on file selection and resetting the input value afterward
- [ ] 2.3 Render the result panel on success: stat chips for `imported`/`duplicates`/`uncategorized`; if `failed.length > 0`, an expansion panel listing `Row {row}: {reason}` with `raw` as a secondary line; `created_months`/`created_categories` are not rendered
- [ ] 2.4 Render an error alert (reusing the existing `ApiError` display pattern from `RulesView.vue`) when the upload request fails, with no result panel shown in that case

## 3. Settings entry point

- [ ] 3.1 Add an "Import CSV" item (`mdi-file-upload-outline`) to the Data section of `frontend/src/components/layout/SettingsDrawer.vue`, with a `goToImport()` handler mirroring `goToRules()` (close drawer, `router.push('/import')`)

## 4. i18n

- [ ] 4.1 Add `import.*` keys (`title`, `charset`, `uploadButton`, `imported`, `duplicates`, `uncategorized`, `failedRows` with `{count}` interpolation) to `frontend/src/i18n/en.json` and `pl.json`, verified by `npm run type-check` and manually toggling language in the running app

## 5. Testing and verification

- [ ] 5.1 Write `ImportCsvView.spec.ts` covering: charset defaults to UTF-8; file selection calls `importCsv` with the chosen charset; a success response renders the three stat chips; a response with `failed` rows renders the expansion panel with correct count/content; a response with no failures renders no expansion panel; an `ApiError` renders the error alert and no result panel; selecting the same filename twice re-triggers the change handler
- [ ] 5.2 Run `npm run type-check` and the full frontend test suite, and verify all pass
- [ ] 5.3 Manually import a real CSV export (one per supported charset) against a local backend instance and confirm the result panel matches the actual `ImportResponse`, per the design doc's Risks note on `File`/`Blob` MIME-type handling
