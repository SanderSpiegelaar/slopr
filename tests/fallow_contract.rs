use assert_cmd::cargo::cargo_bin_cmd;

const TS_APP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/repos/ts-app");

const PLANTED_RULES: [&str; 5] = [
    "fallow/unused-export",
    "fallow/unlisted-dependency",
    "fallow/code-duplication",
    // classify exceeds the cognitive limit, which fallow reports as `exceeded: cognitive_crap`.
    "fallow/high-crap-score",
    "fallow/circular-dependency",
];

#[test]
fn contract_planted_issues_reported() {
    let Some(fallow_bin) = std::env::var_os("FALLOW_BIN") else {
        println!("skipping contract_planted_issues_reported: FALLOW_BIN is unset");
        return;
    };
    let output = cargo_bin_cmd!("slopr")
        .current_dir(TS_APP)
        .args([
            "check",
            "--format",
            "json",
            "--fallow",
            "on",
            "--fallow-bin",
        ])
        .arg(&fallow_bin)
        .output()
        .expect("slopr runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("stdout is JSON ({err}); stderr: {stderr}"));

    let fallow = report["tools"]
        .as_array()
        .expect("tools is an array")
        .iter()
        .find(|tool| tool["name"] == "fallow")
        .unwrap_or_else(|| panic!("tools has a fallow entry: {}", report["tools"]));
    assert_eq!(
        fallow["status"], "ok",
        "fallow ran cleanly: {fallow}; stderr: {stderr}"
    );

    let rules: Vec<&str> = report["findings"]
        .as_array()
        .expect("findings is an array")
        .iter()
        .filter_map(|finding| finding["rule"].as_str())
        .collect();
    let missing: Vec<&str> = PLANTED_RULES
        .into_iter()
        .filter(|rule| !rules.contains(rule))
        .collect();
    assert!(
        missing.is_empty(),
        "planted issues missing from findings: {missing:?}; reported: {rules:?}"
    );
}
