use serde::{Deserialize, Serialize};
use crate::types::{Money, TransactionDate};

/// Maximum length for transaction title
pub const MAX_TITLE_LENGTH: usize = 50;

/// Direction of a transaction, derived from the sign of its amount.
///
/// This is never stored independently: a transaction's kind is always
/// recomputed from `amount` via [`Transaction::kind`], so the signed amount
/// remains the single source of truth for direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    /// A negative amount: money leaving the budget.
    Expense,
    /// A zero or positive amount: money entering the budget.
    Income,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: ulid::Ulid,
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
    pub title: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Transaction {
    /// Derives this transaction's direction from the sign of `amount`.
    ///
    /// A negative amount means an expense; zero or a positive amount means
    /// income. The kind is computed on demand rather than stored, so it can
    /// never drift from the amount.
    ///
    /// # Returns
    ///
    /// `TransactionKind::Expense` if `amount` is negative, otherwise `TransactionKind::Income`.
    pub fn kind(&self) -> TransactionKind {
        if self.amount.value() < 0 {
            TransactionKind::Expense
        } else {
            TransactionKind::Income
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub entry_id: ulid::Ulid,
    pub amount: Money,
    pub date: TransactionDate,
    pub title: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    /// Builds a `Transaction` with the given amount, leaving all other
    /// fields as arbitrary fixed values irrelevant to `kind()`.
    fn transaction_with_amount(amount: Money) -> Transaction {
        let now = chrono::Utc::now();
        Transaction {
            id: ulid::Ulid::new(),
            entry_id: ulid::Ulid::new(),
            amount,
            date: TransactionDate::new(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            title: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_kind_negative_amount_is_expense() {
        let txn = transaction_with_amount(Money::new(-500));
        assert_eq!(txn.kind(), TransactionKind::Expense);
    }

    #[test]
    fn test_kind_positive_amount_is_income() {
        let txn = transaction_with_amount(Money::new(500));
        assert_eq!(txn.kind(), TransactionKind::Income);
    }

    #[test]
    fn test_kind_zero_amount_is_income() {
        let txn = transaction_with_amount(Money::new(0));
        assert_eq!(txn.kind(), TransactionKind::Income);
    }
}
