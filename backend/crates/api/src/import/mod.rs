//! CSV import orchestration: turns an uploaded bank CSV export into
//! persisted `Transaction`s, auto-provisioning the `Month`/`Category`/
//! `BudgetEntry` rows they need along the way.
//!
//! This module composes `csv_import`'s pure decoding/parsing/extraction/
//! categorization functions with the existing `domain` services — it adds
//! no new domain rules of its own (see the design doc's "Where the
//! orchestration logic lives" section for why this lives in `api`, not
//! `domain`). The HTTP handler that calls into this module (section 5 of
//! the change) owns request parsing and response-DTO mapping; this module
//! only exposes [`pipeline::run_import`] and the plain result/dependency
//! types it needs.

mod categorize;
mod decode;
mod matchable;
mod pipeline;
mod provisioning;
mod rule_conversion;

pub use categorize::UNCATEGORIZED;
pub use decode::{Charset, ImportError, RowFailure, RowOutcome};
pub use matchable::MatchableImportDetails;
pub use pipeline::{run_import, ImportOutcome};
pub use provisioning::{
    find_or_create_category, find_or_create_entry, find_or_create_month, ImportDeps, RowError,
};
pub use rule_conversion::csv_rule_to_new_matching_rule;
