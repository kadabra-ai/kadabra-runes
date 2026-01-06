/// A trait for implementing different calculation strategies.
pub trait Calculator {
    /// Performs a calculation on two numbers.
    fn calculate(&self, a: i32, b: i32) -> i32;
}

/// A calculator that adds two numbers.
pub struct Adder;

impl Calculator for Adder {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

/// A calculator that multiplies two numbers.
pub struct Multiplier;

impl Calculator for Multiplier {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

/// A calculator that subtracts the second number from the first.
pub struct Subtractor;

impl Calculator for Subtractor {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a - b
    }
}

/// Performs calculation using any calculator implementation.
pub fn perform_calculation(calc: &dyn Calculator, a: i32, b: i32) -> i32 {
    calc.calculate(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder() {
        let adder = Adder;
        assert_eq!(adder.calculate(5, 3), 8);
    }

    #[test]
    fn test_multiplier() {
        let multiplier = Multiplier;
        assert_eq!(multiplier.calculate(5, 3), 15);
    }

    #[test]
    fn test_subtractor() {
        let subtractor = Subtractor;
        assert_eq!(subtractor.calculate(5, 3), 2);
    }
}
