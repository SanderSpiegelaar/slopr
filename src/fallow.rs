use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use ignore::WalkBuilder;
use semver::{Version, VersionReq};
use serde_json::Value;

use crate::config::{Fallow, FallowCommand, FallowMode};
use crate::finding::{self, Finding, Origin, Severity, Span};

/// fallow is required but missing or outside the configured range.
#[derive(Debug, thiserror::Error)]
#[error(
    "fallow {} does not satisfy {required}. Install it with: npm install -D fallow@^3.42 (or cargo install fallow-cli --version ^3.42). Use --fallow off to skip.",
    found.as_deref().unwrap_or("not found")
)]
pub struct Unsatisfied {
    found: Option<String>,
    required: VersionReq,
}

/// A fallow process that could not start, exited 2 or higher, died on a signal, or printed no
/// JSON envelope.
#[derive(Debug, thiserror::Error)]
#[error("fallow {args} failed: {reason}")]
pub struct Failed {
    args: String,
    reason: String,
}

/// The fallow binary the startup check found in range.
pub struct Bin {
    path: PathBuf,
    pub version: String,
}

/// What fallow did in this run, as the reports show it.
#[derive(Clone, Copy)]
pub enum Outcome<'a> {
    Skipped,
    Ok(&'a str),
    Failed(&'a str),
}

/// What one fallow entry reports, before it is split per location.
#[derive(Clone)]
struct Issue {
    rule: String,
    severity: Severity,
    message: String,
    /// The fallow analysis that produced it.
    command: &'static str,
}

struct Location {
    path: String,
    /// `None` for a file-level location.
    at: Option<Position>,
}

/// 1-based lines, 0-based UTF-8 byte columns, `end_col` exclusive.
struct Position {
    line: u32,
    col: Option<u32>,
    end_line: Option<u32>,
    end_col: Option<u32>,
}

type Mapper = fn(&Value, &mut Vec<(Issue, Location)>);

/// `(result_key, id, description)` of every issue type with a `result_key` in `fallow schema`
/// 3.42.0.
#[rustfmt::skip]
const DEAD_CODE: &[(&str, &str, &str)] = &[
    ("unused_files", "unused-file", "File is not reachable from any entry point"),
    ("unused_exports", "unused-export", "Export is never imported"),
    ("unused_types", "unused-type", "Type export is never imported"),
    ("private_type_leaks", "private-type-leak", "Exported signature references a private type"),
    ("deprecated_exports_in_use", "deprecated-export-in-use", "Export marked @deprecated is still referenced"),
    ("unused_dependencies", "unused-dependency", "Dependency listed but never imported"),
    ("unused_dev_dependencies", "unused-dev-dependency", "Dev dependency listed but never imported"),
    ("unused_optional_dependencies", "unused-optional-dependency", "Optional dependency listed but never imported"),
    ("type_only_dependencies", "type-only-dependency", "Production dependency only used via type-only imports"),
    ("test_only_dependencies", "test-only-dependency", "Production dependency only imported by test files"),
    ("dev_dependencies_in_production", "dev-dependency-in-production", "devDependency imported by production code with a runtime import"),
    ("unused_enum_members", "unused-enum-member", "Enum member is never referenced"),
    ("unused_class_members", "unused-class-member", "Class member is never referenced"),
    ("unused_store_members", "unused-store-member", "Store member is never accessed by any consumer"),
    ("unused_namespace_members", "unused-namespace-member", "Namespace member is never referenced"),
    ("unresolved_imports", "unresolved-import", "Import could not be resolved"),
    ("unlisted_dependencies", "unlisted-dependency", "Dependency used but not in package.json"),
    ("unlisted_binaries", "unlisted-binary", "Binary used but provided by no installed package"),
    ("duplicate_exports", "duplicate-export", "Export name appears in multiple modules"),
    ("duplicate_export_aliases", "duplicate-export-alias", "One module exports one binding under two or more names"),
    ("circular_dependencies", "circular-dependency", "Circular dependency chain detected"),
    ("re_export_cycles", "re-export-cycle", "Two or more barrel files re-export from each other in a loop"),
    ("package_cycles", "package-cycle", "Two or more workspace packages import each other in a loop"),
    ("boundary_violations", "boundary-violation", "Import crosses a configured architecture boundary"),
    ("boundary_coverage_violations", "boundary-coverage", "Source file matches no configured architecture boundary zone"),
    ("boundary_call_violations", "boundary-call-violation", "Zoned file calls a callee its zone forbids"),
    ("policy_violations", "policy-violation", "Usage violates a configured rule-pack policy"),
    ("stale_suppressions", "stale-suppression", "Suppression comment or tag no longer matches any issue"),
    ("unused_catalog_entries", "unused-catalog-entry", "Catalog entry not referenced by any workspace package"),
    ("empty_catalog_groups", "empty-catalog-group", "Named catalog group has no entries"),
    ("unresolved_catalog_references", "unresolved-catalog-reference", "package.json references a catalog that does not declare the package"),
    ("unused_dependency_overrides", "unused-dependency-override", "Package-manager override target is not declared or resolved"),
    ("misconfigured_dependency_overrides", "misconfigured-dependency-override", "Package-manager override has an unparsable key or value"),
    ("invalid_client_exports", "invalid-client-export", "\"use client\" file exports a server-only / route-config name"),
    ("mixed_client_server_barrels", "mixed-client-server-barrel", "Barrel re-exports both a \"use client\" module and a server-only module"),
    ("misplaced_directives", "misplaced-directive", "\"use client\" / \"use server\" directive is not in the leading position and is ignored"),
    ("unprovided_injects", "unprovided-inject", "inject() / getContext() reads a key that no provide() / setContext() supplies"),
    ("unrendered_components", "unrendered-component", "A Vue / Svelte component is reachable through a barrel but rendered nowhere"),
    ("absent_component_props", "absent-component-prop", "Known reachable callers omit an optional prop consumed inside its component"),
    ("unused_component_props", "unused-component-prop", "A Vue, Svelte, or React component prop is referenced nowhere in its own component"),
    ("unused_component_emits", "unused-component-emit", "A Vue <script setup> defineEmits event is emitted nowhere in its own component"),
    ("unused_component_inputs", "unused-component-input", "An Angular @Input() / signal input() / model() input is read nowhere in its own component"),
    ("unused_component_outputs", "unused-component-output", "An Angular @Output() / signal output() output is emitted nowhere in its own component"),
    ("unused_svelte_events", "unused-svelte-event", "A Svelte component dispatches a createEventDispatcher event whose name is listened to nowhere in the project"),
    ("unused_effect_layers", "unused-effect-layer", "An Effect layer is composed only into layers that nothing uses"),
    ("unused_tagged_errors", "unused-tagged-error", "An Effect tagged error class is referenced only as a type or a catchTag string, and no code constructs or fails with it"),
    ("unproduced_catch_tags", "unproduced-catch-tag", "A catchTag handler names a project error tag that no code produces"),
    ("unused_server_actions", "unused-server-action", "A Next.js Server Action exported from a \"use server\" file is referenced by no code in the project"),
    ("unused_load_data_keys", "unused-load-data-key", "A SvelteKit load() return-object key is read by no consumer"),
    ("prop_drilling_chains", "prop-drilling", "A React/Preact prop is forwarded unchanged through 3+ pass-through components to a distant consumer"),
    ("thin_wrappers", "thin-wrapper", "A React/Preact component whose whole body is a single spread-forwarded child render (a candidate for inlining)"),
    ("duplicate_prop_shapes", "duplicate-prop-shape", "Three or more React/Preact components across two or more files declare an identical prop-name set (a missing shared Props type)"),
    ("route_collisions", "route-collision", "Two or more Next.js App Router route files resolve to the same URL"),
    ("dynamic_segment_name_conflicts", "dynamic-segment-name-conflict", "Sibling Next.js dynamic route segments use different slug names at the same position"),
];

/// `(id, description)` of the other issue types slopr builds a message for, from `fallow schema`
/// 3.42.0.
type IssueType = (&'static str, &'static str);
const HIGH_CYCLOMATIC: IssueType = (
    "high-cyclomatic-complexity",
    "Function has high cyclomatic complexity",
);
const HIGH_COGNITIVE: IssueType = (
    "high-cognitive-complexity",
    "Function has high cognitive complexity",
);
const HIGH_COMPLEXITY: IssueType = (
    "high-complexity",
    "Function exceeds both complexity thresholds",
);
const HIGH_CRAP: IssueType = (
    "high-crap-score",
    "Function has a high CRAP score (complexity combined with low coverage)",
);
const LONG_FILE: IssueType = (
    "long-file",
    "File has more lines than the file-length threshold",
);
const CODE_DUPLICATION: IssueType = ("code-duplication", "Duplicated code block");
const FEATURE_FLAG: IssueType = ("feature-flag", "Detected feature flag pattern");

/// A health finding's `exceeded` value and the rule fallow's SARIF reports for it.
const HEALTH: &[(&str, IssueType)] = &[
    ("cyclomatic", HIGH_CYCLOMATIC),
    ("cognitive", HIGH_COGNITIVE),
    ("both", HIGH_COMPLEXITY),
    ("crap", HIGH_CRAP),
    ("cyclomatic_crap", HIGH_CRAP),
    ("cognitive_crap", HIGH_CRAP),
    ("all", HIGH_CRAP),
    ("file_lines", LONG_FILE),
];

/// The first field present names the subject of a dead-code message.
const SUBJECT_KEYS: &[&str] = &[
    "export_name",
    "member_name",
    "package_name",
    "specifier",
    "import_specifier",
    "local_name",
];

/// Arrays that hold one location per element.
const LOCATION_LISTS: &[&str] = &["edges", "imported_from", "locations", "instances"];

/// `Some` when fallow is required and found in range, `None` when it is off or not required.
///
/// # Errors
///
/// fallow is required and missing or out of range.
pub fn check(config: &Fallow, paths: &[PathBuf]) -> Result<Option<Bin>, Unsatisfied> {
    let required = match config.enabled {
        FallowMode::Off => false,
        FallowMode::On => true,
        FallowMode::Auto => has_package_json(paths),
    };
    if !required {
        return Ok(None);
    }
    let local = Path::new("node_modules/.bin/fallow");
    let path = match &config.bin {
        Some(bin) => bin.clone(),
        None if local.is_file() => local.to_path_buf(),
        None => PathBuf::from("fallow"),
    };
    let found = Command::new(&path)
        .arg("--version")
        .output()
        .ok()
        .map(|out| {
            let text = String::from_utf8_lossy(&out.stdout);
            // The npm launcher prints a signature line after the version.
            let text = text.lines().next().unwrap_or_default().trim();
            text.strip_prefix("fallow ").unwrap_or(text).to_owned()
        });
    if let Some(found) = &found
        && let Ok(version) = Version::parse(found)
        && config.version.matches(&version)
    {
        return Ok(Some(Bin {
            path,
            version: version.to_string(),
        }));
    }
    Err(Unsatisfied {
        found,
        required: config.version.clone(),
    })
}

fn has_package_json(paths: &[PathBuf]) -> bool {
    Path::new("package.json").is_file()
        || paths.iter().any(|path| {
            WalkBuilder::new(path)
                .require_git(false)
                .build()
                .flatten()
                .any(|entry| {
                    entry.file_name() == "package.json"
                        && entry.file_type().is_some_and(|t| t.is_file())
                })
        })
}

/// Runs the configured fallow processes from the scan root, one after another, and maps their
/// results to findings under `paths`, in report order with fingerprints.
///
/// # Errors
///
/// A fallow process failed.
pub fn run(bin: &Bin, config: &Fallow, paths: &[PathBuf]) -> Result<Vec<Finding>, Failed> {
    let mut issues = Vec::new();
    for (mut args, map) in processes(&config.commands) {
        args.extend(["--format", "json", "--quiet"].map(String::from));
        args.extend(config.args.iter().cloned());
        map(&invoke(&bin.path, &args)?, &mut issues);
    }
    Ok(findings(issues, &bin.version, paths))
}

/// The arguments before `--format` of each fallow process, with the mapper for its envelope.
fn processes(commands: &[FallowCommand]) -> Vec<(Vec<String>, Mapper)> {
    let only: Vec<&str> = commands
        .iter()
        .filter_map(|command| match command {
            FallowCommand::DeadCode => Some("dead-code"),
            FallowCommand::Dupes => Some("dupes"),
            FallowCommand::Health => Some("health"),
            FallowCommand::Architecture => Some("architecture"),
            FallowCommand::Security | FallowCommand::Flags => None,
        })
        .collect();
    let mut out: Vec<(Vec<String>, Mapper)> = Vec::new();
    if !only.is_empty() {
        out.push((vec!["--only".into(), only.join(",")], map_combined));
    }
    if commands.contains(&FallowCommand::Security) {
        out.push((vec!["security".into()], map_security));
    }
    if commands.contains(&FallowCommand::Flags) {
        out.push((vec!["flags".into()], map_flags));
    }
    out
}

/// Exit codes 0 and 1 are both success, because slopr applies its own gate.
fn invoke(bin: &Path, args: &[String]) -> Result<Value, Failed> {
    let failed = |reason: String| Failed {
        args: args.join(" "),
        reason,
    };
    let out = Command::new(bin)
        .args(args)
        .output()
        .map_err(|err| failed(err.to_string()))?;
    if !matches!(out.status.code(), Some(0 | 1)) {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(failed(
            match stderr.lines().rev().find(|l| !l.trim().is_empty()) {
                Some(line) => format!("{}: {}", out.status, line.trim()),
                None => out.status.to_string(),
            },
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|err| failed(format!("invalid JSON: {err}")))
}

fn map_combined(envelope: &Value, out: &mut Vec<(Issue, Location)>) {
    for (key, entries) in envelope["check"].as_object().into_iter().flatten() {
        let Some(&(_, id, description)) = DEAD_CODE.iter().find(|(k, ..)| k == key) else {
            continue;
        };
        for entry in array(entries) {
            let message = match subject(entry) {
                Some(subject) => format!("{description}: {subject}"),
                None => description.to_owned(),
            };
            let issue = Issue {
                rule: format!("fallow/{id}"),
                severity: severity(entry),
                message,
                command: "dead-code",
            };
            push(out, &issue, locations(entry));
        }
    }
    let (id, description) = CODE_DUPLICATION;
    for group in array(&envelope["dupes"]["clone_groups"]) {
        let issue = Issue {
            rule: format!("fallow/{id}"),
            severity: Severity::Warn,
            message: format!(
                "{description}: {} lines, {} instances",
                group["line_count"],
                array(&group["instances"]).len()
            ),
            command: "dupes",
        };
        push(out, &issue, locations(group));
    }
    for entry in array(&envelope["health"]["findings"]) {
        let exceeded = entry["exceeded"].as_str();
        let Some(&(_, (id, description))) = HEALTH.iter().find(|(e, _)| Some(*e) == exceeded)
        else {
            continue;
        };
        let mut locations = locations(entry);
        let message = if id == LONG_FILE.0 {
            for location in &mut locations {
                location.at = None;
            }
            format!("{description}: {} lines", entry["line_count"])
        } else {
            format!(
                "{description}: {}, cyclomatic {}, cognitive {}, CRAP {}",
                entry["name"].as_str().unwrap_or_default(),
                entry["cyclomatic"],
                entry["cognitive"],
                entry["crap"]
            )
        };
        let issue = Issue {
            rule: format!("fallow/{id}"),
            severity: severity(entry),
            message,
            command: "health",
        };
        push(out, &issue, locations);
    }
}

fn map_security(envelope: &Value, out: &mut Vec<(Issue, Location)>) {
    for entry in array(&envelope["security_findings"]) {
        let Some(category) = entry["category"].as_str() else {
            continue;
        };
        let issue = Issue {
            rule: format!("fallow/{category}"),
            severity: Severity::Info,
            message: entry["evidence"].as_str().unwrap_or_default().to_owned(),
            command: "security",
        };
        push(out, &issue, locations(entry));
    }
}

fn map_flags(envelope: &Value, out: &mut Vec<(Issue, Location)>) {
    let (id, description) = FEATURE_FLAG;
    for entry in array(&envelope["feature_flags"]) {
        let issue = Issue {
            rule: format!("fallow/{id}"),
            severity: Severity::Info,
            message: format!(
                "{description}: {}",
                entry["flag_name"].as_str().unwrap_or_default()
            ),
            command: "flags",
        };
        push(out, &issue, locations(entry));
    }
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

/// `effective_severity` is fallow's severity after its own config. fallow drops entries it
/// turned off.
fn severity(entry: &Value) -> Severity {
    match entry["effective_severity"].as_str() {
        Some("error") => Severity::Error,
        _ => Severity::Warn,
    }
}

fn subject(entry: &Value) -> Option<String> {
    for &key in SUBJECT_KEYS {
        if let Some(name) = entry[key].as_str() {
            return Some(match entry["parent_name"].as_str() {
                Some(parent) if key == "member_name" => format!("{parent}.{name}"),
                _ => name.to_owned(),
            });
        }
    }
    let files: Vec<&str> = array(&entry["files"])
        .iter()
        .filter_map(Value::as_str)
        .collect();
    (!files.is_empty()).then(|| files.join(", "))
}

/// One issue per location, each with ` (location i of n)` when there are several.
fn push(out: &mut Vec<(Issue, Location)>, issue: &Issue, locations: Vec<Location>) {
    let n = locations.len();
    for (i, location) in locations.into_iter().enumerate() {
        let mut issue = issue.clone();
        if n > 1 {
            issue.message = format!("{} (location {} of {n})", issue.message, i + 1);
        }
        out.push((issue, location));
    }
}

fn locations(entry: &Value) -> Vec<Location> {
    let parent = entry["path"]
        .as_str()
        .or_else(|| entry["from_path"].as_str());
    match LOCATION_LISTS
        .iter()
        .map(|&key| array(&entry[key]))
        .find(|list| !list.is_empty())
    {
        Some(list) => list
            .iter()
            .filter_map(|item| location(item, parent))
            .collect(),
        None => location(entry, None).into_iter().collect(),
    }
}

/// `None` when neither `item` nor its entry has a path.
fn location(item: &Value, parent: Option<&str>) -> Option<Location> {
    let path = ["path", "file", "from_path"]
        .iter()
        .find_map(|&key| item[key].as_str())
        .or(parent)?;
    let number = |keys: &[&str]| {
        keys.iter()
            .find_map(|&key| item[key].as_u64())
            .map(|n| u32::try_from(n).unwrap_or(u32::MAX))
    };
    let at = number(&["line", "start_line"]).map(|line| Position {
        line: line.max(1),
        col: number(&["col", "start_col"]),
        end_line: number(&["end_line"]),
        end_col: number(&["end_col"]),
    });
    Some(Location {
        path: path.to_owned(),
        at,
    })
}

/// Normalizes paths, keeps those under `paths`, resolves spans and fingerprints against the
/// files on disk.
fn findings(issues: Vec<(Issue, Location)>, version: &str, paths: &[PathBuf]) -> Vec<Finding> {
    let root = std::env::current_dir()
        .and_then(fs::canonicalize)
        .unwrap_or_default();
    let scopes: Vec<String> = paths
        .iter()
        .filter_map(|path| fs::canonicalize(path).ok())
        .filter_map(|path| root_relative(&root, &path))
        .collect();
    let mut texts: HashMap<String, Option<String>> = HashMap::new();
    let mut findings: Vec<Finding> = issues
        .into_iter()
        .filter_map(|(issue, location)| {
            let path = root_relative(&root, Path::new(&location.path))?;
            if !scopes.iter().any(|scope| is_under(&path, scope)) {
                return None;
            }
            let text = texts
                .entry(path.clone())
                .or_insert_with(|| fs::read_to_string(&path).ok());
            Some(Finding {
                rule: issue.rule,
                severity: issue.severity,
                span: span(text.as_deref(), location.at),
                path,
                message: issue.message,
                fingerprint: String::new(),
                origin: Origin::Fallow {
                    version: version.to_owned(),
                    command: issue.command.to_owned(),
                },
            })
        })
        .collect();
    findings.sort_by(|a, b| crate::report_order(a).cmp(&crate::report_order(b)));

    let mut occurrences: HashMap<(String, String, String), usize> = HashMap::new();
    for f in &mut findings {
        let snippet = match &texts[&f.path] {
            Some(text) => finding::snippet(text, f.span),
            None => f.message.clone(),
        };
        let seen = occurrences
            .entry((f.path.clone(), f.rule.clone(), snippet.clone()))
            .or_default();
        f.fingerprint = finding::fingerprint(&f.rule, &f.path, &snippet, *seen);
        *seen += 1;
    }
    findings
}

/// Forward-slash path relative to `root`, or `None` when `path` lies outside it.
fn root_relative(root: &Path, path: &Path) -> Option<String> {
    let path = if path.is_absolute() {
        path.strip_prefix(root).ok()?
    } else {
        path
    };
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

/// An empty scope is the root itself.
fn is_under(path: &str, scope: &str) -> bool {
    scope.is_empty()
        || path
            .strip_prefix(scope)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Without the file, a byte column stands in for a character column.
fn span(text: Option<&str>, at: Option<Position>) -> Span {
    let line_text = |line: u32| text.and_then(|t| t.lines().nth(line as usize - 1));
    let column = |line: u32, byte: u32| match line_text(line) {
        Some(text) => char_column(text, byte),
        None => byte.saturating_add(1),
    };
    let Some(at) = at else {
        let (end_line, end_col) = text
            .and_then(|t| t.lines().zip(1..).last())
            .map_or((1, 1), |(last, n)| (n, char_column(last, u32::MAX)));
        return Span {
            start_line: 1,
            start_col: 1,
            end_line,
            end_col,
        };
    };
    let start_col = at.col.map_or(1, |byte| column(at.line, byte));
    let end_line = at.end_line.map_or(at.line, |line| line.max(at.line));
    let end_col = at.end_col.map_or(start_col, |byte| column(end_line, byte));
    Span {
        start_line: at.line,
        start_col,
        end_line,
        end_col,
    }
}

/// 1-based count of Unicode scalar values before byte offset `byte` of `line`.
fn char_column(line: &str, byte: u32) -> u32 {
    let chars = line
        .char_indices()
        .take_while(|&(i, _)| i < byte as usize)
        .count();
    u32::try_from(chars).unwrap_or(u32::MAX).saturating_add(1)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn at(line: u32, col: Option<u32>) -> Option<Position> {
        Some(Position {
            line,
            col,
            end_line: None,
            end_col: None,
        })
    }

    fn cols(span: Span) -> [u32; 4] {
        [span.start_line, span.start_col, span.end_line, span.end_col]
    }

    #[test]
    fn span_converts_byte_columns_and_covers_files() {
        let text = "a\n/* ééé */ export const uni = 1;\nlast é\n";
        assert_eq!(cols(span(Some(text), at(2, Some(26)))), [2, 24, 2, 24]);
        assert_eq!(cols(span(None, at(2, Some(26)))), [2, 27, 2, 27]);
        assert_eq!(cols(span(Some(text), at(2, None))), [2, 1, 2, 1]);
        assert_eq!(cols(span(Some(text), None)), [1, 1, 3, 7]);
        assert_eq!(cols(span(None, None)), [1, 1, 1, 1]);
    }

    #[test]
    fn root_relative_drops_paths_outside_root() {
        let root = Path::new("/repo");
        let rel = |p: &str| root_relative(root, Path::new(p));
        assert_eq!(rel("/repo/src/a.ts").as_deref(), Some("src/a.ts"));
        assert_eq!(rel("./src/a.ts").as_deref(), Some("src/a.ts"));
        assert_eq!(rel("/other/a.ts"), None);
        assert_eq!(rel("../a.ts"), None);
    }

    fn mapped(map: Mapper, envelope: &Value) -> Vec<(String, Severity, String, String)> {
        let mut out = Vec::new();
        map(envelope, &mut out);
        out.into_iter()
            .map(|(issue, location)| (issue.rule, issue.severity, issue.message, location.path))
            .collect()
    }

    #[test]
    fn security_and_flags_map_to_info() {
        let security = json!({"security_findings": [{
            "category": "command-injection", "path": "src/extra.ts", "line": 5, "col": 4,
            "evidence": "Non-literal command passed to node:child_process.exec().",
            "trace": [{"path": "src/other.ts", "line": 1, "col": 0}]
        }]});
        assert_eq!(
            mapped(map_security, &security),
            [(
                "fallow/command-injection".into(),
                Severity::Info,
                "Non-literal command passed to node:child_process.exec().".into(),
                "src/extra.ts".into()
            )]
        );
        let flags = json!({"feature_flags": [{
            "path": "src/extra.ts", "flag_name": "FEATURE_NEW_CHECKOUT", "line": 4, "col": 6
        }]});
        assert_eq!(
            mapped(map_flags, &flags),
            [(
                "fallow/feature-flag".into(),
                Severity::Info,
                "Detected feature flag pattern: FEATURE_NEW_CHECKOUT".into(),
                "src/extra.ts".into()
            )]
        );
    }

    #[test]
    fn dead_code_subjects_and_file_level_entries() {
        let envelope = json!({"check": {
            "unused_class_members": [{
                "path": "src/a.ts", "parent_name": "Cart", "member_name": "clear",
                "line": 3, "col": 2, "effective_severity": "warn"
            }],
            "unused_files": [{"path": "src/dead.ts", "effective_severity": "error"}],
            "not_a_fallow_array": [{"path": "src/a.ts", "line": 1}]
        }});
        let mut out = Vec::new();
        map_combined(&envelope, &mut out);
        let got: Vec<_> = out
            .iter()
            .map(|(issue, location)| (issue.message.as_str(), location.at.is_none()))
            .collect();
        assert_eq!(
            got,
            [
                ("Class member is never referenced: Cart.clear", false),
                ("File is not reachable from any entry point", true),
            ]
        );
    }
}
