use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Counter {
    counts: BTreeMap<String, u32>,
}

impl Counter {
    pub fn add(&mut self, word: &str) {
        *self.counts.entry(word.to_owned()).or_default() += 1;
    }

    pub fn get(&self, word: &str) -> u32 {
        self.counts.get(word).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u32 {
        self.counts.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_repeated_words() {
        let mut counter = Counter::default();
        counter.add("a");
        counter.add("a");
        assert_eq!(counter.get("a"), 2);
    }
}
