use crate::errors::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DueDay(u8);

impl DueDay {
    pub fn new(value: u8) -> Result<Self, DomainError> {
        if !(1..=31).contains(&value) {
            return Err(DomainError::InvalidDueDay { value });
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_due_day_1() {
        let d = DueDay::new(1).unwrap();
        assert_eq!(d.value(), 1);
    }

    #[test]
    fn test_valid_due_day_15() {
        let d = DueDay::new(15).unwrap();
        assert_eq!(d.value(), 15);
    }

    #[test]
    fn test_valid_due_day_31() {
        let d = DueDay::new(31).unwrap();
        assert_eq!(d.value(), 31);
    }

    #[test]
    fn test_invalid_due_day_zero() {
        assert!(DueDay::new(0).is_err());
    }

    #[test]
    fn test_invalid_due_day_32() {
        assert!(DueDay::new(32).is_err());
    }

    #[test]
    fn test_value_returns_inner() {
        for v in [1, 10, 20, 31] {
            let d = DueDay::new(v).unwrap();
            assert_eq!(d.value(), v);
        }
    }
}
