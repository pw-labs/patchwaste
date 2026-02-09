use std::io::BufRead;

use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::FileOffender;

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
}

static RE_KV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(PREDICTED_UPDATE_BYTES|CHANGED_CONTENT_BYTES)\s*=\s*([0-9][0-9_]*)\b")
        .expect("valid regex")
});

static RE_PRETTY_UPDATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)predicted update size\s*:\s*([0-9][0-9,]*)\s*bytes").expect("valid regex")
});

static RE_OFFENDER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bTOP_OFFENDER\s*=\s*(.+?)\s*:\s*([0-9][0-9_]*)\s*$").expect("valid regex")
});

pub fn parse_steampipe_log<R: BufRead>(
    r: &mut R,
    mode: ParseMode,
) -> anyhow::Result<ParsedSteamPipeLog> {
    let mut counters = SteamPipeCounters::default();
    let mut offenders: Vec<FileOffender> = Vec::new();

    let mut line = String::new();
    loop {
        line.clear();
        let n = r.read_line(&mut line).context("read_line")?;
        if n == 0 {
            break;
        }

        if let Some(cap) = RE_KV.captures(&line) {
            let key = cap.get(1).unwrap().as_str().to_ascii_uppercase();
            let val = cap.get(2).unwrap().as_str().replace('_', "");
            let num: u64 = val.parse().unwrap_or(0);

            match key.as_str() {
                "PREDICTED_UPDATE_BYTES" => counters.predicted_update_bytes = Some(num),
                "CHANGED_CONTENT_BYTES" => counters.changed_content_bytes = Some(num),
                _ => {}
            }
        }

        if counters.predicted_update_bytes.is_none() {
            if let Some(cap) = RE_PRETTY_UPDATE.captures(&line) {
                let raw = cap.get(1).unwrap().as_str().replace(',', "");
                if let Ok(num) = raw.parse::<u64>() {
                    counters.predicted_update_bytes = Some(num);
                }
            }
        }

        if let Some(cap) = RE_OFFENDER.captures(&line) {
            let path = cap.get(1).unwrap().as_str().trim().to_string();
            let raw = cap.get(2).unwrap().as_str().replace('_', "");
            let bytes = raw.parse::<u64>().unwrap_or(0);
            offenders.push(FileOffender { path, bytes });
        }
    }

    if mode == ParseMode::Strict && counters.predicted_update_bytes.is_none() {
        anyhow::bail!("missing required counter PREDICTED_UPDATE_BYTES");
    }

    Ok(ParsedSteamPipeLog {
        counters,
        offenders,
    })
}
