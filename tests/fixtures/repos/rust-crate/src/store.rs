use std::fs;
use std::path::Path;

// This function saves the total to the given path
pub fn save_total(path: &Path, total: u32) {
    fs::write(path, total.to_string()).ok();
}
