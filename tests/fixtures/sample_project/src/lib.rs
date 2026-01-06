pub mod calculator;

pub use calculator::{Adder, Calculator};

/// Adds two numbers together.
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The sum of a and b
///
/// # Examples
///
/// ```
/// let result = sample_project::add(5, 10);
/// assert_eq!(result, 15);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtracts b from a.
///
/// # Arguments
///
/// * `a` - The number to subtract from
/// * `b` - The number to subtract
///
/// # Returns
///
/// The difference a - b
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiplies two numbers.
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// A simple struct to demonstrate type definitions.
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Creates a new Point.
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    /// Returns the distance from origin.
    pub fn distance_from_origin(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}
