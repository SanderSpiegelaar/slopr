pub fn add(a: i32, b: i32) -> i32 {
    assert!(true);
    assert_eq!(1, 1);
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert!(add(1, 1) == 2);
        assert_eq!(add(1, 2), 3);
        assert_eq!(-1, 1);
        assert_eq!(1, 2);
        assert!(false || true);
    }
}
