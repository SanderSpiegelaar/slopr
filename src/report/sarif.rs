use serde::Serialize;

use crate::finding::{Finding, Severity, Span};
use crate::report::native_rules;
use crate::suppress;

#[derive(Serialize)]
struct Log<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: [Run<'a>; 1],
}

#[derive(Serialize)]
struct Run<'a> {
    tool: Tool,
    results: Vec<SarifResult<'a>>,
}

#[derive(Serialize)]
struct Tool {
    driver: Driver,
}

#[derive(Serialize)]
struct Driver {
    name: &'static str,
    version: &'static str,
    rules: Vec<RuleDescriptor>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleDescriptor {
    id: &'static str,
    short_description: Text<'static>,
    default_configuration: Configuration,
}

#[derive(Serialize)]
struct Configuration {
    level: Level,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Level {
    Error,
    Warning,
    Note,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult<'a> {
    rule_id: &'a str,
    level: Level,
    message: Text<'a>,
    locations: [Location<'a>; 1],
    partial_fingerprints: Fingerprints<'a>,
}

#[derive(Serialize)]
struct Text<'a> {
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Location<'a> {
    physical_location: PhysicalLocation<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PhysicalLocation<'a> {
    artifact_location: ArtifactLocation<'a>,
    region: Region,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactLocation<'a> {
    uri: &'a str,
    uri_base_id: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Region {
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

#[derive(Serialize)]
struct Fingerprints<'a> {
    #[serde(rename = "slopr/v1")]
    slopr_v1: &'a str,
}

impl From<Severity> for Level {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Error => Level::Error,
            Severity::Warn => Level::Warning,
            Severity::Info => Level::Note,
        }
    }
}

impl From<Span> for Region {
    fn from(span: Span) -> Self {
        Region {
            start_line: span.start_line,
            start_column: span.start_col,
            end_line: span.end_line,
            end_column: span.end_col,
        }
    }
}

/// A SARIF 2.1.0 log with slopr's run, pretty-printed with a trailing newline.
pub fn render_sarif(findings: &[Finding]) -> String {
    let mut rules: Vec<RuleDescriptor> = native_rules()
        .into_iter()
        .map(|rule| RuleDescriptor {
            id: rule.id,
            short_description: Text { text: rule.summary },
            default_configuration: Configuration {
                level: rule.default_severity.into(),
            },
        })
        .collect();
    rules.push(RuleDescriptor {
        id: suppress::INVALID_DIRECTIVE,
        short_description: Text {
            text: suppress::INVALID_DIRECTIVE_SUMMARY,
        },
        default_configuration: Configuration {
            level: Level::Warning,
        },
    });
    rules.sort_unstable_by_key(|rule| rule.id);
    let results = findings.iter().map(result).collect();
    let log = Log {
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        version: "2.1.0",
        runs: [Run {
            tool: Tool {
                driver: Driver {
                    name: "slopr",
                    version: env!("CARGO_PKG_VERSION"),
                    rules,
                },
            },
            results,
        }],
    };
    let mut out = serde_json::to_string_pretty(&log).expect("the SARIF log serializes");
    out.push('\n');
    out
}

fn result(finding: &Finding) -> SarifResult<'_> {
    SarifResult {
        rule_id: &finding.rule,
        level: finding.severity.into(),
        message: Text {
            text: &finding.message,
        },
        locations: [Location {
            physical_location: PhysicalLocation {
                artifact_location: ArtifactLocation {
                    uri: &finding.path,
                    uri_base_id: "%SRCROOT%",
                },
                region: finding.span.into(),
            },
        }],
        partial_fingerprints: Fingerprints {
            slopr_v1: &finding.fingerprint,
        },
    }
}
