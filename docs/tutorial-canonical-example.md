# Canonical Tutorial: Baseline + Budget Gate

This tutorial runs a full local loop with fixture data and expected outcomes.

## ELI5 (for absolute beginners)

You run one command to measure patch waste, then one command to compare against a baseline.
If the result is too wasteful, CI can fail early before release.

## For educated non-technical readers (Guardian / FT / Economist style)

The tutorial demonstrates a simple control loop:
1. Establish a baseline from a known build.
2. Compare a new build against that baseline with a budget ratio.
3. Convert that comparison into a binary decision signal (exit code).

This is what makes the tool useful in governance: consistent policy checks with minimal operational overhead.

## For the engineering community

### Prerequisites

- Rust toolchain installed
- Repository checked out

### Step 1: Analyze build output and create baseline

```bash
cargo run -p patchwaste -- analyze \
  --input fixtures/automation_dummy/BuildOutput \
  --out /tmp/patchwaste-open-core-demo
```

Inspect the human-readable report:

```bash
cat /tmp/patchwaste-open-core-demo/report.md
```

The run writes:
- `/tmp/patchwaste-open-core-demo/report.json`
- `/tmp/patchwaste-open-core-demo/report.md`

With the bundled fixture, the CLI prints:
- `new_bytes=4194304`
- `changed_content_bytes=1048576`
- `waste_ratio=0.750`

### Step 2: Re-run with baseline (expected pass)

```bash
cargo run -p patchwaste -- analyze \
  --input fixtures/automation_dummy/BuildOutput \
  --baseline /tmp/patchwaste-open-core-demo/report.json \
  --budget-ratio 1.25 \
  --out /tmp/patchwaste-open-core-demo-compare

echo $?
```

Expected: exit code `0` (budget passes).

### Step 3: Simulate budget failure (expected fail)

```bash
printf '{"metrics":{"new_bytes":1000}}\n' > /tmp/patchwaste-small-baseline.json

cargo run -p patchwaste -- analyze \
  --input fixtures/automation_dummy/BuildOutput \
  --baseline /tmp/patchwaste-small-baseline.json \
  --budget-ratio 1.25 \
  --out /tmp/patchwaste-open-core-demo-fail

echo $?
```

Expected: exit code `2` (budget failed).

## Hacker/Geek deep dive

### What the output implies

- `waste_ratio=0.750` means most bytes in the patch are not net new content.
- High waste can indicate pack churn, reordering, or a large dominant offender.
- The baseline gate (`--budget-ratio`) compares current `new_bytes` to prior run and sets CI-friendly exit codes.

### How to inspect artifacts directly

```bash
cat /tmp/patchwaste-open-core-demo/report.json
cat /tmp/patchwaste-open-core-demo-compare/report.json
cat /tmp/patchwaste-open-core-demo-fail/report.json
```

### Troubleshooting

- Exit `1`: tool/runtime error, inspect stderr
- Empty or unexpected metrics: verify input points to a SteamPipe preview BuildOutput
- CI mismatch: confirm same fixture/input and budget ratio
