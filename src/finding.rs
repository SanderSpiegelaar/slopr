use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Error => "error",
        })
    }
}

/// 1-based lines and columns, columns in Unicode scalar values. `end_line` is inclusive and
/// `end_col` exclusive, as in SARIF.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Span {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Origin {
    Native,
    #[allow(dead_code, reason = "built by the fallow tickets")]
    Fallow {
        version: String,
        command: String,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub rule: String,
    pub severity: Severity,
    /// Root-relative, forward slashes.
    pub path: String,
    pub span: Span,
    pub message: String,
    pub fingerprint: String,
    pub origin: Origin,
}

/// The flagged lines, each trimmed with inner whitespace runs collapsed, joined with `\n`.
pub fn snippet(text: &str, span: Span) -> String {
    let skip = span.start_line as usize - 1;
    let take = (span.end_line - span.start_line) as usize + 1;
    text.lines()
        .skip(skip)
        .take(take)
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// FNV-1a 64 over `rule \0 path \0 snippet \0 occurrence`, as 16 lowercase hex chars.
/// `occurrence` counts earlier findings in the same file with the same rule and snippet.
pub fn fingerprint(rule: &str, path: &str, snippet: &str, occurrence: usize) -> String {
    let input = [rule, path, snippet, &occurrence.to_string()].join("\0");
    format!("{:016x}", fnv1a64(input.as_bytes()))
}

/// Std-only and stable across Rust releases, unlike `DefaultHasher`.
fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_matches_reference_vector() {
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn fingerprint_separates_parts_with_nul() {
        let want = format!("{:016x}", fnv1a64(b"r\0p\0s\x000"));
        assert_eq!(fingerprint("r", "p", "s", 0), want);
    }

    #[test]
    fn snippet_normalizes_whitespace() {
        let span = Span {
            start_line: 2,
            start_col: 1,
            end_line: 3,
            end_col: 1,
        };
        assert_eq!(snippet("x\n  a   b \n\tc\nd", span), "a b\nc");
    }
}
