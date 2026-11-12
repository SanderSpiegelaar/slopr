// ======================== // expect: comment/section-divider
use std::fs;

/* === Helpers === */ // expect: comment/section-divider
pub fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}
// ________ // expect: comment/section-divider
