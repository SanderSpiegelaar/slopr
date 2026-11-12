use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use rayon::prelude::*;
use regex::Regex;

use crate::config;
use crate::finding::{self, Finding, Origin, Severity, Span};
use crate::source::SourceFile;

pub const INVALID_DIRECTIVE: &str = "suppress/invalid-directive";
pub const INVALID_DIRECTIVE_SUMMARY: &str =
    "Suppression directive that is malformed, has no reason or names an unknown rule";

const MARKER: &str = "slopr-ignore-";

static DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^slopr-ignore-(next-line|file)\s+([a-z0-9-]+/[a-z0-9-]+(?:\s*,\s*[a-z0-9-]+/[a-z0-9-]+)*)(?:\s+--(?:\s+(.*))?)?$",
    )
    .expect("valid directive regex")
});

enum Scope {
    NextLine,
    File,
}

/// A line that starts with `slopr-ignore-` after marker stripping.
struct Candidate<'a> {
    text: &'a str,
    span: Span,
    next_line: Option<u32>,
}

/// What one file's valid directives hide.
#[derive(Default)]
struct Suppressions<'a> {
    file: Vec<&'a str>,
    lines: Vec<(u32, &'a str)>,
}

impl Suppressions<'_> {
    fn hides(&self, rule: &str, line: u32) -> bool {
        self.file.contains(&rule) || self.lines.contains(&(line, rule))
    }
}

enum Invalid<'a> {
    Malformed,
    EmptyReason,
    UnknownRule(&'a str),
}

impl fmt::Display for Invalid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invalid::Malformed => f.write_str(
                "malformed directive, expected `slopr-ignore-next-line|file <rule-id> -- <reason>`",
            ),
            Invalid::EmptyReason => f.write_str("directive needs a reason after ` -- `"),
            Invalid::UnknownRule(id) => write!(f, "directive names unknown rule `{id}`"),
        }
    }
}

/// Drops findings hidden by valid directives, then appends one warn finding per invalid directive.
pub fn apply(files: &[SourceFile], findings: &mut Vec<Finding>) {
    let (suppressions, invalid): (HashMap<&str, Suppressions<'_>>, Vec<Vec<Finding>>) = files
        .par_iter()
        .filter(|file| file.text.contains(MARKER))
        .map(|file| {
            let (hidden, invalid) = scan(file);
            ((file.path.as_str(), hidden), invalid)
        })
        .unzip();
    findings.retain(|f| {
        !suppressions
            .get(f.path.as_str())
            .is_some_and(|s| s.hides(&f.rule, f.span.start_line))
    });
    findings.extend(invalid.into_iter().flatten());
}

fn scan(file: &SourceFile) -> (Suppressions<'_>, Vec<Finding>) {
    let mut hidden = Suppressions::default();
    let mut invalid = Vec::new();
    let mut occurrences: HashMap<String, usize> = HashMap::new();
    for candidate in candidates(file) {
        match parse(candidate.text) {
            Ok((Scope::File, ids)) => hidden.file.extend(ids),
            Ok((Scope::NextLine, ids)) => {
                if let Some(line) = candidate.next_line {
                    hidden.lines.extend(ids.into_iter().map(|id| (line, id)));
                }
            }
            Err(why) => {
                let snippet = finding::snippet(&file.text, candidate.span);
                let seen = occurrences.entry(snippet.clone()).or_default();
                let fingerprint =
                    finding::fingerprint(INVALID_DIRECTIVE, &file.path, &snippet, *seen);
                *seen += 1;
                invalid.push(Finding {
                    rule: INVALID_DIRECTIVE.to_string(),
                    severity: Severity::Warn,
                    path: file.path.clone(),
                    span: candidate.span,
                    message: why.to_string(),
                    fingerprint,
                    origin: Origin::Native,
                });
            }
        }
    }
    (hidden, invalid)
}

fn candidates(file: &SourceFile) -> Vec<Candidate<'_>> {
    if file.parsed.is_some() {
        file.comments()
            .flat_map(|(comment, text, _)| {
                text.lines().filter_map(move |line| {
                    let line = line.trim_start();
                    let at = offset(file, line);
                    // Only continuation lines of a block comment carry a leading `*` marker.
                    let line = match line.strip_prefix('*') {
                        Some(rest) if file.span(at..at).start_line > comment.start_line => {
                            rest.trim_start()
                        }
                        _ => line,
                    };
                    candidate(file, line, comment.end_line)
                })
            })
            .collect()
    } else {
        (1..)
            .zip(file.text.lines())
            .filter_map(|(n, line)| candidate(file, strip_line_marker(line), n))
            .collect()
    }
}

