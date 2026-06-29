#!/usr/bin/env bash
set -euo pipefail

echo "=== 900API Local Quality Gate ==="
echo ""

echo "[1/5] Checking Rust compilation..."
cargo check 2>&1
if [ $? -ne 0 ]; then
  echo "FAIL: cargo check failed"
  exit 1
fi
echo "PASS: cargo check"

echo ""
echo "[2/5] Checking Rust warnings..."
WARNINGS=$(cargo check 2>&1 | grep -c "^warning:" || true)
if [ "$WARNINGS" -gt 0 ]; then
  echo "FAIL: $WARNINGS Rust warnings found"
  cargo check 2>&1 | grep "^warning:"
  exit 1
fi
echo "PASS: no Rust warnings"

echo ""
echo "[3/5] Checking clippy..."
CLIPPY=$(cargo clippy 2>&1 | grep -c "^warning:" || true)
if [ "$CLIPPY" -gt 0 ]; then
  echo "FAIL: $CLIPPY clippy warnings found"
  cargo clippy 2>&1 | grep "^warning:"
  exit 1
fi
echo "PASS: no clippy warnings"

echo ""
echo "[4/5] Checking Svelte/TypeScript..."
npm run check 2>&1
if [ $? -ne 0 ]; then
  echo "FAIL: svelte-check failed"
  exit 1
fi
echo "PASS: svelte-check"

echo ""
echo "[5/5] Checking Rust tests..."
cargo test 2>&1
if [ $? -ne 0 ]; then
  echo "FAIL: cargo test failed"
  exit 1
fi
echo "PASS: cargo test"

echo ""
echo "=== All quality gate checks passed ==="
