use async_trait::async_trait;
use crate::entities::{Category, NewCategory};
use crate::errors::CategoryError;
use crate::types::CategoryName;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<Category>, CategoryError>;
    async fn find_by_id(&self, id: &ulid::Ulid) -> Result<Option<Category>, CategoryError>;
    async fn create(&self, category: NewCategory) -> Result<Category, CategoryError>;
    async fn update_name(&self, id: &ulid::Ulid, name: CategoryName) -> Result<Category, CategoryError>;
    async fn update(&self, id: &ulid::Ulid, name: Option<CategoryName>, label: Option<Option<String>>) -> Result<Category, CategoryError>;
}
