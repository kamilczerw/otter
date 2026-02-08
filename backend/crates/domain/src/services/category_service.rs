use std::sync::Arc;
use ulid::Ulid;

use crate::entities::{Category, NewCategory};
use crate::errors::CategoryError;
use crate::ports::CategoryRepository;
use crate::types::CategoryName;

pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_all(&self) -> Result<Vec<Category>, CategoryError> {
        self.repo.list_all().await
    }

    pub async fn create(&self, name: CategoryName) -> Result<Category, CategoryError> {
        let new_category = NewCategory { name };
        self.repo.create(new_category).await
    }

    pub async fn rename(&self, id: &Ulid, name: CategoryName) -> Result<Category, CategoryError> {
        self.repo.update_name(id, name).await
    }
}
