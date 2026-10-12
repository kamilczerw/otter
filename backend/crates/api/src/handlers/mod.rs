pub mod categories;
pub mod entries;
pub mod health;
pub mod imports;
pub mod months;
pub mod summary;
pub mod transactions;

use std::sync::Arc;

use domain::services::{
    CategoryService, EntryService, MonthService, SummaryService, TransactionService,
};

use crate::config::CurrencyConfig;
use crate::import::ImportDeps;

#[derive(Clone)]
pub struct AppState {
    pub category_service: Arc<CategoryService>,
    pub month_service: Arc<MonthService>,
    pub entry_service: Arc<EntryService>,
    pub transaction_service: Arc<TransactionService>,
    pub summary_service: Arc<SummaryService>,
    pub currency_config: CurrencyConfig,
    /// Dependencies for the CSV import pipeline (`crate::import`), built
    /// once at startup from the same repo/service `Arc`s plus resolved
    /// categorization rules — see `handlers::imports::import_transactions`.
    pub import_deps: Arc<ImportDeps>,
}

pub fn parse_ulid(s: &str) -> Result<ulid::Ulid, crate::errors::ApiError> {
    s.parse::<ulid::Ulid>()
        .map_err(|_| crate::errors::ApiError::bad_request(&format!("Invalid ID format: {}", s)))
}
