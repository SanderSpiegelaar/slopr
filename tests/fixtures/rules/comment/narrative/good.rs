//! This module adds numbers.

/// This function adds two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    // Wrapping because the caller checks for overflow.
    a.wrapping_add(b)
}
