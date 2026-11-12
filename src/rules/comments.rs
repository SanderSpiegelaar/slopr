use std::sync::LazyLock;

use aho_corasick::AhoCorasick;
use regex::Regex;

use crate::config::RuleSettings;
use crate::finding::{Severity, Span};
use crate::rules::{Check, Hit, Rule};
use crate::source::{Lang, SourceFile};

#[path = "phrases.rs"]
mod phrases;

const LANGS: &[Lang] = &[
    Lang::TypeScript,
    Lang::Tsx,
    Lang::JavaScript,
    Lang::Python,
    Lang::Rust,
];

pub const RULES: &[Rule] = &[
    Rule {
        id: "comment/narrative",
        summary: "Comment narrates what the code does",
        default_severity: Severity::Warn,
        langs: LANGS,
        default_on: true,
        check: Check::File(narrative),
    },
    Rule {
        id: "comment/section-divider",
        summary: "Decorative section divider comment",
        default_severity: Severity::Warn,
        langs: LANGS,
        default_on: true,
        check: Check::File(section_divider),
    },
    Rule {
        id: "comment/step-numbered",
        summary: "Numbered step or phase header comment",
        default_severity: Severity::Warn,
        langs: LANGS,
        default_on: true,
        check: Check::File(step_numbered),
    },
    Rule {
        id: "comment/agent-artifact",
        summary: "Agent chatter about its plan, spec or past edits",
        default_severity: Severity::Error,
        langs: LANGS,
        default_on: true,
        check: Check::File(agent_artifact),
    },
    Rule {
        id: "comment/untracked-todo",
        summary: "TODO, FIXME, HACK or XXX without an issue reference",
        default_severity: Severity::Info,
        langs: LANGS,
        default_on: true,
        check: Check::File(untracked_todo),
    },
];

static NARRATIVE: LazyLock<AhoCorasick> = LazyLock::new(|| phrase_matcher(phrases::NARRATIVE));
static AGENT_ARTIFACT: LazyLock<AhoCorasick> =
    LazyLock::new(|| phrase_matcher(phrases::AGENT_ARTIFACT));
static DIVIDER_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[=\-*#~_]{3,}\s+.+?\s+[=\-*#~_]{3,}$").expect("valid divider regex")
});
static STEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(step|phase|stage)\s+\d+\b").expect("valid step regex"));
static TODO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(TODO|FIXME|HACK|XXX)\b").expect("valid todo regex"));

/// Plain comments only: doc comments are skipped.
fn comments(file: &SourceFile) -> impl Iterator<Item = (Span, &str)> {
    file.comments()
        .filter(|&(_, _, is_doc)| !is_doc)
        .map(|(span, text, _)| (span, text))
}

fn narrative(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for (span, text) in comments(file) {
        if let Some(phrase) = find_phrase(&NARRATIVE, text, true) {
            hits.push(Hit {
                span,
                message: format!("narrative comment opener `{phrase}`"),
            });
        }
    }
}

fn section_divider(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for (span, text) in comments(file) {
        if is_divider(text) {
            hits.push(Hit {
                span,
                message: "section divider comment".to_string(),
            });
        }
    }
}

fn step_numbered(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    for (span, text) in comments(file) {
        if let Some(step) = STEP.find(text) {
            hits.push(Hit {
                span,
                message: format!("step-numbered comment `{}`", step.as_str()),
            });
        }
    }
}

fn agent_artifact(file: &SourceFile, _: &RuleSettings, hits: &mut Vec<Hit>) {
    let strings = file
        .strings()
        .filter(|&(_, _, is_docstring)| !is_docstring)
        .map(|(span, text, _)| (span, text));
    for (span, text) in comments(file).chain(strings) {
        if let Some(phrase) = find_phrase(&AGENT_ARTIFACT, text, false) {
            hits.push(Hit {
                span,
                message: format!("agent artifact `{phrase}`"),
            });
        }
    }
}

fn untracked_todo(file: &SourceFile, settings: &RuleSettings, hits: &mut Vec<Hit>) {
    for (span, text) in comments(file) {
        if let Some(marker) = TODO.find(text)
            && !settings.todo_issue_pattern.is_match(text)
        {
            hits.push(Hit {
                span,
                message: format!("`{}` without an issue reference", marker.as_str()),
            });
        }
    }
}

/// `^([=\-*#~_])\1{3,}` without a backreference, which `regex` lacks, or a titled divider.
fn is_divider(text: &str) -> bool {
    let bytes = text.as_bytes();
    let repeated = bytes.len() >= 4
        && b"=-*#~_".contains(&bytes[0])
        && bytes[1..4].iter().all(|&b| b == bytes[0]);
    repeated || DIVIDER_TITLE.is_match(text)
}

fn phrase_matcher(patterns: &[&str]) -> AhoCorasick {
    let phrases: Vec<String> = patterns.iter().flat_map(|p| expand(p)).collect();
    AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(phrases)
        .expect("ported phrases build an automaton")
}

/// Expands each `{a|b|}` group into its alternatives. An empty alternative makes it optional.
fn expand(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };
    let close = open + pattern[open..].find('}').expect("phrase groups are closed");
    let (head, tail) = (&pattern[..open], &pattern[close + 1..]);
    pattern[open + 1..close]
        .split('|')
        .flat_map(|alt| expand(&format!("{head}{alt}{tail}")))
        .collect()
}

/// The first phrase on word boundaries in `text` with whitespace collapsed, lowercased.
/// Overlapping matches, so a longer phrase that fails a boundary cannot hide a shorter one.
fn find_phrase(matcher: &AhoCorasick, text: &str, at_start: bool) -> Option<String> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let is_word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    matcher
        .find_overlapping_iter(&text)
        .find(|m| {
            (!at_start || m.start() == 0)
                && !is_word(text[..m.start()].chars().next_back())
                && !is_word(text[m.end()..].chars().next())
        })
        .map(|m| text[m.range()].to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_groups_and_optional_parts() {
        assert_eq!(
            expand("as {per|requested} {the |}spec"),
            [
                "as per the spec",
                "as per spec",
                "as requested the spec",
                "as requested spec"
            ]
        );
    }

    #[test]
    fn find_phrase_checks_word_boundaries() {
        assert_eq!(
            find_phrase(&NARRATIVE, "This   function adds", true).as_deref(),
            Some("this function")
        );
        assert_eq!(find_phrase(&NARRATIVE, "This functionality", true), None);
        assert_eq!(find_phrase(&NARRATIVE, "Note: this function", true), None);
        assert_eq!(
            find_phrase(&AGENT_ARTIFACT, "Done, as requested.", false).as_deref(),
            Some("as requested")
        );
        assert_eq!(find_phrase(&AGENT_ARTIFACT, "has requested", false), None);
    }
}
