use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub new_bytes: u64,
    pub changed_content_bytes: u64,
    pub delta_efficiency: f64,
    pub waste_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOffender {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub evidence: Vec<String>,
    pub likely_cause: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotMetrics {
    pub depot_id: String,
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseDiagnostics {
    pub log_files_found: usize,
    pub lines_scanned: usize,
    pub lines_matched: usize,
    pub counters_found: Vec<String>,
    pub warnings: Vec<String>,
    pub near_miss_lines: Vec<String>,
    pub depot_files_found: usize,
    pub depot_files_total_bytes: u64,
}
