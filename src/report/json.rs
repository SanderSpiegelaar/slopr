use serde::Serialize;

use crate::fallow::Outcome;
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
    tools: [Tool<'a>; 1],
}

#[derive(Serialize)]
struct Tool<'a> {
    name: &'static str,
    version: Option<&'a str>,
    status: &'static str,
}

/// The json report, pretty-printed with a trailing newline. Expects findings in report order.
pub fn render_json(findings: &[Finding], fallow: Outcome<'_>) -> String {
    let (version, status) = match fallow {
        Outcome::Skipped => (None, "skipped"),
        Outcome::Ok(version) => (Some(version), "ok"),
        Outcome::Failed(version) => (Some(version), "failed"),
    };
    let report = Report {
        schema_version: SCHEMA_VERSION,
        version: env!("CARGO_PKG_VERSION"),
        findings,
        summary: Summary::of(findings),
        tools: [Tool {
            name: "fallow",
            version,
            status,
        }],
    };
    let mut out = serde_json::to_string_pretty(&report).expect("the report serializes");
    out.push('\n');
    out
}
