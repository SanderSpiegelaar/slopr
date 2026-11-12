use std::sync::LazyLock;

use regex::Regex;

use crate::config::RuleSettings;
use crate::finding::Severity;
use crate::rules::{Check, Hit, Rule};
use crate::source::SourceFile;

pub const RULES: &[Rule] = &[
    Rule {
        id: "text/invisible-unicode",
        summary: "Zero-width or bidi control character",
        default_severity: Severity::Error,
        langs: &[],
        default_on: true,
        check: Check::File(invisible_unicode),
    },
    Rule {
        id: "sec/hardcoded-secret",
        summary: "Hardcoded API key, token or private key",
        default_severity: Severity::Error,
        langs: &[],
        default_on: true,
        check: Check::File(hardcoded_secret),
    },
];

// Code points from `ZERO_WIDTH`, `ZWJ_CHAR`, `FEFF_CHAR` and `BIDI` in WhiteBite/stop-ai-slop
// skill/scripts/src/markers.mjs at d2c586881bf4ead6ab2865dde90f45f247d58972 (MIT).
fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200D}'
            | '\u{2060}'
            | '\u{FEFF}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn invisible_unicode(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let mut line_start = 0;
    for line in file.text.split_inclusive('\n') {
        let mut first = None;
        let mut found: Vec<char> = Vec::new();
        for (i, c) in line.char_indices() {
            let at = line_start + i;
            // A U+FEFF at offset 0 is a byte order mark.
            if !is_invisible(c) || (at == 0 && c == '\u{FEFF}') {
                continue;
            }
            first.get_or_insert(at..at + c.len_utf8());
            if !found.contains(&c) {
                found.push(c);
            }
        }
        if let Some(range) = first {
            let names: Vec<String> = found
                .iter()
                .map(|&c| format!("U+{:04X}", c as u32))
                .collect();
            hits.push(Hit {
                span: file.span(range),
                message: format!("invisible character {}", names.join(", ")),
            });
        }
        line_start += line.len();
    }
}

static SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"sk_live_[0-9a-zA-Z]{20,}",
        r"|sk-(proj-)?[A-Za-z0-9_-]{32,}",
        r"|ghp_[A-Za-z0-9]{36}",
        r"|github_pat_[A-Za-z0-9_]{82}",
        r"|AKIA[0-9A-Z]{16}",
        r"|glpat-[A-Za-z0-9_-]{20}",
        r"|xox[bp]-[0-9A-Za-z-]{10,}",
        r"|-----BEGIN ([A-Z]+ )?PRIVATE KEY-----",
    ))
    .expect("secret patterns are a valid regex")
});

fn hardcoded_secret(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for m in SECRET.find_iter(&file.text) {
        // Every pattern is ASCII, so byte 4 is a char boundary.
        let shown = &m.as_str()[..4];
        hits.push(Hit {
            span: file.span(m.range()),
            message: format!("hardcoded secret {shown}... ({} chars)", m.len()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(check: fn(&SourceFile, &RuleSettings, &mut Vec<Hit>), text: &str) -> Vec<Hit> {
        let file = SourceFile::new("f.txt".to_string(), text.to_string(), None, false);
        let mut hits = Vec::new();
        check(&file, &RuleSettings::default(), &mut hits);
        hits
    }

    #[test]
    fn secret_message_never_contains_the_full_match() {
        let text = "a = \"AKIAIOSFODNN7EXAMPLE\"\n-----BEGIN PRIVATE KEY-----\n";
        let hits = run(hardcoded_secret, text);
        assert_eq!(hits.len(), 2);
        for (hit, secret) in hits
            .iter()
            .zip(["AKIAIOSFODNN7EXAMPLE", "-----BEGIN PRIVATE KEY-----"])
        {
            assert!(!hit.message.contains(secret), "{}", hit.message);
        }
        assert_eq!(hits[0].message, "hardcoded secret AKIA... (20 chars)");
    }

    #[test]
    fn bom_only_exempt_at_offset_zero() {
        assert!(run(invisible_unicode, "\u{FEFF}let a = 1;\n").is_empty());
        let hits = run(
            invisible_unicode,
            "let a = 1;\nlet b\u{FEFF} = 2;\u{200B}\u{FEFF}\n",
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].span.start_line, 2);
        assert_eq!(hits[0].span.start_col, 6);
        assert_eq!(hits[0].message, "invisible character U+FEFF, U+200B");
    }
}
