use patchwaste_core::parser::{parse_steampipe_log, ParseMode, SteamPipeCounters};

#[test]
fn merge_prefers_latest_non_none() {
    let mut a = SteamPipeCounters {
        predicted_update_bytes: Some(10),
        changed_content_bytes: None,
    };
    let b = SteamPipeCounters {
        predicted_update_bytes: Some(20),
        changed_content_bytes: Some(30),
    };

    a.merge(b);

    assert_eq!(a.predicted_update_bytes, Some(20));
    assert_eq!(a.changed_content_bytes, Some(30));
}

#[test]
fn parse_pretty_update_and_offender() {
    let input = b"predicted update size: 1,234 bytes\nTOP_OFFENDER = foo.pak: 2_048\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.predicted_update_bytes, Some(1234));
    assert_eq!(parsed.offenders[0].path, "foo.pak");
    assert_eq!(parsed.offenders[0].bytes, 2048);
}

#[test]
fn parse_estimated_download_size() {
    let input = b"estimated download size for users on latest manifest: 5,678,901 bytes\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.predicted_update_bytes, Some(5_678_901));
    assert!(parsed
        .diagnostics
        .counters_found
        .contains(&"estimated_download_size".to_string()));
}

#[test]
fn parse_total_new_content() {
    let input = b"Total new content: 456,789 bytes\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.changed_content_bytes, Some(456_789));
    assert!(parsed
        .diagnostics
        .counters_found
        .contains(&"total_new_content".to_string()));
}

#[test]
fn parse_total_bytes_written() {
    let input = b"Total bytes written: 12,345,678\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.predicted_update_bytes, Some(12_345_678));
    assert!(parsed
        .diagnostics
        .counters_found
        .contains(&"total_bytes_written".to_string()));
}

#[test]
fn parse_unique_bytes() {
    let input = b"Unique bytes: 2,000,000\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.changed_content_bytes, Some(2_000_000));
    assert!(parsed
        .diagnostics
        .counters_found
        .contains(&"unique_bytes".to_string()));
}

#[test]
fn diagnostics_track_lines_and_near_misses() {
    let input = b"[Header]\nsome random line\ntotal depot bytes: 999\nPREDICTED_UPDATE_BYTES=100\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.diagnostics.lines_scanned, 4);
    assert_eq!(parsed.diagnostics.lines_matched, 1);
    assert!(parsed
        .diagnostics
        .near_miss_lines
        .iter()
        .any(|l| l.contains("total depot bytes")));
}

#[test]
fn kv_format_takes_priority_over_fallbacks() {
    let input = b"PREDICTED_UPDATE_BYTES=100\npredicted update size: 200 bytes\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert_eq!(parsed.counters.predicted_update_bytes, Some(100));
}

#[test]
fn no_data_produces_warnings() {
    let input = b"nothing useful here\njust some text\n";
    let mut r = std::io::Cursor::new(&input[..]);
    let parsed = parse_steampipe_log(&mut r, ParseMode::BestEffort).unwrap();

    assert!(parsed.counters.predicted_update_bytes.is_none());
    assert!(!parsed.diagnostics.warnings.is_empty());
}
