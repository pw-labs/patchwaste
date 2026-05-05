# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-05-05

### Added

- `validate` subcommand for studios to check whether patchwaste can parse their BuildOutput before wiring up CI
- `ParseDiagnostics` on every report (lines scanned, lines matched, depot file totals, near-miss lines, warnings)
- `EMPTY_DELTA` finding when `new_bytes > 0` but `changed_content_bytes = 0` (catches structural churn like the Enshrouded case)
- `LOW_PARSE_CONFIDENCE` finding when log lines were found but no SteamPipe patterns matched
- Parser tolerance for non-canonical SteamPipe log layouts via new `fixtures/alt_format/`
- New fixture `fixtures/no_match/` for the no-pattern path

### Changed

- Canonical CLI subcommand renamed from `analyze` to `analyse` (`analyze` retained as alias)
- CLI about string no longer carries `(estimated)` suffix; confidence framing moved to README

## [0.1.0] - 2026-03-21

### Added

- SteamPipe BuildOutput log parser with waste metrics (new_bytes, changed_content_bytes, delta_efficiency, waste_ratio)
- Baseline comparison and budget gating (exit 0 pass, exit 2 fail)
- JSON and Markdown report output
- R1 (HIGH_WASTE_RATIO) and R2 (LARGE_TOP_OFFENDER) detection rules
- Per-depot metrics for multi-depot builds
- `patchwaste.toml` configuration support
- JUnit XML output for CI integration
- GitLab CI template (`examples/gitlab-ci.yml`)
- GitHub Actions CI workflow (fmt, clippy, tests)
- Property tests and fuzz-style input testing
- Clone-to-contribute workflow (`scripts/bootstrap-dev.sh`, `scripts/verify.sh`, pinned toolchain)
- Governance files (LICENSE, CONTRIBUTING, SECURITY, SUPPORT, TRADEMARK, NOTICE)
- Issue templates (bug report, feature request, commercial interest, usage report)

[0.2.0]: https://github.com/pw-labs/patchwaste/releases/tag/v0.2.0
[0.1.0]: https://github.com/pw-labs/patchwaste/releases/tag/v0.1.0
