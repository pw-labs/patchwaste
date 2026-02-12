mod steampipe_log;

use std::collections::HashMap;
use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

use crate::types::FileOffender;

pub use steampipe_log::{parse_steampipe_log, SteamPipeCounters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone)]
pub struct DepotOutput {
    pub depot_id: String,
    pub counters: SteamPipeCounters,
    pub offenders: Vec<FileOffender>,
}

#[derive(Debug, Clone)]
pub struct ParsedBuildOutput {
    pub mode: ParseMode,
    pub counters: SteamPipeCounters,
    pub offenders: Vec<FileOffender>,
    pub sources: Vec<String>,
    pub per_depot: Vec<DepotOutput>,
}

static RE_DEPOT_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d{5,})").expect("valid regex"));

pub fn extract_depot_id(path: &Path) -> Option<String> {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if let Some(cap) = RE_DEPOT_ID.captures(stem) {
        return Some(cap.get(1).unwrap().as_str().to_string());
    }
    if let Some(parent) = path.parent() {
        let dir_name = parent.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if let Some(cap) = RE_DEPOT_ID.captures(dir_name) {
            return Some(cap.get(1).unwrap().as_str().to_string());
        }
    }
    None
}

pub fn parse_buildoutput_dir(
    input: &Path,
    mode: ParseMode,
    max_total_bytes_scanned: u64,
) -> anyhow::Result<ParsedBuildOutput> {
    let mut counters = SteamPipeCounters::default();
    let mut offenders: Vec<FileOffender> = Vec::new();
    let mut sources: Vec<String> = Vec::new();
    let mut depot_map: HashMap<String, (SteamPipeCounters, Vec<FileOffender>)> = HashMap::new();

    let mut scanned: u64 = 0;

    for entry in WalkDir::new(input).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !(ext == "log" || ext == "txt") {
            continue;
        }

        let meta = std::fs::metadata(path)?;
        let len = meta.len();
        if scanned.saturating_add(len) > max_total_bytes_scanned {
            break;
        }
        scanned += len;

        let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mut reader = BufReader::new(f);

        let parsed = parse_steampipe_log(&mut reader, mode)
            .with_context(|| format!("parse log {}", path.display()))?;

        counters.merge(parsed.counters.clone());
        offenders.extend(parsed.offenders.clone());
        sources.push(path.display().to_string());

        if let Some(depot_id) = extract_depot_id(path) {
            let entry = depot_map
                .entry(depot_id)
                .or_insert_with(|| (SteamPipeCounters::default(), Vec::new()));
            entry.0.merge(parsed.counters);
            entry.1.extend(parsed.offenders);
        }
    }

    offenders.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));

    if mode == ParseMode::Strict && counters.predicted_update_bytes.is_none() {
        anyhow::bail!(
            "insufficient input: missing required counter predicted_update_bytes; \
         run BestEffort mode or provide logs containing PREDICTED_UPDATE_BYTES=..."
        );
    }

    let mut per_depot: Vec<DepotOutput> = depot_map
        .into_iter()
        .map(|(depot_id, (counters, offenders))| DepotOutput {
            depot_id,
            counters,
            offenders,
        })
        .collect();
    per_depot.sort_by(|a, b| a.depot_id.cmp(&b.depot_id));

    Ok(ParsedBuildOutput {
        mode,
        counters,
        offenders,
        sources,
        per_depot,
    })
}
