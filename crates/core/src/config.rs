use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub app_id: Option<u64>,
    pub depot_ids: Vec<u64>,
    pub branches: Vec<String>,
    pub budget_ratio: Option<f64>,
    pub strict: Option<bool>,
    pub depot_budgets: HashMap<String, f64>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let contents =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
        Ok(config)
    }

    pub fn discover() -> Option<Self> {
        let path = Path::new("patchwaste.toml");
        if path.exists() {
            Config::load(path).ok()
        } else {
            None
        }
    }
}