fn strip_line_marker(line: &str) -> &str {
    let line = line.trim_start();
    let line = match line.strip_prefix("<!--") {
        Some(inner) => {
            let inner = inner.trim_end();
            inner.strip_suffix("-->").unwrap_or(inner)
        }
        None => ["#", "//", "--", ";"]
            .iter()
            .find_map(|marker| line.strip_prefix(marker))
            .unwrap_or(line),
    };
    line.trim_start()
}

/// `text` must borrow from `file.text`; `after_line` is where `next-line` starts looking.
fn candidate<'a>(file: &'a SourceFile, text: &'a str, after_line: u32) -> Option<Candidate<'a>> {
    let text = text.trim_end();
    if !text.starts_with(MARKER) {
        return None;
    }
    let start = offset(file, text);
    let next_line = (1..)
        .zip(file.text.lines())
        .find(|&(n, line)| n > after_line && !line.trim().is_empty())
        .map(|(n, _)| n);
    Some(Candidate {
        text,
        span: file.span(start..start + text.len()),
        next_line,
    })
}

/// Byte offset of `s` in `file.text`. `s` must borrow from `file.text`.
fn offset(file: &SourceFile, s: &str) -> usize {
    s.as_ptr().addr() - file.text.as_ptr().addr()
}

fn parse(text: &str) -> Result<(Scope, Vec<&str>), Invalid<'_>> {
    let caps = DIRECTIVE.captures(text).ok_or(Invalid::Malformed)?;
    if caps
        .get(3)
        .is_none_or(|reason| reason.as_str().trim().is_empty())
    {
        return Err(Invalid::EmptyReason);
    }
    let ids: Vec<&str> = caps
        .get(2)
        .expect("the id group is not optional")
        .as_str()
        .split(',')
        .map(str::trim)
        .collect();
    if let Some(id) = ids
        .iter()
        .find(|id| config::native_rule(id).is_none() && !id.starts_with("fallow/"))
    {
        return Err(Invalid::UnknownRule(id));
    }
    let scope = if &caps[1] == "file" {
        Scope::File
    } else {
        Scope::NextLine
    };
    Ok((scope, ids))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(line: u32) -> Finding {
        Finding {
            rule: "text/invisible-unicode".to_string(),
            severity: Severity::Error,
            path: "a.txt".to_string(),
            span: Span {
                start_line: line,
                start_col: 1,
                end_line: line,
                end_col: 2,
            },
            message: String::new(),
            fingerprint: String::new(),
            origin: Origin::Native,
        }
    }

    #[test]
    fn text_file_hash_directive_hides_next_non_blank_line() {
        let text = "  # slopr-ignore-next-line text/invisible-unicode -- fixture\n\nx\ny\n";
        let file = SourceFile::new("a.txt".to_string(), text.to_string(), None, false);
        let mut findings = vec![finding(3), finding(4)];
        apply(&[file], &mut findings);
        let lines: Vec<u32> = findings.iter().map(|f| f.span.start_line).collect();
        assert_eq!(lines, [4]);
    }

    #[test]
    fn reason_needs_space_after_separator() {
        let file = "slopr-ignore-file ts/unsafe-assertion";
        assert!(matches!(
            parse(&format!("{file} --because")),
            Err(Invalid::Malformed)
        ));
        assert!(matches!(
            parse(&format!("{file} -- because")),
            Ok((Scope::File, _))
        ));
    }
}
