use std::fmt::Write;

use crate::finding::Finding;
use crate::report::{BySeverity, Summary};

/// One `<path>:<line>:<col> <severity> <rule> <message>` line per finding, files separated by a
/// blank line, then per-rule counts and a total. Expects findings in report order.
pub fn render_human(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "0 findings\n".to_string();
    }
    let mut out = String::new();
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
    }
    out.push('\n');
    let summary = Summary::of(findings);
    for (rule, count) in &summary.by_rule {
        writeln!(out, "{count} {rule}").expect("writing to a String");
    }
    let BySeverity { error, warn, info } = summary.by_severity;
    writeln!(
        out,
        "{} findings ({error} error, {warn} warn, {info} info)",
        summary.total
    )
    .expect("writing to a String");
    out
}
