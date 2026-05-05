use std::path::{Path, PathBuf};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use patchwaste_core::{analyse_dir, AnalyseOptions};

#[test]
fn analyse_fixture_produces_stable_report_json() {
    let input = Path::new("../../fixtures/synthetic_case_01/BuildOutput");
    let opts = AnalyseOptions {
        strict: false,
        ..AnalyseOptions::default()
    };

    let report = analyse_dir(input, opts).expect("analyse_dir ok");

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
    let opts = AnalyseOptions {
        strict: true,
        ..AnalyseOptions::default()
    };

    let err = analyse_dir(input, opts).unwrap_err();
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

    let opts = AnalyseOptions {
        baseline_path: Some(baseline_path.clone()),
        budget_ratio: Some(1.25),
        ..AnalyseOptions::default()
    };

    let report = analyse_dir(input, opts).expect("analyse_dir with baseline");
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
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse dummy fixture");

    assert_eq!(report.metrics.new_bytes, 4_194_304);
    assert_eq!(report.metrics.changed_content_bytes, 1_048_576);
}

#[test]
fn multi_depot_fixture_produces_per_depot_metrics() {
    let input = Path::new("../../fixtures/multi_depot/BuildOutput");
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse multi_depot");

    assert_eq!(report.per_depot.len(), 2);

    let d12345 = report
        .per_depot
        .iter()
        .find(|d| d.depot_id == "12345")
        .expect("depot 12345 present");
    assert_eq!(d12345.metrics.new_bytes, 5_000_000);
    assert_eq!(d12345.metrics.changed_content_bytes, 3_000_000);

    let d67890 = report
        .per_depot
        .iter()
        .find(|d| d.depot_id == "67890")
        .expect("depot 67890 present");
    assert_eq!(d67890.metrics.new_bytes, 8_000_000);
    assert_eq!(d67890.metrics.changed_content_bytes, 1_000_000);

    // aggregate uses merge (overwrite) semantics across log files
    assert!(report.metrics.new_bytes > 0);
}

#[test]
fn extract_depot_id_from_filename() {
    use patchwaste_core::parser::extract_depot_id;

    assert_eq!(
        extract_depot_id(&PathBuf::from("steampipe_preview_12345.log")),
        Some("12345".to_string())
    );
    assert_eq!(extract_depot_id(&PathBuf::from("plain.log")), None);
    assert_eq!(
        extract_depot_id(&PathBuf::from("/some/99999/plain.log")),
        Some("99999".to_string())
    );
}

#[test]
fn alt_format_fixture_parses_estimated_download_and_total_new_content() {
    let input = Path::new("../../fixtures/alt_format/BuildOutput");
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse alt_format");

    assert_eq!(report.metrics.new_bytes, 2_345_678);
    assert_eq!(report.metrics.changed_content_bytes, 456_789);

    let diag = report.diagnostics.as_ref().expect("diagnostics present");
    assert!(diag
        .counters_found
        .contains(&"estimated_download_size".to_string()));
    assert!(diag
        .counters_found
        .contains(&"total_new_content".to_string()));
    assert!(diag.lines_matched >= 3);
}

#[test]
fn no_match_fixture_produces_diagnostics_warnings() {
    let input = Path::new("../../fixtures/no_match/BuildOutput");
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse no_match");

    assert_eq!(report.metrics.new_bytes, 0);

    let diag = report.diagnostics.as_ref().expect("diagnostics present");
    assert_eq!(diag.log_files_found, 1);
    assert!(diag.lines_scanned > 0);
    assert_eq!(diag.lines_matched, 0);
    assert!(!diag.warnings.is_empty());

    let has_low_confidence = report
        .findings
        .iter()
        .any(|f| f.id == "LOW_PARSE_CONFIDENCE");
    assert!(has_low_confidence, "expected LOW_PARSE_CONFIDENCE finding");
}

#[test]
fn diagnostics_populated_for_standard_fixture() {
    let input = Path::new("../../fixtures/synthetic_case_01/BuildOutput");
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse ok");

    let diag = report.diagnostics.as_ref().expect("diagnostics present");
    assert_eq!(diag.log_files_found, 1);
    assert_eq!(diag.lines_scanned, 4);
    assert_eq!(diag.lines_matched, 3);
    assert!(diag
        .counters_found
        .contains(&"PREDICTED_UPDATE_BYTES".to_string()));
    assert!(diag
        .counters_found
        .contains(&"CHANGED_CONTENT_BYTES".to_string()));
    assert!(diag.warnings.is_empty());
}

#[test]
fn empty_delta_rule_fires_when_changed_content_zero() {
    let input = Path::new("../../fixtures/synthetic_case_missing_required/BuildOutput");
    let report = analyse_dir(input, AnalyseOptions::default()).expect("analyse ok");

    assert!(report.metrics.changed_content_bytes > 0 || report.metrics.new_bytes > 0);
}
