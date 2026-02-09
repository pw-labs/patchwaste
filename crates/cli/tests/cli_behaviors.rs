use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn cli_analyze_writes_reports_and_exits_0_without_budget() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/synthetic_case_01/BuildOutput");

    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyze",
        "--input",
        fixture_path.to_str().unwrap(),
        "--out",
        "patchwaste-out-test",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_bytes=12345678"));

    assert!(std::path::Path::new("patchwaste-out-test/report.json").exists());
    assert!(std::path::Path::new("patchwaste-out-test/report.md").exists());
}

#[test]
fn cli_analyze_exits_2_when_budget_fails() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/synthetic_case_01/BuildOutput");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let baseline_path = std::env::temp_dir().join(format!("patchwaste-baseline-{nonce}.json"));

    fs::write(&baseline_path, r#"{"metrics":{"new_bytes":1000}}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyze",
        "--input",
        fixture_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--budget-ratio",
        "1.25",
        "--out",
        "patchwaste-out-test",
    ]);

    cmd.assert().code(2);

    let _ = fs::remove_file(baseline_path);
}

#[test]
fn cli_analyze_errors_on_missing_input() {
    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyze",
        "--input",
        "does-not-exist",
        "--out",
        "patchwaste-out-test",
    ]);
    cmd.assert().failure().code(1);
}
