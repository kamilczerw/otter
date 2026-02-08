use std::sync::Arc;
use ulid::Ulid;

use crate::entities::{NewTransaction, Transaction};
use crate::errors::TransactionError;
use crate::ports::{BudgetEntryRepository, TransactionRepository};
use crate::types::{Money, TransactionDate};

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

    pub async fn create(
        &self,
        entry_id: Ulid,
        amount: Money,
        date: TransactionDate,
    ) -> Result<Transaction, TransactionError> {
        // Validate amount >= 0
        if amount.value() < 0 {
            return Err(TransactionError::InvalidAmount {
                value: amount.value(),
            });
        }

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
        };

        self.transaction_repo.create(new_transaction).await
    }

    pub async fn update(
        &self,
        id: &Ulid,
        entry_id: Option<Ulid>,
        amount: Option<Money>,
        date: Option<TransactionDate>,
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
        if let Some(ref amt) = amount {
            if amt.value() < 0 {
                return Err(TransactionError::InvalidAmount {
                    value: amt.value(),
                });
            }
        }

        self.transaction_repo.update(id, entry_id, amount, date).await
    }

    pub async fn delete(&self, id: &Ulid) -> Result<(), TransactionError> {
        self.transaction_repo.delete(id).await
    }
}
