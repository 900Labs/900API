#!/usr/bin/env bash
set -euo pipefail

echo "900API local release gate"

echo "[1/10] Rust formatting"
cargo fmt --all -- --check

echo "[2/10] Rust lint"
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "[3/10] Clean frontend install"
npm ci

echo "[4/10] Frontend tests"
npm test

echo "[5/10] Svelte and TypeScript checks"
npm run check

echo "[6/10] Frontend production build"
npm run build

echo "[7/10] Rust workspace tests"
cargo test --workspace --locked

echo "[8/10] Documentation links"
npm run check:docs

echo "[9/10] Privacy gate self-test"
./scripts/test-privacy-gate.sh

echo "[10/10] Public release privacy gate"
./scripts/verify-public-release.sh

echo "900API local release gate passed"
