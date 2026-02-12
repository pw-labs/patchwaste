use std::io::Write;

use patchwaste_core::config::Config;

#[test]
fn parse_valid_toml() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
app_id = 480
depot_ids = [481, 482]
branches = ["main", "staging"]
budget_ratio = 1.25
strict = true

[depot_budgets]
"481" = 1.5
"482" = 2.0
"#
    )
    .unwrap();

    let cfg = Config::load(f.path()).unwrap();
    assert_eq!(cfg.app_id, Some(480));
    assert_eq!(cfg.depot_ids, vec![481, 482]);
    assert_eq!(cfg.branches, vec!["main", "staging"]);
    assert_eq!(cfg.budget_ratio, Some(1.25));
    assert_eq!(cfg.strict, Some(true));
    assert_eq!(cfg.depot_budgets.len(), 2);
    assert_eq!(cfg.depot_budgets["481"], 1.5);
}

#[test]
fn parse_empty_toml_gives_defaults() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "").unwrap();

    let cfg = Config::load(f.path()).unwrap();
    assert_eq!(cfg.app_id, None);
    assert!(cfg.depot_ids.is_empty());
    assert!(cfg.branches.is_empty());
    assert_eq!(cfg.budget_ratio, None);
    assert_eq!(cfg.strict, None);
    assert!(cfg.depot_budgets.is_empty());
}

#[test]
fn parse_invalid_toml_returns_error() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "this is not valid [ toml {{{{").unwrap();

    let result = Config::load(f.path());
    assert!(result.is_err());
}
