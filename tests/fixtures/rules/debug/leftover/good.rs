pub fn report(total: i32) {
    println!("total: {total}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn total_is_printed() {
        dbg!(super::report(1));
    }
}
