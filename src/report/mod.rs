use std::collections::BTreeMap;
use std::fmt::Write;

use serde::Serialize;

use crate::finding::{Finding, Severity};
use crate::rules::{FAMILIES, Rule};

mod human;
mod json;
mod sarif;

pub use human::render_human;
pub use json::render_json;
pub use sarif::render_sarif;

#[derive(Serialize)]
struct Summary<'a> {
    total: usize,
    by_severity: BySeverity,
    by_rule: BTreeMap<&'a str, usize>,
}

#[derive(Default, Serialize)]
struct BySeverity {
    error: usize,
    warn: usize,
    info: usize,
}

impl<'a> Summary<'a> {
    fn of(findings: &'a [Finding]) -> Self {
        let mut by_severity = BySeverity::default();
        let mut by_rule = BTreeMap::new();
        for f in findings {
            *match f.severity {
                Severity::Error => &mut by_severity.error,
                Severity::Warn => &mut by_severity.warn,
                Severity::Info => &mut by_severity.info,
            } += 1;
            *by_rule.entry(f.rule.as_str()).or_default() += 1;
        }
        Summary {
            total: findings.len(),
            by_severity,
            by_rule,
        }
    }
}

/// One `<id> <default severity> <on|off> <summary>` line per native rule.
pub fn render_rules() -> String {
    let mut out = String::new();
    for rule in native_rules() {
        let state = if rule.default_on { "on" } else { "off" };
        writeln!(
            out,
            "{} {} {state} {}",
            rule.id, rule.default_severity, rule.summary
        )
        .expect("writing to a String");
    }
    out
}

/// Every native rule, sorted by id.
fn native_rules() -> Vec<&'static Rule> {
    let mut rules: Vec<_> = FAMILIES.iter().copied().flatten().collect();
    rules.sort_unstable_by_key(|rule| rule.id);
    rules
}
