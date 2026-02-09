use std::path::Path;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use patchwaste_core::{analyze_dir, AnalyzeOptions};

#[test]
fn analyze_fixture_produces_stable_report_json() {
    let input = Path::new("../../fixtures/synthetic_case_01/BuildOutput");
    let opts = AnalyzeOptions {
        strict: false,
        ..AnalyzeOptions::default()
    };

    let report = analyze_dir(input, opts).expect("analyze_dir ok");

    assert_eq!(report.metrics.new_bytes, 12_345_678);
    assert_eq!(report.metrics.changed_content_bytes, 2_000_000);
    assert!(report
        .inputs
        .sources
        .iter()
        .any(|s| s.contains("steampipe_preview.log")));

    insta::assert_json_snapshot!(report);
}

#[test]
fn strict_mode_requires_required_counter() {
    let input = Path::new("../../fixtures/synthetic_case_missing_required/BuildOutput");
    let opts = AnalyzeOptions {
        strict: true,
        ..AnalyzeOptions::default()
    };

    let err = analyze_dir(input, opts).unwrap_err();
    let msg = format!("{:#}", err);
    assert!(
        msg.to_lowercase().contains("missing required counter")
            || msg.to_lowercase().contains("insufficient input")
    );
}

#[test]
fn baseline_comparison_and_budget_gate_are_computed() {
    let input = Path::new("../../fixtures/synthetic_case_01/BuildOutput");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let baseline_path = std::env::temp_dir().join(format!("patchwaste-core-baseline-{nonce}.json"));
    fs::write(&baseline_path, r#"{"metrics":{"new_bytes":1000}}"#).unwrap();

    let opts = AnalyzeOptions {
        baseline_path: Some(baseline_path.clone()),
        budget_ratio: Some(1.25),
        ..AnalyzeOptions::default()
    };

    let report = analyze_dir(input, opts).expect("analyze_dir with baseline");
    let cmp = report
        .baseline_comparison
        .as_ref()
        .expect("baseline comparison present");
    let budget = report.budget.as_ref().expect("budget result present");

    assert_eq!(cmp.baseline_new_bytes, 1000);
    assert!(cmp.regression_ratio > 1.25);
    assert!(!budget.pass);

    let _ = fs::remove_file(baseline_path);
}

#[test]
fn automation_dummy_fixture_is_parseable() {
    let input = Path::new("../../fixtures/automation_dummy/BuildOutput");
    let report = analyze_dir(input, AnalyzeOptions::default()).expect("analyze dummy fixture");

    assert_eq!(report.metrics.new_bytes, 4_194_304);
    assert_eq!(report.metrics.changed_content_bytes, 1_048_576);
}
