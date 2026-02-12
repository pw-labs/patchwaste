pub mod baseline;
pub mod config;
pub mod parser;
pub mod report;
pub mod rules;
pub mod types;

use std::path::Path;

use anyhow::Context;

use crate::{
    baseline::Baseline,
    parser::ParseMode,
    report::{BudgetResult, DepotReport, Report},
    rules::run_rules,
    types::{ConfidenceLevel, Metrics},
};

#[derive(Debug, Clone)]
pub struct AnalyzeOptions {
    pub strict: bool,
    pub baseline_path: Option<std::path::PathBuf>,
    pub budget_ratio: Option<f64>,
    pub max_total_bytes_scanned: u64,
    pub build_metadata: Option<report::BuildMetadata>,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            strict: false,
            baseline_path: None,
            budget_ratio: None,
            max_total_bytes_scanned: 50 * 1024 * 1024,
            build_metadata: None,
        }
    }
}

pub fn analyze_dir(input: &Path, opts: AnalyzeOptions) -> anyhow::Result<Report> {
    let parse_mode = if opts.strict {
        ParseMode::Strict
    } else {
        ParseMode::BestEffort
    };

    let parsed = parser::parse_buildoutput_dir(input, parse_mode, opts.max_total_bytes_scanned)
        .with_context(|| format!("failed to parse BuildOutput at {}", input.display()))?;

    let (metrics, confidence) = compute_metrics(&parsed);

    let findings = run_rules(&parsed, &metrics);

    let baseline = if let Some(p) = &opts.baseline_path {
        Some(
            Baseline::load_json(p)
                .with_context(|| format!("failed to load baseline {}", p.display()))?,
        )
    } else {
        None
    };

    let baseline_comparison = baseline
        .as_ref()
        .map(|b| report::compare_to_baseline(b, &metrics));

    let budget = match (
        opts.budget_ratio,
        baseline.as_ref(),
        baseline_comparison.as_ref(),
    ) {
        (Some(threshold), Some(_), Some(cmp)) => {
            let pass = cmp.regression_ratio <= threshold;
            Some(BudgetResult {
                threshold_regression_ratio: threshold,
                pass,
                reason: if pass {
                    "within regression budget".to_string()
                } else {
                    format!(
                        "regression_ratio {:.3} exceeds threshold {:.3}",
                        cmp.regression_ratio, threshold
                    )
                },
            })
        }
        _ => None,
    };

    let per_depot: Vec<DepotReport> = parsed
        .per_depot
        .iter()
        .map(|d| {
            let depot_parsed = parser::ParsedBuildOutput {
                mode: parse_mode,
                counters: d.counters.clone(),
                offenders: d.offenders.clone(),
                sources: vec![],
                per_depot: vec![],
            };
            let (depot_metrics, depot_confidence) = compute_metrics(&depot_parsed);
            DepotReport {
                depot_id: d.depot_id.clone(),
                metrics: depot_metrics,
                confidence: depot_confidence.overall,
            }
        })
        .collect();

    let mut report = Report::new(
        input,
        parse_mode,
        metrics,
        confidence,
        findings,
        baseline_comparison,
        budget,
        opts.build_metadata,
    );
    report.inputs.sources = parsed.sources;
    report.per_depot = per_depot;

    Ok(report)
}

pub fn compute_metrics(parsed: &parser::ParsedBuildOutput) -> (Metrics, report::ConfidenceSummary) {
    let mut new_bytes = parsed.counters.predicted_update_bytes;
    let mut changed_content_bytes = parsed.counters.changed_content_bytes;

    let mut new_conf = ConfidenceLevel::Low;
    let mut changed_conf = ConfidenceLevel::Low;

    if let Some(v) = new_bytes {
        if v > 0 {
            new_conf = ConfidenceLevel::High;
        }
    }

    if let Some(v) = changed_content_bytes {
        if v > 0 {
            changed_conf = ConfidenceLevel::High;
        }
    }

    if let (Some(0), Some(cb)) = (new_bytes, changed_content_bytes) {
        if cb > 0 {
            new_bytes = Some(cb);
            new_conf = ConfidenceLevel::Low;
        }
    }

    match (new_bytes, changed_content_bytes) {
        (Some(nb), None) => {
            changed_content_bytes = Some(nb);
            changed_conf = ConfidenceLevel::Low;
        }
        (None, Some(cb)) => {
            new_bytes = Some(cb);
            new_conf = ConfidenceLevel::Low;
        }
        (None, None) => {
            new_bytes = Some(0);
            changed_content_bytes = Some(0);
        }
        _ => {}
    }

    let nb = new_bytes.unwrap_or(0);
    let cb = changed_content_bytes.unwrap_or(0);

    let delta_efficiency = if nb == 0 {
        1.0
    } else {
        (cb as f64) / (nb as f64)
    };
    let delta_efficiency = delta_efficiency.clamp(0.0, 1.0);
    let waste_ratio = (1.0 - delta_efficiency).clamp(0.0, 1.0);

    let metrics = Metrics {
        new_bytes: nb,
        changed_content_bytes: cb,
        delta_efficiency,
        waste_ratio,
    };

    let confidence = report::ConfidenceSummary {
        new_bytes: new_conf,
        changed_content_bytes: changed_conf,
        delta_efficiency: ConfidenceLevel::Medium,
        waste_ratio: ConfidenceLevel::Medium,
        overall: confidence_overall(new_conf, changed_conf),
    };

    (metrics, confidence)
}

fn confidence_overall(a: ConfidenceLevel, b: ConfidenceLevel) -> ConfidenceLevel {
    use ConfidenceLevel::*;
    match (a, b) {
        (High, High) => High,
        (High, Medium) | (Medium, High) | (Medium, Medium) => Medium,
        _ => Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParseMode, ParsedBuildOutput, SteamPipeCounters};

    #[test]
    fn metrics_handle_zero_new_bytes_with_positive_changed_bytes() {
        let parsed = ParsedBuildOutput {
            mode: ParseMode::BestEffort,
            counters: SteamPipeCounters {
                predicted_update_bytes: Some(0),
                changed_content_bytes: Some(1024),
            },
            offenders: vec![],
            sources: vec!["x.log".to_string()],
            per_depot: vec![],
        };

        let (metrics, confidence) = compute_metrics(&parsed);

        assert_eq!(metrics.new_bytes, 1024);
        assert_eq!(metrics.changed_content_bytes, 1024);
        assert!(metrics.delta_efficiency > 0.0);
        assert_eq!(confidence.new_bytes, ConfidenceLevel::Low);
    }
}
