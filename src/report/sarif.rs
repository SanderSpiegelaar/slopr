use std::collections::BTreeSet;

use serde::Serialize;

use crate::fallow::Outcome;
use crate::finding::{Finding, Origin, Severity, Span};
use crate::report::native_rules;
use crate::suppress;

#[derive(Serialize)]
struct Log<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<Run<'a>>,
}

#[derive(Serialize)]
struct Run<'a> {
    tool: Tool<'a>,
    results: Vec<SarifResult<'a>>,
}

#[derive(Serialize)]
struct Tool<'a> {
    driver: Driver<'a>,
}

#[derive(Serialize)]
struct Driver<'a> {
    name: &'static str,
    version: &'a str,
    rules: Vec<RuleDescriptor<'a>>,
}

/// fallow's rules carry only an `id`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleDescriptor<'a> {
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_description: Option<Text<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_configuration: Option<Configuration>,
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

/// A SARIF 2.1.0 log with slopr's run, then fallow's run when fallow ran, pretty-printed with a
/// trailing newline. Expects findings in report order.
pub fn render_sarif(findings: &[Finding], fallow: Outcome<'_>) -> String {
    let (native, from_fallow): (Vec<&Finding>, Vec<&Finding>) = findings
        .iter()
        .partition(|f| matches!(f.origin, Origin::Native));
    let mut rules: Vec<RuleDescriptor> = native_rules()
        .into_iter()
        .map(|rule| RuleDescriptor {
            id: rule.id,
            short_description: Some(Text { text: rule.summary }),
            default_configuration: Some(Configuration {
                level: rule.default_severity.into(),
            }),
        })
        .collect();
    rules.push(RuleDescriptor {
        id: suppress::INVALID_DIRECTIVE,
        short_description: Some(Text {
            text: suppress::INVALID_DIRECTIVE_SUMMARY,
        }),
        default_configuration: Some(Configuration {
            level: Level::Warning,
        }),
    });
    rules.sort_unstable_by_key(|rule| rule.id);
    let mut runs = vec![Run {
        tool: Tool {
            driver: Driver {
                name: "slopr",
                version: env!("CARGO_PKG_VERSION"),
                rules,
            },
        },
        results: native.into_iter().map(result).collect(),
    }];
    if let Outcome::Ok(version) = fallow {
        let ids: BTreeSet<&str> = from_fallow.iter().map(|f| f.rule.as_str()).collect();
        runs.push(Run {
            tool: Tool {
                driver: Driver {
                    name: "fallow",
                    version,
                    rules: ids
                        .into_iter()
                        .map(|id| RuleDescriptor {
                            id,
                            short_description: None,
                            default_configuration: None,
                        })
                        .collect(),
                },
            },
            results: from_fallow.into_iter().map(result).collect(),
        });
    }
    let log = Log {
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        version: "2.1.0",
        runs,
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
