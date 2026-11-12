use serde::Serialize;

use crate::finding::Finding;
use crate::report::Summary;

/// Bump on any rename or removal.
const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct Report<'a> {
    schema_version: u32,
    version: &'static str,
    findings: &'a [Finding],
    summary: Summary<'a>,
    tools: [Tool; 1],
}

#[derive(Serialize)]
struct Tool {
    name: &'static str,
    version: Option<String>,
    status: &'static str,
}

/// The json report, pretty-printed with a trailing newline. Expects findings in report order.
pub fn render_json(findings: &[Finding]) -> String {
    let report = Report {
        schema_version: SCHEMA_VERSION,
        version: env!("CARGO_PKG_VERSION"),
        findings,
        summary: Summary::of(findings),
        tools: [Tool {
            name: "fallow",
            version: None,
            status: "skipped",
        }],
    };
    let mut out = serde_json::to_string_pretty(&report).expect("the report serializes");
    out.push('\n');
    out
}
