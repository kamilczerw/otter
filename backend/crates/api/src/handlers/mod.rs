pub mod categories;
pub mod entries;
pub mod health;
pub mod months;
pub mod summary;
pub mod transactions;

use std::sync::Arc;

use domain::services::{
    CategoryService, EntryService, MonthService, SummaryService, TransactionService,
};

#[derive(Clone)]
pub struct AppState {
    pub category_service: Arc<CategoryService>,
    pub month_service: Arc<MonthService>,
    pub entry_service: Arc<EntryService>,
    pub transaction_service: Arc<TransactionService>,
    pub summary_service: Arc<SummaryService>,
}

pub fn parse_ulid(s: &str) -> Result<ulid::Ulid, crate::errors::ApiError> {
    s.parse::<ulid::Ulid>()
        .map_err(|_| crate::errors::ApiError::bad_request(&format!("Invalid ID format: {}", s)))
}
