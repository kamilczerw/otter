use crate::errors::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryName(String);

impl CategoryName {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.is_empty() {
            return Err(DomainError::InvalidCategoryName {
                reason: "category name must not be empty".to_string(),
            });
        }

        if value.starts_with('/') {
            return Err(DomainError::InvalidCategoryName {
                reason: "category name must not start with '/'".to_string(),
            });
        }

        if value.ends_with('/') {
            return Err(DomainError::InvalidCategoryName {
                reason: "category name must not end with '/'".to_string(),
            });
        }

        if value.contains("//") {
            return Err(DomainError::InvalidCategoryName {
                reason: "category name must not contain empty segments (double slashes)".to_string(),
            });
        }

        let segments: Vec<&str> = value.split('/').collect();

        for segment in &segments {
            if segment.is_empty() {
                return Err(DomainError::InvalidCategoryName {
                    reason: "category name must not contain empty segments".to_string(),
                });
            }
            if !segment
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(DomainError::InvalidCategoryName {
                    reason: format!(
                        "segment '{}' contains invalid characters; only alphanumeric, hyphens, and underscores are allowed",
                        segment
                    ),
                });
            }
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simple_name() {
        let name = CategoryName::new("food").unwrap();
        assert_eq!(name.as_str(), "food");
    }

    #[test]
    fn test_valid_hierarchical_name() {
        let name = CategoryName::new("utils/electricity").unwrap();
        assert_eq!(name.as_str(), "utils/electricity");
    }

    #[test]
    fn test_valid_deep_hierarchy() {
        let name = CategoryName::new("a/b/c").unwrap();
        assert_eq!(name.as_str(), "a/b/c");
    }

    #[test]
    fn test_valid_with_hyphens() {
        let name = CategoryName::new("my-category").unwrap();
        assert_eq!(name.as_str(), "my-category");
    }

    #[test]
    fn test_valid_with_underscores() {
        let name = CategoryName::new("my_category").unwrap();
        assert_eq!(name.as_str(), "my_category");
    }

    #[test]
    fn test_valid_uppercase_and_digits() {
        let name = CategoryName::new("ABC123").unwrap();
        assert_eq!(name.as_str(), "ABC123");
    }

    #[test]
    fn test_invalid_empty() {
        assert!(CategoryName::new("").is_err());
    }

    #[test]
    fn test_invalid_leading_slash() {
        assert!(CategoryName::new("/leading").is_err());
    }

    #[test]
    fn test_invalid_trailing_slash() {
        assert!(CategoryName::new("trailing/").is_err());
    }

    #[test]
    fn test_invalid_double_slash() {
        assert!(CategoryName::new("a//b").is_err());
    }

    #[test]
    fn test_invalid_space_in_segment() {
        assert!(CategoryName::new("a/ /b").is_err());
    }

    #[test]
    fn test_invalid_space_within_segment() {
        assert!(CategoryName::new("a/b c").is_err());
    }

    #[test]
    fn test_invalid_special_chars() {
        assert!(CategoryName::new("special!").is_err());
    }

    #[test]
    fn test_invalid_ampersand() {
        assert!(CategoryName::new("food & drinks").is_err());
    }

    #[test]
    fn test_as_str_returns_inner() {
        let name = CategoryName::new("housing/rent").unwrap();
        assert_eq!(name.as_str(), "housing/rent");
    }
}
