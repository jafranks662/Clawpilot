#!/usr/bin/env bash
set -euo pipefail

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup not found; install rustup before running preflight checks."
  exit 1
fi

if ! rustup component list --installed | grep -q '^rustfmt$'; then
  echo "Missing rustfmt. Install with: rustup component add rustfmt"
fi

if ! rustup component list --installed | grep -q '^clippy$'; then
  echo "Missing clippy. Install with: rustup component add clippy"
fi

echo "Running checks..."
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
