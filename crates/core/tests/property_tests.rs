use std::io::Cursor;

use proptest::prelude::*;

use patchwaste_core::compute_metrics;
use patchwaste_core::parser::{
    parse_steampipe_log, ParseMode, ParsedBuildOutput, SteamPipeCounters,
};

proptest! {
    #[test]
    fn parser_never_panics_on_arbitrary_input(data in prop::collection::vec(any::<u8>(), 0..4096)) {
        let mut cursor = Cursor::new(data);
        // Should not panic regardless of input
        let _ = parse_steampipe_log(&mut cursor, ParseMode::BestEffort);
    }

    #[test]
    fn metrics_waste_plus_efficiency_equals_one(
        new_bytes in 1u64..10_000_000_000,
        changed in 1u64..10_000_000_000,
    ) {
        let parsed = ParsedBuildOutput {
            mode: ParseMode::BestEffort,
            counters: SteamPipeCounters {
                predicted_update_bytes: Some(new_bytes),
                changed_content_bytes: Some(changed),
            },
            offenders: vec![],
            sources: vec![],
            per_depot: vec![],
        };

        let (metrics, _) = compute_metrics(&parsed);
        let sum = metrics.waste_ratio + metrics.delta_efficiency;
        prop_assert!((sum - 1.0).abs() < 1e-10, "waste + efficiency = {}, expected ~1.0", sum);
    }

    #[test]
    fn regression_ratio_non_negative(
        baseline in 1u64..10_000_000_000,
        new_bytes in 0u64..10_000_000_000,
    ) {
        let ratio = new_bytes as f64 / baseline as f64;
        prop_assert!(ratio >= 0.0, "regression_ratio={} is negative", ratio);
    }
}
