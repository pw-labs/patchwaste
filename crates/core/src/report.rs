use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    baseline::Baseline,
    parser::ParseMode,
    types::{ConfidenceLevel, Finding, Metrics},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub report_version: String,
    pub inputs: Inputs,
    pub metrics: Metrics,
    pub confidence: ConfidenceSummary,
    pub findings: Vec<Finding>,
    pub baseline_comparison: Option<BaselineComparison>,
    pub budget: Option<BudgetResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inputs {
    pub input_path: String,
    pub parse_mode: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceSummary {
    pub new_bytes: ConfidenceLevel,
    pub changed_content_bytes: ConfidenceLevel,
    pub delta_efficiency: ConfidenceLevel,
    pub waste_ratio: ConfidenceLevel,
    pub overall: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_new_bytes: u64,
    pub regression_ratio: f64,
    pub delta_new_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetResult {
    pub threshold_regression_ratio: f64,
    pub pass: bool,
    pub reason: String,
}

impl Report {
    pub fn new(
        input: &Path,
        mode: ParseMode,
        metrics: Metrics,
        confidence: ConfidenceSummary,
        findings: Vec<Finding>,
        baseline_comparison: Option<BaselineComparison>,
        budget: Option<BudgetResult>,
    ) -> Self {
        Self {
            report_version: "0.1.0".to_string(),
            inputs: Inputs {
                input_path: input.display().to_string(),
                parse_mode: match mode {
                    ParseMode::Strict => "STRICT".to_string(),
                    ParseMode::BestEffort => "BEST_EFFORT".to_string(),
                },
                sources: vec![],
            },
            metrics,
            confidence,
            findings,
            baseline_comparison,
            budget,
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str("# patchwaste report\n\n");
        s.push_str(&format!("- report_version: `{}`\n", self.report_version));
        s.push_str(&format!("- input_path: `{}`\n", self.inputs.input_path));
        s.push_str(&format!("- parse_mode: `{}`\n", self.inputs.parse_mode));
        s.push('\n');

        s.push_str("## Metrics\n\n");
        s.push_str(&format!("- new_bytes: `{}`\n", self.metrics.new_bytes));
        s.push_str(&format!(
            "- changed_content_bytes: `{}`\n",
            self.metrics.changed_content_bytes
        ));
        s.push_str(&format!(
            "- delta_efficiency: `{:.3}`\n",
            self.metrics.delta_efficiency
        ));
        s.push_str(&format!(
            "- waste_ratio: `{:.3}`\n",
            self.metrics.waste_ratio
        ));
        s.push('\n');

        if let Some(cmp) = &self.baseline_comparison {
            s.push_str("## Baseline comparison\n\n");
            s.push_str(&format!(
                "- baseline_new_bytes: `{}`\n",
                cmp.baseline_new_bytes
            ));
            s.push_str(&format!("- delta_new_bytes: `{}`\n", cmp.delta_new_bytes));
            s.push_str(&format!(
                "- regression_ratio: `{:.3}`\n",
                cmp.regression_ratio
            ));
            s.push('\n');
        }

        if let Some(b) = &self.budget {
            s.push_str("## Budget gate\n\n");
            s.push_str(&format!(
                "- threshold_regression_ratio: `{:.3}`\n",
                b.threshold_regression_ratio
            ));
            s.push_str(&format!("- pass: `{}`\n", b.pass));
            s.push_str(&format!("- reason: `{}`\n", b.reason));
            s.push('\n');
        }

        s.push_str("## Findings\n\n");
        if self.findings.is_empty() {
            s.push_str("- (none)\n");
        } else {
            for f in &self.findings {
                s.push_str(&format!("### {}\n", f.id));
                s.push_str(&format!("- severity: `{:?}`\n", f.severity));
                s.push_str(&format!("- likely_cause: {}\n", f.likely_cause));
                if !f.evidence.is_empty() {
                    s.push_str("- evidence:\n");
                    for e in &f.evidence {
                        s.push_str(&format!("  - {}\n", e));
                    }
                }
                if !f.suggested_actions.is_empty() {
                    s.push_str("- suggested_actions:\n");
                    for a in &f.suggested_actions {
                        s.push_str(&format!("  - {}\n", a));
                    }
                }
                s.push('\n');
            }
        }

        s
    }
}

pub fn compare_to_baseline(b: &Baseline, metrics: &Metrics) -> BaselineComparison {
    let baseline = b.baseline_new_bytes;
    let regression_ratio = if baseline == 0 {
        if metrics.new_bytes == 0 {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        metrics.new_bytes as f64 / baseline as f64
    };

    BaselineComparison {
        baseline_new_bytes: baseline,
        regression_ratio,
        delta_new_bytes: metrics.new_bytes as i64 - baseline as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConfidenceLevel, Finding, Metrics, Severity};

    #[test]
    fn markdown_includes_sections_and_findings() {
        let report = Report {
            report_version: "0.1.0".to_string(),
            inputs: Inputs {
                input_path: "x".to_string(),
                parse_mode: "STRICT".to_string(),
                sources: vec!["a.log".to_string()],
            },
            metrics: Metrics {
                new_bytes: 10,
                changed_content_bytes: 5,
                delta_efficiency: 0.5,
                waste_ratio: 0.5,
            },
            confidence: ConfidenceSummary {
                new_bytes: ConfidenceLevel::Low,
                changed_content_bytes: ConfidenceLevel::Low,
                delta_efficiency: ConfidenceLevel::Medium,
                waste_ratio: ConfidenceLevel::Medium,
                overall: ConfidenceLevel::Low,
            },
            findings: vec![Finding {
                id: "X".to_string(),
                severity: Severity::High,
                evidence: vec!["e".to_string()],
                likely_cause: "c".to_string(),
                suggested_actions: vec!["a".to_string()],
            }],
            baseline_comparison: Some(BaselineComparison {
                baseline_new_bytes: 1,
                regression_ratio: 10.0,
                delta_new_bytes: 9,
            }),
            budget: Some(BudgetResult {
                threshold_regression_ratio: 1.0,
                pass: false,
                reason: "nope".to_string(),
            }),
        };

        let md = report.to_markdown();
        assert!(md.contains("## Metrics"));
        assert!(md.contains("## Baseline comparison"));
        assert!(md.contains("## Budget gate"));
        assert!(md.contains("### X"));
    }

    #[test]
    fn baseline_comparison_infinite_when_baseline_zero() {
        let b = Baseline {
            baseline_new_bytes: 0,
        };
        let m = Metrics {
            new_bytes: 10,
            changed_content_bytes: 10,
            delta_efficiency: 1.0,
            waste_ratio: 0.0,
        };
        let cmp = compare_to_baseline(&b, &m);
        assert!(cmp.regression_ratio.is_infinite());
    }
}
