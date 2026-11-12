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

const SUPPRESS: &str = "tests/fixtures/suppress";

fn assert_suppress(fixture: &str, want: &str) {
    let output = run_check(&[&format!("{SUPPRESS}/{fixture}")]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), want);
}

#[test]
fn suppress_next_line() {
    assert_suppress(
        "next-line.ts",
        "tests/fixtures/suppress/next-line.ts:5:15 warn ts/unsafe-assertion `as any` assertion

1 ts/unsafe-assertion
1 findings (0 error, 1 warn, 0 info)
",
    );
    let path = format!("{SUPPRESS}/next-line.ts");
    let json = run_check(&["--format", "json", &path]);
    assert_eq!(json.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&json.stdout).expect("json is JSON");
    let lines: Vec<&serde_json::Value> = report["findings"]
        .as_array()
        .expect("findings is an array")
        .iter()
        .map(|f| &f["span"]["start_line"])
        .collect();
    assert_eq!(lines, [5]);
    let sarif = run_check(&["--format", "sarif", &path]);
    assert_eq!(sarif.status.code(), Some(0));
    let log: serde_json::Value = serde_json::from_slice(&sarif.stdout).expect("SARIF is JSON");
    let lines: Vec<&serde_json::Value> = log["runs"][0]["results"]
        .as_array()
        .expect("results is an array")
        .iter()
        .map(|r| &r["locations"][0]["physicalLocation"]["region"]["startLine"])
        .collect();
    assert_eq!(lines, [5]);
}

#[test]
fn suppress_next_line_skips_blank() {
    assert_suppress(
        "next-line-skips-blank.ts",
        "tests/fixtures/suppress/next-line-skips-blank.ts:6:15 warn ts/unsafe-assertion `as any` assertion

1 ts/unsafe-assertion
1 findings (0 error, 1 warn, 0 info)
",
    );
}

#[test]
fn suppress_file() {
    assert_suppress(
        "file.ts",
        "tests/fixtures/suppress/file.ts:5:1 info comment/untracked-todo `TODO` without an issue reference

1 comment/untracked-todo
1 findings (0 error, 0 warn, 1 info)
",
    );
}

#[test]
fn suppress_multiple_ids() {
    assert_suppress(
        "multiple-ids.ts",
        "tests/fixtures/suppress/multiple-ids.ts:5:15 warn ts/unsafe-assertion `as any` assertion
tests/fixtures/suppress/multiple-ids.ts:5:29 info comment/untracked-todo `TODO` without an issue reference

1 comment/untracked-todo
1 ts/unsafe-assertion
2 findings (0 error, 1 warn, 1 info)
",
    );
}

#[test]
fn suppress_missing_reason_is_invalid() {
    assert_suppress(
        "missing-reason.ts",
        "tests/fixtures/suppress/missing-reason.ts:5:4 warn suppress/invalid-directive directive needs a reason after ` -- `
tests/fixtures/suppress/missing-reason.ts:6:15 warn ts/unsafe-assertion `as any` assertion
tests/fixtures/suppress/missing-reason.ts:7:4 warn suppress/invalid-directive directive needs a reason after ` -- `
tests/fixtures/suppress/missing-reason.ts:8:19 warn ts/unsafe-assertion `as any` assertion

2 suppress/invalid-directive
2 ts/unsafe-assertion
4 findings (0 error, 4 warn, 0 info)
",
    );
}

#[test]
fn suppress_unknown_rule_is_invalid() {
    assert_suppress(
        "unknown-rule.ts",
        "tests/fixtures/suppress/unknown-rule.ts:5:4 warn suppress/invalid-directive directive names unknown rule `ts/unsafe-asertion`
tests/fixtures/suppress/unknown-rule.ts:6:15 warn ts/unsafe-assertion `as any` assertion

1 suppress/invalid-directive
1 ts/unsafe-assertion
2 findings (0 error, 2 warn, 0 info)
",
    );
}

#[test]
fn suppress_fallow_prefix_accepted() {
    assert_suppress(
        "fallow-prefix.ts",
        "tests/fixtures/suppress/fallow-prefix.ts:5:22 warn ts/unsafe-assertion `as any` assertion

1 ts/unsafe-assertion
1 findings (0 error, 1 warn, 0 info)
",
    );
}

#[test]
fn suppress_invalid_cannot_be_suppressed() {
    assert_suppress(
        "invalid-cannot-be-suppressed.ts",
        "tests/fixtures/suppress/invalid-cannot-be-suppressed.ts:1:4 warn suppress/invalid-directive directive names unknown rule `suppress/invalid-directive`
tests/fixtures/suppress/invalid-cannot-be-suppressed.ts:6:4 warn suppress/invalid-directive malformed directive, expected `slopr-ignore-next-line|file <rule-id> -- <reason>`
tests/fixtures/suppress/invalid-cannot-be-suppressed.ts:7:15 warn ts/unsafe-assertion `as any` assertion

2 suppress/invalid-directive
1 ts/unsafe-assertion
3 findings (0 error, 3 warn, 0 info)
",
    );
}

