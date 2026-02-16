use std::sync::Arc;
use ulid::Ulid;

use crate::entities::{NewTransaction, Transaction, MAX_TITLE_LENGTH};
use crate::errors::TransactionError;
use crate::ports::{BudgetEntryRepository, TransactionRepository};
use crate::types::{Money, TransactionDate};

/// Normalizes title by trimming whitespace and converting empty strings to None.
///
/// # Arguments
///
/// * `title` - Optional title string to normalize
///
/// # Returns
///
/// Normalized title: `None` if input was `None`, empty, or whitespace-only; otherwise `Some` with trimmed string
fn normalize_title(title: Option<String>) -> Option<String> {
    title.and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Validates that a title does not exceed the maximum allowed length.
///
/// # Arguments
///
/// * `title` - Optional title to validate
///
/// # Returns
///
/// `Ok(())` if title is valid (None or within length limit)
///
/// # Errors
///
/// Returns `TransactionError::TitleTooLong` if title exceeds `MAX_TITLE_LENGTH`
fn validate_title_length(title: &Option<String>) -> Result<(), TransactionError> {
    if let Some(t) = title
        && t.len() > MAX_TITLE_LENGTH
    {
        return Err(TransactionError::TitleTooLong {
            length: t.len(),
            max: MAX_TITLE_LENGTH,
        });
    }
    Ok(())
}

pub struct TransactionService {
    transaction_repo: Arc<dyn TransactionRepository>,
    entry_repo: Arc<dyn BudgetEntryRepository>,
}

impl TransactionService {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        entry_repo: Arc<dyn BudgetEntryRepository>,
    ) -> Self {
        Self {
            transaction_repo,
            entry_repo,
        }
    }

    pub async fn list_by_month(
        &self,
        month_id: &Ulid,
    ) -> Result<Vec<Transaction>, TransactionError> {
        self.transaction_repo.list_by_month(month_id).await
    }

    /// Creates a new transaction.
    ///
    /// # Arguments
    ///
    /// * `entry_id` - Budget entry this transaction belongs to
    /// * `amount` - Transaction amount (must be non-negative)
    /// * `date` - Transaction date
    /// * `title` - Optional transaction title (max 50 characters)
    ///
    /// # Returns
    ///
    /// Created transaction entity
    ///
    /// # Errors
    ///
    /// * `TransactionError::InvalidAmount` - Amount is negative
    /// * `TransactionError::EntryNotFound` - Budget entry does not exist
    /// * `TransactionError::TitleTooLong` - Title exceeds maximum length
    /// * `TransactionError::Repository` - Database error
    pub async fn create(
        &self,
        entry_id: Ulid,
        amount: Money,
        date: TransactionDate,
        title: Option<String>,
    ) -> Result<Transaction, TransactionError> {
        // Validate amount >= 0
        if amount.value() < 0 {
            return Err(TransactionError::InvalidAmount {
                value: amount.value(),
            });
        }

        // Normalize and validate title
        let normalized_title = normalize_title(title);
        validate_title_length(&normalized_title)?;

        // Check entry exists
        self.entry_repo
            .find_by_id(&entry_id)
            .await
            .map_err(|e| TransactionError::Repository(e.to_string()))?
            .ok_or(TransactionError::EntryNotFound)?;

        let new_transaction = NewTransaction {
            entry_id,
            amount,
            date,
            title: normalized_title,
        };

        self.transaction_repo.create(new_transaction).await
    }

    /// Updates an existing transaction.
    ///
    /// # Arguments
    ///
    /// * `id` - Transaction ID to update
    /// * `entry_id` - Optional new budget entry ID
    /// * `amount` - Optional new amount (must be non-negative)
    /// * `date` - Optional new date
    /// * `title` - Optional title update: `None` = don't change, `Some(None)` = clear, `Some(Some(v))` = set value
    ///
    /// # Returns
    ///
    /// Updated transaction entity
    ///
    /// # Errors
    ///
    /// * `TransactionError::NotFound` - Transaction does not exist
    /// * `TransactionError::InvalidAmount` - Amount is negative
    /// * `TransactionError::EntryNotFound` - Budget entry does not exist
    /// * `TransactionError::TitleTooLong` - Title exceeds maximum length
    /// * `TransactionError::Repository` - Database error
    pub async fn update(
        &self,
        id: &Ulid,
        entry_id: Option<Ulid>,
        amount: Option<Money>,
        date: Option<TransactionDate>,
        title: Option<Option<String>>,
    ) -> Result<Transaction, TransactionError> {
        // If entry_id provided, verify it exists
        if let Some(ref eid) = entry_id {
            self.entry_repo
                .find_by_id(eid)
                .await
                .map_err(|e| TransactionError::Repository(e.to_string()))?
                .ok_or(TransactionError::EntryNotFound)?;
        }

        // If amount provided, validate >= 0
        if let Some(ref amt) = amount
            && amt.value() < 0
        {
            return Err(TransactionError::InvalidAmount {
                value: amt.value(),
            });
        }

        // If title provided, normalize and validate
        let normalized_title = title.map(normalize_title);
        if let Some(ref t) = normalized_title {
            validate_title_length(t)?;
        }

        self.transaction_repo.update(id, entry_id, amount, date, normalized_title).await
    }

    pub async fn list_by_entry(
        &self,
        entry_id: &Ulid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Transaction>, TransactionError> {
        self.transaction_repo.list_by_entry(entry_id, limit, offset).await
    }

    pub async fn delete(&self, id: &Ulid) -> Result<(), TransactionError> {
        self.transaction_repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_none() {
        assert_eq!(normalize_title(None), None);
    }

    #[test]
    fn test_normalize_title_empty_string() {
        assert_eq!(normalize_title(Some("".to_string())), None);
    }

    #[test]
    fn test_normalize_title_whitespace_only() {
        assert_eq!(normalize_title(Some("   ".to_string())), None);
        assert_eq!(normalize_title(Some("\t\n".to_string())), None);
    }

    #[test]
    fn test_normalize_title_valid() {
        assert_eq!(
            normalize_title(Some("Grocery shopping".to_string())),
            Some("Grocery shopping".to_string())
        );
    }

    #[test]
    fn test_normalize_title_trims_whitespace() {
        assert_eq!(
            normalize_title(Some("  Grocery shopping  ".to_string())),
            Some("Grocery shopping".to_string())
        );
    }

    #[test]
    fn test_validate_title_length_none() {
        assert!(validate_title_length(&None).is_ok());
    }

    #[test]
    fn test_validate_title_length_valid() {
        let title = Some("Grocery shopping".to_string());
        assert!(validate_title_length(&title).is_ok());
    }

    #[test]
    fn test_validate_title_length_at_limit() {
        let title = Some("a".repeat(MAX_TITLE_LENGTH));
        assert!(validate_title_length(&title).is_ok());
    }

    #[test]
    fn test_validate_title_length_too_long() {
        let title = Some("a".repeat(MAX_TITLE_LENGTH + 1));
        let result = validate_title_length(&title);
        assert!(result.is_err());
        match result {
            Err(TransactionError::TitleTooLong { length, max }) => {
                assert_eq!(length, MAX_TITLE_LENGTH + 1);
                assert_eq!(max, MAX_TITLE_LENGTH);
            }
            _ => panic!("Expected TitleTooLong error"),
        }
    }
}
