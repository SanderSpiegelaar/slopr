//! ===== Module docs =====
use std::fs;

/// ======== Reads a file ========
pub fn read(path: &str) -> String {
    // --- not a divider
    fs::read_to_string(path).unwrap_or_default()
}
