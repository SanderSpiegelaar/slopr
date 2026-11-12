use std::collections::BTreeMap;
use std::fmt::Write;

use crate::finding::{Finding, Severity};

/// One `<path>:<line>:<col> <severity> <rule> <message>` line per finding, files separated by a
/// blank line, then per-rule counts and a total. Expects findings in report order.
pub fn render_human(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "0 findings\n".to_string();
    }
    let mut out = String::new();
    let mut by_rule: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_severity: BTreeMap<Severity, usize> = BTreeMap::new();
    for (i, f) in findings.iter().enumerate() {
        if i > 0 && findings[i - 1].path != f.path {
            out.push('\n');
        }
        let (line, col) = (f.span.start_line, f.span.start_col);
        writeln!(
            out,
            "{}:{line}:{col} {} {} {}",
            f.path, f.severity, f.rule, f.message
        )
        .expect("writing to a String");
        *by_rule.entry(&f.rule).or_default() += 1;
        *by_severity.entry(f.severity).or_default() += 1;
    }
    out.push('\n');
    for (rule, count) in by_rule {
        writeln!(out, "{count} {rule}").expect("writing to a String");
    }
    let count = |s| by_severity.get(&s).copied().unwrap_or(0);
    writeln!(
        out,
        "{} findings ({} error, {} warn, {} info)",
        findings.len(),
        count(Severity::Error),
        count(Severity::Warn),
        count(Severity::Info)
    )
    .expect("writing to a String");
    out
}
