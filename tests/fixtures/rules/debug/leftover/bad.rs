pub fn total(items: &[i32]) -> i32 {
    let sum = dbg!(items.iter().sum()); // expect: debug/leftover
    std::dbg!(sum); // expect: debug/leftover
    sum
}
