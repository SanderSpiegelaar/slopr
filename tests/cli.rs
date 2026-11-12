use assert_cmd::cargo::cargo_bin_cmd;

const FIXTURES: &str = "tests/fixtures/rules/ts/unsafe-assertion";

#[test]
fn skeleton_flags_unsafe_assertion() {
    let output = cargo_bin_cmd!("slopr")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["check", &format!("{FIXTURES}/bad.ts")])
        .output()
        .expect("slopr runs");
    // ts/unsafe-assertion is warn, below the default error gate.
    assert_eq!(output.status.code(), Some(0));
    insta::assert_snapshot!(String::from_utf8(output.stdout).expect("stdout is UTF-8"));
}

#[test]
fn skeleton_clean_file_exits_zero() {
    cargo_bin_cmd!("slopr")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["check", &format!("{FIXTURES}/good.ts")])
        .assert()
        .code(0)
        .stdout("0 findings\n");
}

const CONFIG_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/config");

fn check_in(fixture: &str) -> std::process::Output {
    cargo_bin_cmd!("slopr")
        .current_dir(format!("{CONFIG_FIXTURES}/{fixture}"))
        .arg("check")
        .output()
        .expect("slopr runs")
}

fn stdout(output: &std::process::Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn assert_config_error(fixture: &str) {
    let output = check_in(fixture);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.starts_with("slopr: slopr.toml: "), "{stderr}");
}

/// A fresh empty directory per test, so parallel tests never share files.
fn empty_dir(name: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir is created");
    dir
}

#[test]
fn config_rule_off() {
    let output = check_in("rule-off");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "0 findings\n");
}

#[test]
fn config_severity_raised() {
    let output = check_in("severity-raised");
    assert_eq!(output.status.code(), Some(1));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn config_override_last_wins() {
    let output = check_in("override-last-wins");
    assert_eq!(output.status.code(), Some(0));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn config_exclude() {
    let output = check_in("exclude");
    assert_eq!(output.status.code(), Some(0));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn config_unknown_key_exits_2() {
    assert_config_error("unknown-key");
}

#[test]
fn config_bad_regex_exits_2() {
    assert_config_error("bad-regex");
}

#[test]
fn config_init_writes_file() {
    let dir = empty_dir("config_init_writes_file");
    cargo_bin_cmd!("slopr")
        .current_dir(&dir)
        .arg("init")
        .assert()
        .code(0);
    let written = std::fs::read_to_string(dir.join("slopr.toml")).expect("slopr.toml is written");
    insta::assert_snapshot!(written);
    cargo_bin_cmd!("slopr")
        .current_dir(&dir)
        .arg("check")
        .assert()
        .code(0)
        .stdout("0 findings\n");
}

#[test]
fn config_init_refuses_overwrite() {
    let dir = empty_dir("config_init_refuses_overwrite");
    std::fs::write(dir.join("slopr.toml"), "exclude = []\n").expect("slopr.toml is written");
    cargo_bin_cmd!("slopr")
        .current_dir(&dir)
        .arg("init")
        .assert()
        .code(2);
    let kept = std::fs::read_to_string(dir.join("slopr.toml")).expect("slopr.toml is read");
    assert_eq!(kept, "exclude = []\n");
}

/// One file each at error, info and warn.
const MIXED: &[&str] = &[
    "tests/fixtures/rules/comment/agent-artifact/bad.rs",
    "tests/fixtures/rules/comment/untracked-todo/bad.ts",
    "tests/fixtures/rules/ts/unsafe-assertion/bad.tsx",
];

/// Runs `slopr check <args>` from the repo root.
fn run_check(args: &[&str]) -> std::process::Output {
    cargo_bin_cmd!("slopr")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("check")
        .args(args)
        .output()
        .expect("slopr runs")
}

#[test]
fn output_json_snapshot() {
    let output = run_check(&[&["--format", "json"], MIXED].concat());
    assert_eq!(output.status.code(), Some(1));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn output_sarif_snapshot() {
    let output = run_check(&[&["--format", "sarif"], MIXED].concat());
    assert_eq!(output.status.code(), Some(1));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn output_sarif_required_fields() {
    let output = run_check(&["--format", "sarif", "tests/fixtures/rules"]);
    let sarif: serde_json::Value = serde_json::from_slice(&output.stdout).expect("SARIF is JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["$schema"].is_string());
    let driver = &sarif["runs"][0]["tool"]["driver"];
    assert_eq!(driver["name"], "slopr");
    let rule_ids: Vec<&str> = driver["rules"]
        .as_array()
        .expect("driver rules is an array")
        .iter()
        .map(|rule| rule["id"].as_str().expect("rule id is a string"))
        .collect();
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("results is an array");
    assert!(!results.is_empty());
    for result in results {
        let rule_id = result["ruleId"].as_str().expect("result has a ruleId");
        assert!(
            rule_ids.contains(&rule_id),
            "{rule_id} is not a driver rule"
        );
        assert!(result["level"].is_string(), "{result}");
        assert!(result["message"]["text"].is_string(), "{result}");
        assert!(
            result["locations"][0]["physicalLocation"]["region"]["startLine"].is_u64(),
            "{result}"
        );
        assert!(
            result["partialFingerprints"]["slopr/v1"].is_string(),
            "{result}"
        );
    }
}

#[test]
fn output_gate_warn_fails_on_warn() {
    let bad = format!("{FIXTURES}/bad.ts");
    let output = run_check(&["--max-severity-exit", "warn", &bad]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn output_gate_default_passes_warn() {
    let output = run_check(&[&format!("{FIXTURES}/bad.ts")]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn output_usage_error_exits_2() {
    for args in [["--format", "xml"], ["--max-severity-exit", "fatal"]] {
        let output = run_check(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?} scanned anyway");
    }
}

#[test]
fn output_rules_lists_native() {
    let output = cargo_bin_cmd!("slopr")
        .arg("rules")
        .output()
        .expect("slopr runs");
    assert_eq!(output.status.code(), Some(0));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn output_json_deterministic() {
    let first = run_check(&["--format", "json", "tests/fixtures/rules"]);
    let second = run_check(&["--format", "json", "tests/fixtures/rules"]);
    let report: serde_json::Value = serde_json::from_slice(&first.stdout).expect("json is JSON");
    assert!(report["summary"]["total"].as_u64() > Some(0), "no findings");
    assert_eq!(first.stdout, second.stdout);
}
