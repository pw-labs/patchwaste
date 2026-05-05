use std::io::BufRead;

use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{FileOffender, ParseDiagnostics};

use super::ParseMode;

#[derive(Debug, Clone, Default)]
pub struct SteamPipeCounters {
    pub predicted_update_bytes: Option<u64>,
    pub changed_content_bytes: Option<u64>,
}

impl SteamPipeCounters {
    pub fn merge(&mut self, other: SteamPipeCounters) {
        if other.predicted_update_bytes.is_some() {
            self.predicted_update_bytes = other.predicted_update_bytes;
        }
        if other.changed_content_bytes.is_some() {
            self.changed_content_bytes = other.changed_content_bytes;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedSteamPipeLog {
    pub counters: SteamPipeCounters,
    pub offenders: Vec<FileOffender>,
    pub diagnostics: ParseDiagnostics,
}

static RE_KV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(PREDICTED_UPDATE_BYTES|CHANGED_CONTENT_BYTES)\s*=\s*([0-9][0-9_]*)\b")
        .expect("valid regex")
});

static RE_PRETTY_UPDATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)predicted update size\s*:\s*([0-9][0-9,]*)\s*bytes").expect("valid regex")
});

static RE_ESTIMATED_DOWNLOAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)estimated download size[^:]*:\s*([0-9][0-9,]*)\s*bytes").expect("valid regex")
});

static RE_TOTAL_NEW_CONTENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)total new content\s*:\s*([0-9][0-9,]*)\s*bytes").expect("valid regex")
});

static RE_TOTAL_BYTES_WRITTEN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)total bytes written\s*:\s*([0-9][0-9,]*)").expect("valid regex"));

static RE_UNIQUE_BYTES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)unique bytes\s*:\s*([0-9][0-9,]*)").expect("valid regex"));

static RE_OFFENDER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bTOP_OFFENDER\s*=\s*(.+?)\s*:\s*([0-9][0-9_]*)\s*$").expect("valid regex")
});

static RE_NEAR_MISS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(bytes|update.size|download.size|content.size|depot.size|patch.size|delta|chunk)",
    )
    .expect("valid regex")
});

fn parse_comma_number(s: &str) -> Option<u64> {
    s.replace([',', '_'], "").parse::<u64>().ok()
}

pub fn parse_steampipe_log<R: BufRead>(
    r: &mut R,
    mode: ParseMode,
) -> anyhow::Result<ParsedSteamPipeLog> {
    let mut counters = SteamPipeCounters::default();
    let mut offenders: Vec<FileOffender> = Vec::new();
    let mut diag = ParseDiagnostics::default();

    let mut line = String::new();
    loop {
        line.clear();
        let n = r.read_line(&mut line).context("read_line")?;
        if n == 0 {
            break;
        }

        diag.lines_scanned += 1;
        let mut matched = false;

        if let Some(cap) = RE_KV.captures(&line) {
            let key = cap.get(1).unwrap().as_str().to_ascii_uppercase();
            let val = cap.get(2).unwrap().as_str().replace('_', "");
            let num: u64 = val.parse().unwrap_or(0);

            match key.as_str() {
                "PREDICTED_UPDATE_BYTES" => {
                    counters.predicted_update_bytes = Some(num);
                    diag.counters_found
                        .push("PREDICTED_UPDATE_BYTES".to_string());
                }
                "CHANGED_CONTENT_BYTES" => {
                    counters.changed_content_bytes = Some(num);
                    diag.counters_found
                        .push("CHANGED_CONTENT_BYTES".to_string());
                }
                _ => {}
            }
            matched = true;
        }

        if counters.predicted_update_bytes.is_none() {
            if let Some(cap) = RE_PRETTY_UPDATE.captures(&line) {
                if let Some(num) = parse_comma_number(cap.get(1).unwrap().as_str()) {
                    counters.predicted_update_bytes = Some(num);
                    diag.counters_found
                        .push("predicted_update_size".to_string());
                    matched = true;
                }
            }
        }

        if counters.predicted_update_bytes.is_none() {
            if let Some(cap) = RE_ESTIMATED_DOWNLOAD.captures(&line) {
                if let Some(num) = parse_comma_number(cap.get(1).unwrap().as_str()) {
                    counters.predicted_update_bytes = Some(num);
                    diag.counters_found
                        .push("estimated_download_size".to_string());
                    matched = true;
                }
            }
        }

        if counters.predicted_update_bytes.is_none() {
            if let Some(cap) = RE_TOTAL_BYTES_WRITTEN.captures(&line) {
                if let Some(num) = parse_comma_number(cap.get(1).unwrap().as_str()) {
                    counters.predicted_update_bytes = Some(num);
                    diag.counters_found.push("total_bytes_written".to_string());
                    matched = true;
                }
            }
        }

        if counters.changed_content_bytes.is_none() {
            if let Some(cap) = RE_TOTAL_NEW_CONTENT.captures(&line) {
                if let Some(num) = parse_comma_number(cap.get(1).unwrap().as_str()) {
                    counters.changed_content_bytes = Some(num);
                    diag.counters_found.push("total_new_content".to_string());
                    matched = true;
                }
            }
        }

        if counters.changed_content_bytes.is_none() {
            if let Some(cap) = RE_UNIQUE_BYTES.captures(&line) {
                if let Some(num) = parse_comma_number(cap.get(1).unwrap().as_str()) {
                    counters.changed_content_bytes = Some(num);
                    diag.counters_found.push("unique_bytes".to_string());
                    matched = true;
                }
            }
        }

        if let Some(cap) = RE_OFFENDER.captures(&line) {
            let path = cap.get(1).unwrap().as_str().trim().to_string();
            let raw = cap.get(2).unwrap().as_str().replace('_', "");
            let bytes = raw.parse::<u64>().unwrap_or(0);
            offenders.push(FileOffender { path, bytes });
            matched = true;
        }

        if matched {
            diag.lines_matched += 1;
        } else if RE_NEAR_MISS.is_match(&line) {
            let trimmed = line.trim().to_string();
            if !trimmed.is_empty()
                && !trimmed.starts_with('[')
                && !trimmed.starts_with('#')
                && diag.near_miss_lines.len() < 10
            {
                diag.near_miss_lines.push(trimmed);
            }
        }
    }

    if counters.predicted_update_bytes.is_none() {
        diag.warnings.push(
            "No PREDICTED_UPDATE_BYTES or equivalent counter found. \
             Expected format: PREDICTED_UPDATE_BYTES=<number> or \
             'predicted update size: <number> bytes'."
                .to_string(),
        );
    }
    if counters.changed_content_bytes.is_none() {
        diag.warnings.push(
            "No CHANGED_CONTENT_BYTES or equivalent counter found. \
             Waste ratio accuracy will be reduced."
                .to_string(),
        );
    }

    if mode == ParseMode::Strict && counters.predicted_update_bytes.is_none() {
        anyhow::bail!("missing required counter PREDICTED_UPDATE_BYTES");
    }

    Ok(ParsedSteamPipeLog {
        counters,
        offenders,
        diagnostics: diag,
    })
}
