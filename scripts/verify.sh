#!/usr/bin/env bash
set -euo pipefail

echo "[verify] cargo fmt"
cargo fmt --all -- --check

echo "[verify] cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "[verify] cargo test"
cargo test --workspace --all-targets
