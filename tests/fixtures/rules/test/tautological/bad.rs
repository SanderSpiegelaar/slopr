pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert!(true); // expect: test/tautological
        assert!(true, "always {}", add(1, 2)); // expect: test/tautological
        assert_eq!(1, 1); // expect: test/tautological
        assert_eq!("a", "a", "same"); // expect: test/tautological
        assert_eq!(add(1, 2), 3);
    }
}