#[test]
fn suppress_block_comment_targets_line_after_comment() {
    assert_suppress(
        "block-comment.ts",
        "tests/fixtures/suppress/block-comment.ts:9:4 warn suppress/invalid-directive directive needs a reason after ` -- `
tests/fixtures/suppress/block-comment.ts:11:15 warn ts/unsafe-assertion `as any` assertion

1 suppress/invalid-directive
1 ts/unsafe-assertion
2 findings (0 error, 2 warn, 0 info)
",
    );
}

#[test]
fn suppress_partly_invalid_suppresses_nothing() {
    assert_suppress(
        "partly-invalid.ts",
        "tests/fixtures/suppress/partly-invalid.ts:5:4 warn suppress/invalid-directive directive names unknown rule `made/up`
tests/fixtures/suppress/partly-invalid.ts:6:15 warn ts/unsafe-assertion `as any` assertion

1 suppress/invalid-directive
1 ts/unsafe-assertion
2 findings (0 error, 2 warn, 0 info)
",
    );
}

#[test]
fn suppress_mention_mid_line_ignored() {
    assert_suppress(
        "mention-mid-line.ts",
        "tests/fixtures/suppress/mention-mid-line.ts:6:15 warn ts/unsafe-assertion `as any` assertion
tests/fixtures/suppress/mention-mid-line.ts:8:18 warn ts/unsafe-assertion `as any` assertion

2 ts/unsafe-assertion
2 findings (0 error, 2 warn, 0 info)
",
    );
}

const JS_MIN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/repos/js-min");
const FAKE_FALLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fake-fallow/fallow"
);

fn fallow_unsatisfied(found: &str) -> String {
    format!(
        "slopr: fallow {found} does not satisfy >=3.42, <4. Install it with: npm install -D fallow@^3.42 (or cargo install fallow-cli --version ^3.42). Use --fallow off to skip.\n"
    )
}

/// `slopr check <args>` in `dir`, with an empty PATH dir so a real fallow never answers.
fn fallow_check(test: &str, dir: &std::path::Path, args: &[&str]) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("slopr")
        .current_dir(dir)
        .env("PATH", empty_dir(&format!("{test}_path")))
        .arg("check")
        .args(args)
        .assert()
}

/// A temp dir with a `package.json` and one clean source file.
fn js_dir(name: &str) -> std::path::PathBuf {
    let dir = empty_dir(name);
    for file in ["package.json", "index.js"] {
        std::fs::copy(format!("{JS_MIN}/{file}"), dir.join(file)).expect("fixture is copied");
    }
    dir
}

#[test]
fn fallow_check_missing_exits_3() {
    fallow_check("fallow_check_missing_exits_3", JS_MIN.as_ref(), &[])
        .code(3)
        .stdout("")
        .stderr(fallow_unsatisfied("not found"));
}

#[test]
fn fallow_check_out_of_range_exits_3() {
    cargo_bin_cmd!("slopr")
        .current_dir(JS_MIN)
        .env("PATH", empty_dir("fallow_check_out_of_range_exits_3_path"))
        .env("FAKE_FALLOW_VERSION", "3.41.0")
        .args(["check", "--fallow-bin", FAKE_FALLOW])
        .assert()
        .code(3)
        .stdout("")
        .stderr(fallow_unsatisfied("3.41.0"));
}

#[test]
fn fallow_check_in_range_passes() {
    fallow_check(
        "fallow_check_in_range_passes",
        JS_MIN.as_ref(),
        &["--fallow-bin", FAKE_FALLOW],
    )
    .code(0)
    .stdout("0 findings\n");
}

#[test]
fn fallow_check_off_skips() {
    fallow_check(
        "fallow_check_off_skips",
        JS_MIN.as_ref(),
        &["--fallow", "off"],
    )
    .code(0)
    .stdout("0 findings\n");
}

#[test]
fn fallow_check_auto_without_package_json_skips() {
    let dir = empty_dir("fallow_check_auto_without_package_json_skips");
    std::fs::copy(format!("{JS_MIN}/index.js"), dir.join("index.js")).expect("fixture is copied");
    fallow_check("fallow_check_auto_without_package_json_skips", &dir, &[])
        .code(0)
        .stdout("0 findings\n");
}

