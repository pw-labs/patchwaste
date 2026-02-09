mod steampipe_log;

use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

use crate::types::FileOffender;

pub use steampipe_log::{parse_steampipe_log, SteamPipeCounters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone)]
pub struct ParsedBuildOutput {
    pub mode: ParseMode,
    pub counters: SteamPipeCounters,
    pub offenders: Vec<FileOffender>,
    pub sources: Vec<String>,
}

pub fn parse_buildoutput_dir(
    input: &Path,
    mode: ParseMode,
    max_total_bytes_scanned: u64,
) -> anyhow::Result<ParsedBuildOutput> {
    let mut counters = SteamPipeCounters::default();
    let mut offenders: Vec<FileOffender> = Vec::new();
    let mut sources: Vec<String> = Vec::new();

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

        counters.merge(parsed.counters);
        offenders.extend(parsed.offenders);
        sources.push(path.display().to_string());
    }

    offenders.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));

    if mode == ParseMode::Strict && counters.predicted_update_bytes.is_none() {
        anyhow::bail!(
            "insufficient input: missing required counter predicted_update_bytes; \
         run BestEffort mode or provide logs containing PREDICTED_UPDATE_BYTES=..."
        );
    }

    Ok(ParsedBuildOutput {
        mode,
        counters,
        offenders,
        sources,
    })
}
