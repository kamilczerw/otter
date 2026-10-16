//! Port for persisting and retrieving user-stored matching rules.

use async_trait::async_trait;
use crate::entities::{MatchingRule, NewMatchingRule};
use crate::errors::MatchingRuleError;

/// Storage port for user-stored [`MatchingRule`]s. Config-defined default
/// rules are not stored through this port; they come from config and are
/// read-only (see the `add-rule-editor-ui` design doc).
#[async_trait]
pub trait MatchingRuleRepository: Send + Sync {
    /// Lists every stored user rule in evaluation order (creation order,
    /// per the `position` column assigned at `create` time).
    ///
    /// # Errors
    ///
    /// Returns a [`MatchingRuleError::Repository`] if the underlying storage
    /// fails.
    async fn list_all(&self) -> Result<Vec<MatchingRule>, MatchingRuleError>;

    /// Creates a new user rule, appending it after every existing rule in
    /// evaluation order.
    ///
    /// # Errors
    ///
    /// Returns a [`MatchingRuleError::Repository`] if the underlying storage
    /// fails.
    async fn create(&self, rule: NewMatchingRule) -> Result<MatchingRule, MatchingRuleError>;

    /// Updates an existing user rule's criteria and category in place,
    /// without changing its position in the evaluation order.
    ///
    /// # Errors
    ///
    /// Returns [`MatchingRuleError::NotFound`] if no rule with `id` exists,
    /// or a [`MatchingRuleError::Repository`] if the underlying storage
    /// fails.
    async fn update(&self, id: &ulid::Ulid, rule: NewMatchingRule) -> Result<MatchingRule, MatchingRuleError>;

    /// Permanently removes a user rule.
    ///
    /// # Errors
    ///
    /// Returns [`MatchingRuleError::NotFound`] if no rule with `id` exists,
    /// or a [`MatchingRuleError::Repository`] if the underlying storage
    /// fails.
    async fn delete(&self, id: &ulid::Ulid) -> Result<(), MatchingRuleError>;
}
