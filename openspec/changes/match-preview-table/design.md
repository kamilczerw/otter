## Context

See `docs/superpowers/specs/2026-10-14-match-preview-table-design.md` for the full brainstormed design this change implements; this document summarizes the technical decisions for OpenSpec purposes.

`count_matches` (`backend/crates/api/src/handlers/uncategorized.rs`) loads the entire `UNCATEGORIZED`-category queue into memory on every preview call and filters it in Rust — no SQL-level filtering or pagination exists today. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- Extend the existing preview endpoint rather than adding a new one, keeping today's count-only callers unaffected.
- Keep the added backend cost to "slice a page out of a scan already being done," not a new query shape.

**Non-Goals:**
- Pagination or async processing of the uncategorized queue itself (see "Performance note" in the linked design doc — explicitly deferred).
- Any change to the editable `TransactionDrawer`/`TransactionForm` flow; the new per-row detail view is a separate, read-only component.

## Decisions

- **Extend `/transactions-review/preview` with optional fields instead of a new endpoint.** Both the count and the paginated matches come from the same in-memory filter pass; a second endpoint would either duplicate that filter or require sharing state across two requests. Optional `page`/`exclude_transaction_id` request fields and optional `other_count`/`transactions` response fields keep the existing count-only contract byte-for-byte unchanged when `page` is omitted.
- **Server-side page-size cap, independent of the client's requested `limit`.** Bounds response size regardless of what a client sends, without needing a separate validation error path — an oversized `limit` is just silently capped.
- **Exclude a transaction id server-side, not client-side.** The head transaction is already known to always match its own derived rule; filtering it out in Rust (same pass as the match scan) avoids sending it over the wire just to drop it in the UI.
- **Frontend fetches the paginated list only while the table is expanded**, decoupled from the existing debounced count-only call. This was an explicit tradeoff in favor of laziness over request-count minimalism: collapsed users (the common case while typing a draft rule) never pay for transaction payloads they won't see.
- **Extract `TransactionDetailCard` as a shared presentational component** used by both the existing head card and the new row-click peek, rather than duplicating the metadata-rendering markup. Two call sites, one component, no new behavior for the existing head card.

## Risks / Trade-offs

- [Risk] Expanding the table while actively editing criteria doubles the backend scan per debounce tick (one count-only call, one paginated call) → Mitigation: accepted; the scan is already an in-memory filter over a single-household queue (see performance note in proposal/linked design doc), and this only happens while the table is open, not during normal rule editing.
- [Risk] A future multi-user/larger-household version could make the full-queue scan slow → Mitigation: explicitly out of scope here; flagged in project memory for future pagination/async work, not addressed by this change.

## Open Questions

(none — the linked brainstormed design doc resolved scope, UX, and API shape before this change was created.)
