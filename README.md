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

https://github.com/user-attachments/assets/60419e3a-cf40-428e-90c4-82de7c84de56

## Try it on your build

Five second sanity check before you wire patchwaste into CI.

```
cargo install --git https://github.com/pw-labs/patchwaste patchwaste
patchwaste validate --input path/to/BuildOutput
```

No data leaves your machine. patchwaste prints what it recognised, what it could not parse, and which counters it found. If the output looks right, the metrics from `analyse` are honest enough to gate a budget. If it does not, paste the validate output into [a new issue](https://github.com/pw-labs/patchwaste/issues/new) and the parser will be extended to fit your log format.

This is the lowest commitment way to test whether patchwaste fits your pipeline.

## Why this exists

In 2024, Enshrouded shipped a 299KB content change that triggered a 30.6GB Steam download. Studios have complained about opaque SteamPipe patch sizes for 8+ years with zero tooling to catch it before players do. `patchwaste` fills that gap as a focused CI gate that tells you whether your build wasted bandwidth before you push to production.

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
git clone https://github.com/pw-labs/patchwaste.git
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
  run: cargo install --git https://github.com/pw-labs/patchwaste patchwaste

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

## Further Reading

- FAQ: `docs/FAQ.md`
- Canonical tutorial: `docs/tutorial-canonical-example.md`
- Open-core scope: `docs/open-core-scope.md`
- Roadmap: `docs/roadmap.md`

## Current limitations

- **Synthetic fixtures only.** The included test data is synthetic. Real SteamPipe BuildOutput from a production title has not yet been tested against the parser. If you have real logs, we want to hear from you.
- **Unreal + Steam scope.** This tool is intentionally narrow. It targets UE5 pak-based builds distributed through SteamPipe. Other engines and storefronts are out of scope.
- **Log-level analysis.** `patchwaste` analyses SteamPipe preview log output, not raw binary diffs. Accuracy depends on what SteamPipe reports.
- **No external validation yet.** Zero studios have run this against production data. The metrics and thresholds are designed from first principles, not calibrated against real-world baselines.

If any of these limits affect your use case, open an issue or reach out. Fixing them is the roadmap.

## License

Apache-2.0. See `LICENSE`.
Attribution notice file: `NOTICE`.
