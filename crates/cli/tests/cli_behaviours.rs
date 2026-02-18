use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn cli_analyse_writes_reports_and_exits_0_without_budget() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/synthetic_case_01/BuildOutput");

    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyse",
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
fn cli_analyse_exits_2_when_budget_fails() {
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
        "analyse",
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
fn cli_analyse_writes_junit_xml_when_requested() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/synthetic_case_01/BuildOutput");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out_dir = format!("patchwaste-out-junit-{nonce}");

    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyse",
        "--input",
        fixture_path.to_str().unwrap(),
        "--output-format",
        "all",
        "--out",
        &out_dir,
    ]);

    cmd.assert().success();

    let out_path = std::path::Path::new(&out_dir);
    assert!(out_path.join("report.json").exists());
    assert!(out_path.join("report.md").exists());
    assert!(out_path.join("report.xml").exists());

    let xml = fs::read_to_string(out_path.join("report.xml")).unwrap();
    assert!(xml.contains("<testsuite"));
    assert!(xml.contains("HIGH_WASTE_RATIO"));
    assert!(xml.contains("budget_gate"));

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn cli_analyse_errors_on_missing_input() {
    let mut cmd = cargo_bin_cmd!("patchwaste");
    cmd.args([
        "analyse",
        "--input",
        "does-not-exist",
        "--out",
        "patchwaste-out-test",
    ]);
    cmd.assert().failure().code(1);
}
