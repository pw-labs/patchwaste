use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    baseline::Baseline,
    parser::ParseMode,
    types::{ConfidenceLevel, Finding, Metrics, Severity},
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_metadata: Option<BuildMetadata>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub per_depot: Vec<DepotReport>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotReport {
    pub depot_id: String,
    pub metrics: Metrics,
    pub confidence: ConfidenceLevel,
}

impl BuildMetadata {
    pub fn is_empty(&self) -> bool {
        self.sha.is_none() && self.branch.is_none() && self.build_id.is_none()
    }
}

impl Report {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input: &Path,
        mode: ParseMode,
        metrics: Metrics,
        confidence: ConfidenceSummary,
        findings: Vec<Finding>,
        baseline_comparison: Option<BaselineComparison>,
        budget: Option<BudgetResult>,
        build_metadata: Option<BuildMetadata>,
    ) -> Self {
        let build_metadata = build_metadata.filter(|m| !m.is_empty());
        Self {
            report_version: "1.0.0".to_string(),
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
            build_metadata,
            per_depot: Vec::new(),
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

        if !self.per_depot.is_empty() {
            s.push_str("## Per-depot metrics\n\n");
            for d in &self.per_depot {
                s.push_str(&format!("### Depot {}\n", d.depot_id));
                s.push_str(&format!("- new_bytes: `{}`\n", d.metrics.new_bytes));
                s.push_str(&format!(
                    "- changed_content_bytes: `{}`\n",
                    d.metrics.changed_content_bytes
                ));
                s.push_str(&format!("- waste_ratio: `{:.3}`\n", d.metrics.waste_ratio));
                s.push_str(&format!("- confidence: `{:?}`\n", d.confidence));
                s.push('\n');
            }
        }

        if let Some(meta) = &self.build_metadata {
            s.push_str("## Build metadata\n\n");
            if let Some(sha) = &meta.sha {
                s.push_str(&format!("- sha: `{}`\n", sha));
            }
            if let Some(branch) = &meta.branch {
                s.push_str(&format!("- branch: `{}`\n", branch));
            }
            if let Some(build_id) = &meta.build_id {
                s.push_str(&format!("- build_id: `{}`\n", build_id));
            }
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

    pub fn to_junit_xml(&self) -> String {
        let mut x = String::new();
        x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let total = self.findings.len() + 1; // +1 for budget gate testcase
        let failures: usize = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count()
            + if self.budget.as_ref().is_some_and(|b| !b.pass) {
                1
            } else {
                0
            };

        x.push_str(&format!(
            "<testsuite name=\"patchwaste\" tests=\"{}\" failures=\"{}\">\n",
            total, failures
        ));

        for f in &self.findings {
            x.push_str(&format!(
                "  <testcase name=\"{}\" classname=\"patchwaste.findings\"",
                xml_escape(&f.id)
            ));
            if f.severity == Severity::High {
                x.push_str(">\n");
                x.push_str(&format!(
                    "    <failure message=\"{}\">{}</failure>\n",
                    xml_escape(&f.likely_cause),
                    xml_escape(&f.evidence.join("; "))
                ));
                x.push_str("  </testcase>\n");
            } else {
                x.push_str(" />\n");
            }
        }

        // Budget gate testcase
        x.push_str("  <testcase name=\"budget_gate\" classname=\"patchwaste.budget\"");
        match &self.budget {
            Some(b) if !b.pass => {
                x.push_str(">\n");
                x.push_str(&format!(
                    "    <failure message=\"{}\">{}</failure>\n",
                    xml_escape(&b.reason),
                    xml_escape(&format!(
                        "regression_ratio exceeded threshold {}",
                        b.threshold_regression_ratio
                    ))
                ));
                x.push_str("  </testcase>\n");
            }
            _ => {
                x.push_str(" />\n");
            }
        }

        x.push_str("</testsuite>\n");
        x
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
            report_version: "1.0.0".to_string(),
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
            build_metadata: None,
            per_depot: Vec::new(),
        };

        let md = report.to_markdown();
        assert!(md.contains("## Metrics"));
        assert!(md.contains("## Baseline comparison"));
        assert!(md.contains("## Budget gate"));
        assert!(md.contains("### X"));
    }

    #[test]
    fn build_metadata_appears_in_markdown_when_present() {
        let report = Report::new(
            Path::new("x"),
            ParseMode::BestEffort,
            Metrics {
                new_bytes: 10,
                changed_content_bytes: 5,
                delta_efficiency: 0.5,
                waste_ratio: 0.5,
            },
            ConfidenceSummary {
                new_bytes: ConfidenceLevel::Low,
                changed_content_bytes: ConfidenceLevel::Low,
                delta_efficiency: ConfidenceLevel::Medium,
                waste_ratio: ConfidenceLevel::Medium,
                overall: ConfidenceLevel::Low,
            },
            vec![],
            None,
            None,
            Some(BuildMetadata {
                sha: Some("abc123".to_string()),
                branch: Some("main".to_string()),
                build_id: Some("42".to_string()),
            }),
        );

        let md = report.to_markdown();
        assert!(md.contains("## Build metadata"));
        assert!(md.contains("abc123"));
        assert!(md.contains("main"));
        assert!(md.contains("42"));

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("build_metadata"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn build_metadata_omitted_when_all_none() {
        let report = Report::new(
            Path::new("x"),
            ParseMode::BestEffort,
            Metrics {
                new_bytes: 10,
                changed_content_bytes: 5,
                delta_efficiency: 0.5,
                waste_ratio: 0.5,
            },
            ConfidenceSummary {
                new_bytes: ConfidenceLevel::Low,
                changed_content_bytes: ConfidenceLevel::Low,
                delta_efficiency: ConfidenceLevel::Medium,
                waste_ratio: ConfidenceLevel::Medium,
                overall: ConfidenceLevel::Low,
            },
            vec![],
            None,
            None,
            Some(BuildMetadata {
                sha: None,
                branch: None,
                build_id: None,
            }),
        );

        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("build_metadata"));
    }

    #[test]
    fn junit_xml_contains_findings_and_budget_gate() {
        let report = Report {
            report_version: "1.0.0".to_string(),
            inputs: Inputs {
                input_path: "x".to_string(),
                parse_mode: "BEST_EFFORT".to_string(),
                sources: vec![],
            },
            metrics: Metrics {
                new_bytes: 10,
                changed_content_bytes: 5,
                delta_efficiency: 0.5,
                waste_ratio: 0.5,
            },
            confidence: ConfidenceSummary {
                new_bytes: ConfidenceLevel::High,
                changed_content_bytes: ConfidenceLevel::High,
                delta_efficiency: ConfidenceLevel::Medium,
                waste_ratio: ConfidenceLevel::Medium,
                overall: ConfidenceLevel::High,
            },
            findings: vec![
                Finding {
                    id: "HIGH_WASTE_RATIO".to_string(),
                    severity: Severity::High,
                    evidence: vec!["waste_ratio=0.500".to_string()],
                    likely_cause: "churn".to_string(),
                    suggested_actions: vec![],
                },
                Finding {
                    id: "LOW_SEV".to_string(),
                    severity: Severity::Low,
                    evidence: vec![],
                    likely_cause: "minor".to_string(),
                    suggested_actions: vec![],
                },
            ],
            baseline_comparison: None,
            budget: Some(BudgetResult {
                threshold_regression_ratio: 1.0,
                pass: false,
                reason: "exceeded".to_string(),
            }),
            build_metadata: None,
            per_depot: Vec::new(),
        };

        let xml = report.to_junit_xml();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("tests=\"3\""));
        assert!(xml.contains("failures=\"2\"")); // HIGH finding + failed budget
        assert!(xml.contains("HIGH_WASTE_RATIO"));
        assert!(xml.contains("<failure"));
        assert!(xml.contains("LOW_SEV"));
        assert!(xml.contains("budget_gate"));
    }

    #[test]
    fn xml_escape_handles_special_chars() {
        assert_eq!(xml_escape("<test>&\"'"), "&lt;test&gt;&amp;&quot;&apos;");
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
