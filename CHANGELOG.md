# Changelog

All notable changes to this project will be documented in this file.

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

[0.1.0]: https://github.com/pw-labs/patchwaste/releases/tag/v0.1.0
