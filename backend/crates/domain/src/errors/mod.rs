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

/// Errors raised by `MatchingRule`/`MatchCriterion` construction and by
/// `MatchingRuleRepository` implementations.
#[derive(Debug, Error)]
pub enum MatchingRuleError {
    /// A criterion's `Regex` pattern failed to compile. Caught at
    /// construction time (see `MatchCriterion::new`) rather than left to
    /// silently never-match at categorization time.
    #[error("invalid regex pattern {pattern:?}: {reason}")]
    InvalidRegex { pattern: String, reason: String },
    /// No matching rule exists with the requested identity.
    #[error("matching rule not found")]
    NotFound,
    /// The underlying repository failed for a reason not covered above.
    #[error("Repository error: {0}")]
    Repository(String),
}

#[derive(Debug, Error)]
pub enum TransactionError {
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
