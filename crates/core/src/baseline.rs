use serde::{Deserialize, Serialize};
use std::path::Path;

use anyhow::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub baseline_new_bytes: u64,
}

impl Baseline {
    pub fn from_report_json(bytes: &[u8]) -> anyhow::Result<Self> {
        let v: serde_json::Value = serde_json::from_slice(bytes).context("parse json")?;
        let nb = v
            .get("metrics")
            .and_then(|m| m.get("new_bytes"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0);
        Ok(Self {
            baseline_new_bytes: nb,
        })
    }

    pub fn load_json(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        Self::from_report_json(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_from_report_json_defaults_to_zero() {
        let bytes = br#"{"metrics":{}}"#;
        let baseline = Baseline::from_report_json(bytes).unwrap();
        assert_eq!(baseline.baseline_new_bytes, 0);
    }
}
