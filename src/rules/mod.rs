use std::collections::HashMap;

use rayon::prelude::*;

use crate::config::RuleSettings;
use crate::finding::{self, Finding, Origin, Severity, Span};
use crate::source::{Lang, SourceFile};

mod comments;
mod debug;
mod errors;
mod stubs;
mod tests;
mod text;
mod types;

pub struct Hit {
    pub span: Span,
    pub message: String,
}

pub enum Check {
    File(fn(&SourceFile, &RuleSettings, &mut Vec<Hit>)),
}

pub struct Rule {
    pub id: &'static str,
    #[allow(dead_code, reason = "read by the rules and sarif output tickets")]
    pub summary: &'static str,
    pub default_severity: Severity,
    /// Empty means every file, including unparsed text.
    pub langs: &'static [Lang],
    pub default_on: bool,
    pub check: Check,
}

pub static FAMILIES: &[&[Rule]] = &[
    comments::RULES,
    errors::RULES,
    stubs::RULES,
    tests::RULES,
    debug::RULES,
    text::RULES,
    types::RULES,
];

/// Runs every enabled rule on every file, in report order with fingerprints.
pub fn run_rules(files: &[SourceFile], settings: &RuleSettings) -> Vec<Finding> {
    let mut hits: Vec<(&SourceFile, &Rule, Hit)> = files
        .par_iter()
        .flat_map_iter(|file| file_hits(file, settings))
        .collect();
    hits.sort_by(|a, b| report_order(a).cmp(&report_order(b)));

    let mut occurrences: HashMap<(&str, &str, String), usize> = HashMap::new();
    hits.into_iter()
        .map(|(file, rule, hit)| {
            let snippet = finding::snippet(&file.text, hit.span);
            let seen = occurrences
                .entry((&file.path, rule.id, snippet.clone()))
                .or_default();
            let fingerprint = finding::fingerprint(rule.id, &file.path, &snippet, *seen);
            *seen += 1;
            Finding {
                rule: rule.id.to_string(),
                severity: rule.default_severity,
                path: file.path.clone(),
                span: hit.span,
                message: hit.message,
                fingerprint,
                origin: Origin::Native,
            }
        })
        .collect()
}

fn file_hits<'a>(
    file: &'a SourceFile,
    settings: &RuleSettings,
) -> Vec<(&'a SourceFile, &'static Rule, Hit)> {
    let mut out = Vec::new();
    for rule in FAMILIES.iter().copied().flatten() {
        if !rule.default_on || !applies(rule, file) {
            continue;
        }
        let mut hits = Vec::new();
        match rule.check {
            Check::File(check) => check(file, settings, &mut hits),
        }
        out.extend(hits.into_iter().map(|hit| (file, rule, hit)));
    }
    out
}

fn report_order<'a>(
    (file, rule, hit): &'a (&SourceFile, &Rule, Hit),
) -> (&'a [u8], u32, u32, &'a str, &'a str) {
    (
        file.path.as_bytes(),
        hit.span.start_line,
        hit.span.start_col,
        rule.id,
        &hit.message,
    )
}

fn applies(rule: &Rule, file: &SourceFile) -> bool {
    rule.langs.is_empty()
        || file
            .parsed
            .as_ref()
            .is_some_and(|p| rule.langs.contains(&p.lang))
}