#[test]
fn fallow_check_auto_finds_nested_package_json() {
    fallow_check(
        "fallow_check_auto_finds_nested_package_json",
        env!("CARGO_MANIFEST_DIR").as_ref(),
        &["tests/fixtures/repos"],
    )
    .code(3)
    .stdout("")
    .stderr(fallow_unsatisfied("not found"));
}

#[test]
fn fallow_check_bin_flag_wins() {
    let dir = js_dir("fallow_check_bin_flag_wins");
    std::fs::write(
        dir.join("slopr.toml"),
        "[fallow]\nbin = \"missing/fallow\"\n",
    )
    .expect("slopr.toml is written");
    fallow_check("fallow_check_bin_flag_wins", &dir, &[])
        .code(3)
        .stdout("")
        .stderr(fallow_unsatisfied("not found"));
    fallow_check(
        "fallow_check_bin_flag_wins",
        &dir,
        &["--fallow-bin", FAKE_FALLOW],
    )
    .code(0)
    .stdout("0 findings\n");
}

#[test]
fn fallow_check_node_modules_bin_found() {
    let dir = js_dir("fallow_check_node_modules_bin_found");
    let bin = dir.join("node_modules/.bin");
    std::fs::create_dir_all(&bin).expect("node_modules/.bin is created");
    // fs::copy keeps the executable bit.
    std::fs::copy(FAKE_FALLOW, bin.join("fallow")).expect("fake fallow is copied");
    fallow_check("fallow_check_node_modules_bin_found", &dir, &[])
        .code(0)
        .stdout("0 findings\n");
}

const FAKE_FALLOW_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake-fallow");
const FAKE_FALLOW_REPO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fake-fallow/repo"
);

/// `slopr check <args>` in the planted repo, with fake fallow printing the canned envelopes and
/// exiting with `exit`.
fn fallow_run(exit: &str, args: &[&str]) -> std::process::Output {
    cargo_bin_cmd!("slopr")
        .current_dir(FAKE_FALLOW_REPO)
        .env("FAKE_FALLOW_DIR", FAKE_FALLOW_DIR)
        .env("FAKE_FALLOW_EXIT", exit)
        .args(["check", "--fallow-bin", FAKE_FALLOW])
        .args(args)
        .output()
        .expect("slopr runs")
}

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

#[test]
fn fallow_run_findings_in_human() {
    let output = fallow_run("0", &[]);
    assert_eq!(output.status.code(), Some(1));
    insta::assert_snapshot!(stdout(&output));
}

#[test]
fn fallow_run_sarif_fallow_run() {
    let output = fallow_run("1", &["--format", "sarif"]);
    assert_eq!(output.status.code(), Some(1));
    let log = json(&output);
    let runs = log["runs"].as_array().expect("runs is an array");
    assert_eq!(runs.len(), 2);
    let rule_ids = |run: &serde_json::Value| -> Vec<String> {
        run["results"]
            .as_array()
            .expect("results is an array")
            .iter()
            .map(|r| r["ruleId"].as_str().expect("ruleId is a string").to_owned())
            .collect()
    };
    let slopr = rule_ids(&runs[0]);
    assert!(!slopr.is_empty());
    assert!(
        slopr.iter().all(|id| !id.starts_with("fallow/")),
        "{slopr:?}"
    );
    let mut fallow = rule_ids(&runs[1]);
    assert!(
        fallow.iter().all(|id| id.starts_with("fallow/")),
        "{fallow:?}"
    );
    fallow.sort();
    fallow.dedup();
    let declared: Vec<&str> = runs[1]["tool"]["driver"]["rules"]
        .as_array()
        .expect("driver rules is an array")
        .iter()
        .map(|rule| rule["id"].as_str().expect("rule id is a string"))
        .collect();
    assert_eq!(declared, fallow);
    insta::assert_snapshot!(serde_json::to_string_pretty(&runs[1]).expect("run serializes"));
}

