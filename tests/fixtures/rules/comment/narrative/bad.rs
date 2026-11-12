// This function adds two numbers // expect: comment/narrative
pub fn add(a: i32, b: i32) -> i32 {
    // It returns the sum // expect: comment/narrative
    a + b
}

// Finally the function is exported // expect: comment/narrative
pub use add as sum;
