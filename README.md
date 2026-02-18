<p align="center">
  <img src="logo.svg" alt="patch|waste" width="480">
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/model-open--core-success" alt="Open Core">
  <img src="https://img.shields.io/badge/focus-Unreal%20%2B%20Steam-orange" alt="Focus">
</p>

<p align="center">
  Rust CLI + GitHub Actions gate to detect patch-size regressions from SteamPipe preview build output.
</p>

---

## ELI5

You ship a game update. Steam patch is much bigger than expected. `patchwaste` tells you if your build changed too much data for too little real content change.

Think of it like this: if you changed one sentence in a book, but the printer made everyone download half the book again, that's waste. This tool measures that waste and gives a red/green signal for CI.

## Quickstart

```bash
cargo run -p patchwaste -- analyse --input fixtures/synthetic_case_01/BuildOutput --out patchwaste-out
cat patchwaste-out/report.md
```

## Clone and start contributing

```bash
git clone https://github.com/patchwaste/patchwaste.git
cd patchwaste
./scripts/bootstrap-dev.sh
./scripts/verify.sh
```

This gives you:
- pinned Rust toolchain (`rust-toolchain.toml`)
- local git hooks (`.githooks/pre-commit`)
- one command (`./scripts/verify.sh`) matching CI checks

Example output:

```
# patchwaste report

- report_version: `0.1.0`
- input_path: `fixtures/synthetic_case_01/BuildOutput`
- parse_mode: `BEST_EFFORT`

## Metrics

- new_bytes: `12345678`
- changed_content_bytes: `2000000`
- delta_efficiency: `0.162`
- waste_ratio: `0.838`

## Findings

### HIGH_WASTE_RATIO
- severity: `High`
- likely_cause: Large packed file churn or content reorder causing many new chunks
- evidence:
  - waste_ratio=0.838
- suggested_actions:
  - Avoid reordering assets inside large packed files between builds
  - Split packs by level/realm to localize churn
  - Align pack layout to stable boundaries (e.g., 1MB) where applicable

### LARGE_TOP_OFFENDER
- severity: `Medium`
- likely_cause: A large file dominates predicted update size
- evidence:
  - GameContent.pak (800000000 bytes)
- suggested_actions:
  - If this is a pack file, consider splitting into multiple packs
  - Ensure build process does not rewrite the whole file for small changes
```

## Full E2E example (baseline + compare + budget gate)

Use the included automation-safe dummy fixture:

```bash
# 1) Create baseline
cargo run -p patchwaste -- analyse \
  --input fixtures/automation_dummy/BuildOutput \
  --out patchwaste-out
cp patchwaste-out/report.json baseline.json

# 2) Compare against baseline (expected pass, exit 0)
cargo run -p patchwaste -- analyse \
  --input fixtures/automation_dummy/BuildOutput \
  --baseline baseline.json \
  --budget-ratio 1.25 \
  --out patchwaste-out-compare
echo $?   # 0

# 3) Simulate failing budget gate by using a tiny baseline (expected exit 2)
printf '{"metrics":{"new_bytes":1000}}\n' > baseline-small.json
cargo run -p patchwaste -- analyse \
  --input fixtures/automation_dummy/BuildOutput \
  --baseline baseline-small.json \
  --budget-ratio 1.25 \
  --out patchwaste-out-fail
echo $?   # 2
```

## Project layout

- Parser and analysis core: `crates/core/`
- CLI entrypoint: `crates/cli/`
- Fixture-driven tests: `crates/core/tests/` and `fixtures/`
- Report schema stability matters for CI gate consumers

## CI integration

Add patchwaste as a budget gate in your GitHub Actions workflow:

```yaml
- name: Install patchwaste
  run: cargo install --git https://github.com/patchwaste/patchwaste patchwaste

- name: Run patchwaste budget gate
  run: |
    patchwaste analyse \
      --input path/to/BuildOutput \
      --baseline baseline.json \
      --budget-ratio 1.25 \
      --out patchwaste-out
```

Exit code `0` means the patch is within budget. Exit code `2` means it exceeded the threshold. The step fails and the pipeline stops.

Store `patchwaste-out/report.json` from a known-good build as your `baseline.json`. Update it when you intentionally accept a new baseline.

## Open Core Model

This repository is the Apache-2.0 open core for local-first Unreal/Steam patch waste analysis.

**patchwaste-cloud** is the commercial, hosted tier that layers team-scale workflow on top of the open-core CLI. It provides managed baselines and history, org policies and approvals, dashboards, alerts, and CI/PR integrations. It does not require uploading build artifacts; metadata-only ingestion and configurable retention are the default.

## Exit codes

- 0: pass
- 2: budget failed
- 1: tool error (or strict mode missing required counters)

## Development

```bash
# One-time setup
./scripts/bootstrap-dev.sh

# Local checks (same commands as CI)
./scripts/verify.sh
```

See `CONTRIBUTING.md` for the full contribution workflow.

## Notes

- This repo includes a synthetic fixture log. Replace `fixtures/*` with your real SteamPipe preview BuildOutput.
- This repo also includes `fixtures/automation_dummy/BuildOutput` for CI/test automation.
- Metrics are labeled as *estimated* unless confidence is HIGH.

## License

Apache-2.0. See `LICENSE`.
Attribution notice file: `NOTICE`.
