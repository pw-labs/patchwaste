use crate::{
    parser::ParsedBuildOutput,
    types::Metrics,
    types::{Finding, Severity},
};

pub fn run_rules(parsed: &ParsedBuildOutput, metrics: &Metrics) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    if metrics.waste_ratio >= 0.50 && metrics.new_bytes > 0 {
        findings.push(Finding {
            id: "HIGH_WASTE_RATIO".to_string(),
            severity: Severity::High,
            evidence: vec![format!("waste_ratio={:.3}", metrics.waste_ratio)],
            likely_cause: "Large packed file churn or content reorder causing many new chunks"
                .to_string(),
            suggested_actions: vec![
                "Avoid reordering assets inside large packed files between builds".to_string(),
                "Split packs by level/realm to localize churn".to_string(),
                "Align pack layout to stable boundaries (e.g., 1MB) where applicable".to_string(),
            ],
        });
    }

    if let Some(off) = parsed.offenders.first() {
        if off.bytes >= 100 * 1024 * 1024 {
            findings.push(Finding {
                id: "LARGE_TOP_OFFENDER".to_string(),
                severity: Severity::Medium,
                evidence: vec![format!("{} ({} bytes)", off.path, off.bytes)],
                likely_cause: "A large file dominates predicted update size".to_string(),
                suggested_actions: vec![
                    "If this is a pack file, consider splitting into multiple packs".to_string(),
                    "Ensure build process does not rewrite the whole file for small changes"
                        .to_string(),
                ],
            });
        }
    }

    if metrics.new_bytes > 0 && metrics.changed_content_bytes == 0 {
        findings.push(Finding {
            id: "EMPTY_DELTA".to_string(),
            severity: Severity::High,
            evidence: vec![format!(
                "new_bytes={} but changed_content_bytes=0",
                metrics.new_bytes
            )],
            likely_cause:
                "Update contains data but no meaningful content change was detected. \
                           The entire update may be structural churn (repack, reorder, or metadata)"
                    .to_string(),
            suggested_actions: vec![
                "Check whether the build process is rewriting paks without content changes"
                    .to_string(),
                "Verify that CHANGED_CONTENT_BYTES is being reported in the SteamPipe log"
                    .to_string(),
            ],
        });
    }

    if parsed.diagnostics.lines_scanned > 0 && parsed.diagnostics.lines_matched == 0 {
        findings.push(Finding {
            id: "LOW_PARSE_CONFIDENCE".to_string(),
            severity: Severity::Medium,
            evidence: vec![format!(
                "{} lines scanned, 0 matched known patterns",
                parsed.diagnostics.lines_scanned
            )],
            likely_cause:
                "Log files were found but no known SteamPipe output patterns were recognised. \
                 Metrics are derived from fallback heuristics and may not reflect actual patch size"
                    .to_string(),
            suggested_actions: vec![
                "Run 'patchwaste validate --input <path>' to inspect what was found".to_string(),
                "Check that your BuildOutput contains SteamPipe preview build logs".to_string(),
                "Open an issue with a (redacted) sample of your log format".to_string(),
            ],
        });
    }

    findings.sort_by(|a, b| a.id.cmp(&b.id));
    findings
}
