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
