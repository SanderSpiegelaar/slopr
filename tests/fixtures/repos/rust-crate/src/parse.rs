use crate::Counter;

pub fn count_words(text: &str) -> Counter {
    let mut counter = Counter::default();
    for word in text.split_whitespace() {
        counter.add(word);
    }
    dbg!(counter.total());
    counter
}

pub fn count_lines(text: &str) -> Counter {
    todo!()
}
