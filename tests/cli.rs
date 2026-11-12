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