#[test]
fn fallow_run_json_tools_ok() {
    let output = fallow_run("0", &["--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let report = json(&output);
    assert_eq!(
        report["tools"],
        serde_json::json!([{"name": "fallow", "version": "3.42.0", "status": "ok"}])
    );
    let origins: std::collections::BTreeMap<&str, &serde_json::Value> = report["findings"]
        .as_array()
        .expect("findings is an array")
        .iter()
        .map(|f| (f["rule"].as_str().expect("rule is a string"), &f["origin"]))
        .collect();
    let fallow = |command: &str| serde_json::json!({"kind": "fallow", "version": "3.42.0", "command": command});
    assert_eq!(*origins["fallow/unused-export"], fallow("dead-code"));
    assert_eq!(*origins["fallow/code-duplication"], fallow("dupes"));
    assert_eq!(*origins["fallow/high-crap-score"], fallow("health"));
    assert_eq!(
        *origins["ts/unsafe-assertion"],
        serde_json::json!({"kind": "native"})
    );
}

#[test]
fn fallow_run_exit_2_is_tool_failure() {
    let output = fallow_run("2", &[]);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        stdout(&output),
        "src/index.ts:10:1 warn debug/leftover `console.log` left in library code

src/utils.ts:2:10 warn ts/unsafe-assertion `as any` assertion

1 debug/leftover
1 ts/unsafe-assertion
2 findings (0 error, 2 warn, 0 info)
"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
        "slopr: fallow --only dead-code,dupes,health --format json --quiet failed: exit status: 2\n"
    );
    let report = json(&fallow_run("2", &["--format", "json"]));
    assert_eq!(report["tools"][0]["status"], "failed");
}

#[test]
fn fallow_run_multi_location_split() {
    let output = fallow_run("0", &[]);
    let cycle: Vec<&str> = stdout(&output)
        .lines()
        .filter(|line| line.contains(" fallow/circular-dependency "))
        .collect();
    assert_eq!(
        cycle,
        [
            "src/cycle-a.ts:1:10 error fallow/circular-dependency Circular dependency chain detected: src/cycle-a.ts, src/cycle-b.ts (location 1 of 2)",
            "src/cycle-b.ts:1:10 error fallow/circular-dependency Circular dependency chain detected: src/cycle-a.ts, src/cycle-b.ts (location 2 of 2)",
        ]
    );
}

#[test]
fn fallow_run_paths_filter() {
    let output = fallow_run("0", &["src/utils.ts", "src/cycle-b.ts"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        "src/cycle-b.ts:1:10 error fallow/circular-dependency Circular dependency chain detected: src/cycle-a.ts, src/cycle-b.ts (location 2 of 2)

src/utils.ts:2:10 warn ts/unsafe-assertion `as any` assertion
src/utils.ts:5:14 error fallow/unused-export Export is never imported: unusedHelper
src/utils.ts:7:24 error fallow/unused-export Export is never imported: uni

1 fallow/circular-dependency
2 fallow/unused-export
1 ts/unsafe-assertion
4 findings (3 error, 1 warn, 0 info)
"
    );
}

const REPOS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/repos");

/// `slopr check --fallow off <args>` with the fixture repo as cwd, so output paths are root-relative.
fn repo_check(repo: &str, args: &[&str]) -> std::process::Output {
    cargo_bin_cmd!("slopr")
        .current_dir(format!("{REPOS}/{repo}"))
        .args(["check", "--fallow", "off"])
        .args(args)
        .output()
        .expect("slopr runs")
}

fn repo_report(repo: &str, format: &str) -> String {
    let output = repo_check(repo, &["--format", format]);
    stdout(&output).to_owned()
}

#[test]
fn repos_ts_app_human() {
    insta::assert_snapshot!(repo_report("ts-app", "human"));
}

#[test]
fn repos_ts_app_json() {
    insta::assert_snapshot!(repo_report("ts-app", "json"));
}

#[test]
fn repos_ts_app_sarif() {
    insta::assert_snapshot!(repo_report("ts-app", "sarif"));
}

#[test]
fn repos_py_lib_human() {
    insta::assert_snapshot!(repo_report("py-lib", "human"));
}

#[test]
fn repos_py_lib_json() {
    insta::assert_snapshot!(repo_report("py-lib", "json"));
}

#[test]
fn repos_py_lib_sarif() {
    insta::assert_snapshot!(repo_report("py-lib", "sarif"));
}

#[test]
fn repos_rust_crate_human() {
    insta::assert_snapshot!(repo_report("rust-crate", "human"));
}

#[test]
fn repos_rust_crate_json() {
    insta::assert_snapshot!(repo_report("rust-crate", "json"));
}

#[test]
fn repos_rust_crate_sarif() {
    insta::assert_snapshot!(repo_report("rust-crate", "sarif"));
}

#[test]
fn repos_exit_0() {
    // ts-app's native findings are all warn, below the default error gate.
    let output = repo_check("ts-app", &[]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn repos_exit_1() {
    let output = repo_check("py-lib", &[]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn repos_exit_2() {
    let output = repo_check("rust-crate", &["--max-severity-exit", "fatal"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "scanned anyway");
}

#[test]
fn repos_exit_3() {
    cargo_bin_cmd!("slopr")
        .current_dir(format!("{REPOS}/ts-app"))
        .env("FAKE_FALLOW_VERSION", "3.41.0")
        .args(["check", "--fallow-bin", FAKE_FALLOW])
        .assert()
        .code(3)
        .stderr(fallow_unsatisfied("3.41.0"));
}
