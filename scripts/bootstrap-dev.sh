#!/usr/bin/env bash
set -euo pipefail

if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup is required. Install from https://rustup.rs/" >&2
  exit 1
fi

echo "[bootstrap] sync Rust toolchain from rust-toolchain.toml"
rustup show active-toolchain >/dev/null
rustup component add rustfmt clippy >/dev/null

echo "[bootstrap] configure git hooks"
git config core.hooksPath .githooks

if command -v pre-commit >/dev/null 2>&1; then
  echo "[bootstrap] installing pre-commit hooks"
  pre-commit install >/dev/null
fi

echo "[bootstrap] done"
echo "Next: ./scripts/verify.sh"
