use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use regex::Regex;

struct Finding {
    line: usize,
    rule: String,
}

#[test]
fn fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules");
    let line_re = Regex::new(r"^(.+):(\d+):(\d+) (info|warn|error) (\S+) ").expect("valid regex");
    let marker_re =
        Regex::new(r"(?://|#|/\*)\s*expect:\s*(\S+?)\s*(?:\*/)?\s*$").expect("valid regex");
    let mut failures = Vec::new();

    for dir in rule_dirs(&root) {
        let rule = rule_id(&root, &dir);
        let mut rule_seen = false;
        for name in fixture_files(&dir) {
            let output = cargo_bin_cmd!("slopr")
                .current_dir(&dir)
                .args(["check", &name])
                .output()
                .expect("slopr runs");
            if !matches!(output.status.code(), Some(0 | 1)) {
                let stderr = String::from_utf8_lossy(&output.stderr);
                failures.push(format!(
                    "{rule}/{name}: exit {:?}: {stderr}",
                    output.status.code()
                ));
                continue;
            }
            let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
            let findings: Vec<Finding> = stdout
                .lines()
                .filter_map(|l| line_re.captures(l))
                .filter(|c| c[1] == name)
                .map(|c| Finding {
                    line: c[2].parse().expect("line number"),
                    rule: c[5].to_string(),
                })
                .collect();
            rule_seen |= name.starts_with("bad") && findings.iter().any(|f| f.rule == rule);

            if name.starts_with("good") {
                for f in findings.iter().filter(|f| f.rule == rule) {
                    failures.push(format!("{rule}/{name}:{}: unexpected {rule}", f.line));
                }
                continue;
            }

            let text = fs::read_to_string(dir.join(&name)).expect("fixture is readable");
            let markers: Vec<(usize, &str)> = text
                .lines()
                .enumerate()
                .filter_map(|(i, l)| {
                    marker_re
                        .captures(l)
                        .map(|c| (i + 1, c.get(1).expect("group").as_str()))
                })
                .collect();
            if markers.is_empty() {
                failures.push(format!("{rule}/{name}: no `expect:` marker"));
            }
            for &(line, want) in &markers {
                let got = findings
                    .iter()
                    .filter(|f| f.line == line && f.rule == want)
                    .count();
                if got != 1 {
                    failures.push(format!("{rule}/{name}:{line}: want 1 {want}, got {got}"));
                }
            }
            for f in &findings {
                let checked = f.rule == rule || markers.iter().any(|&(_, m)| m == f.rule);
                let marked = markers.iter().any(|&(l, m)| l == f.line && m == f.rule);
                if checked && !marked {
                    failures.push(format!("{rule}/{name}:{}: unmarked {}", f.line, f.rule));
                }
            }
        }
        if !rule_seen {
            failures.push(format!("{rule}: no bad* file produced a {rule} finding"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} fixture failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Dirs exactly two levels below the root, whose relative path is a rule ID.
fn rule_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = subdirs(root)
        .iter()
        .flat_map(|family| subdirs(family))
        .collect();
    dirs.sort();
    dirs
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("fixture dir is readable")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.is_dir())
        .collect()
}

fn rule_id(root: &Path, dir: &Path) -> String {
    let relative = dir.strip_prefix(root).expect("dir is under root");
    let parts: Vec<_> = relative
        .iter()
        .map(|p| p.to_str().expect("UTF-8 dir name"))
        .collect();
    parts.join("/")
}

fn fixture_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("fixture dir is readable")
        .map(|e| {
            e.expect("dir entry")
                .file_name()
                .into_string()
                .expect("UTF-8 file name")
        })
        .filter(|n| n.starts_with("bad") || n.starts_with("good"))
        .collect();
    names.sort();
    names
}
