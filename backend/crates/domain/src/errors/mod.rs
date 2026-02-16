use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid budget month: {reason}")]
    InvalidBudgetMonth { reason: String },
    #[error("Invalid due day: {value}")]
    InvalidDueDay { value: u8 },
    #[error("Invalid category name: {reason}")]
    InvalidCategoryName { reason: String },
    #[error("Invalid transaction date: {reason}")]
    InvalidTransactionDate { reason: String },
}

#[derive(Debug, Error)]
pub enum CategoryError {
    #[error("Category not found")]
    NotFound,
    #[error("Category name already exists: {name}")]
    NameAlreadyExists { name: String },
    #[error("Invalid category name format: {reason}")]
    InvalidNameFormat { reason: String },
    #[error("Repository error: {0}")]
    Repository(String),
}

#[derive(Debug, Error)]
pub enum MonthError {
    #[error("Month already exists: {month}")]
    AlreadyExists { month: String },
    #[error("Invalid month format: {value}")]
    InvalidFormat { value: String },
    #[error("Month not found")]
    NotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("Category already in month")]
    CategoryAlreadyInMonth { category_id: String, month: String },
    #[error("Entry not found")]
    NotFound,
    #[error("Entry has transactions")]
    HasTransactions { transaction_count: i64 },
    #[error("Invalid due day: {value}")]
    InvalidDueDay { value: u8 },
    #[error("Category not found")]
    CategoryNotFound,
    #[error("Month not found")]
    MonthNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Invalid amount: {value}")]
    InvalidAmount { value: i64 },
    #[error("Transaction not found")]
    NotFound,
    #[error("Entry not found")]
    EntryNotFound,
    #[error("Invalid date: {value}")]
    InvalidDate { value: String },
    #[error("Title too long: {length} characters (max {max})")]
    TitleTooLong { length: usize, max: usize },
    #[error("Repository error: {0}")]
    Repository(String),
}
