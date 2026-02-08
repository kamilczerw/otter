use serde::{Deserialize, Serialize};
use std::iter::Sum;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money::new(0), |acc, m| acc + m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_new_and_value() {
        let m = Money::new(1500);
        assert_eq!(m.value(), 1500);
    }

    #[test]
    fn test_money_positive() {
        let m = Money::new(42);
        assert_eq!(m.value(), 42);
    }

    #[test]
    fn test_money_negative() {
        let m = Money::new(-500);
        assert_eq!(m.value(), -500);
    }

    #[test]
    fn test_money_zero() {
        let m = Money::new(0);
        assert_eq!(m.value(), 0);
    }

    #[test]
    fn test_money_add() {
        let a = Money::new(100);
        let b = Money::new(250);
        assert_eq!((a + b).value(), 350);
    }

    #[test]
    fn test_money_add_negative() {
        let a = Money::new(100);
        let b = Money::new(-30);
        assert_eq!((a + b).value(), 70);
    }

    #[test]
    fn test_money_sub() {
        let a = Money::new(300);
        let b = Money::new(100);
        assert_eq!((a - b).value(), 200);
    }

    #[test]
    fn test_money_sub_negative_result() {
        let a = Money::new(50);
        let b = Money::new(100);
        assert_eq!((a - b).value(), -50);
    }

    #[test]
    fn test_money_sum() {
        let amounts = vec![Money::new(100), Money::new(200), Money::new(300)];
        let total: Money = amounts.into_iter().sum();
        assert_eq!(total.value(), 600);
    }

    #[test]
    fn test_money_empty_sum() {
        let amounts: Vec<Money> = vec![];
        let total: Money = amounts.into_iter().sum();
        assert_eq!(total.value(), 0);
    }

    #[test]
    fn test_money_sum_with_negatives() {
        let amounts = vec![Money::new(100), Money::new(-30), Money::new(50)];
        let total: Money = amounts.into_iter().sum();
        assert_eq!(total.value(), 120);
    }

    #[test]
    fn test_money_serde_roundtrip() {
        let m = Money::new(999);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
        assert_eq!(deserialized.value(), 999);
    }

    #[test]
    fn test_money_serde_roundtrip_negative() {
        let m = Money::new(-250);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_money_serde_roundtrip_zero() {
        let m = Money::new(0);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }
}
