#!/usr/bin/env bash
set -euo pipefail

echo "=== 900API Public Release Privacy Gate ==="
echo ""

echo "[1/3] Checking for hardcoded secrets..."
SECRETS=$(grep -rn --include="*.rs" --include="*.ts" --include="*.svelte" --include="*.json" \
  -E "(api_key|apikey|secret|password|token)\s*[:=]\s*['\"][^'\"]{8,}['\"]" \
  src/ src-tauri/src/ crates/ 2>/dev/null \
  | grep -v "test" \
  | grep -v "#\[cfg(test)\]" \
  | grep -v "mod tests" \
  | grep -v "TestResult" \
  | grep -v "test_" \
  || true)
if [ -n "$SECRETS" ]; then
  echo "FAIL: Potential hardcoded secrets found:"
  echo "$SECRETS"
  exit 1
fi
echo "PASS: no hardcoded secrets"

echo ""
echo "[2/3] Checking for telemetry/analytics code..."
TELEMETRY=$(grep -rn --include="*.rs" --include="*.ts" --include="*.svelte" \
  -iE "(telemetry|analytics|tracking|posthog|amplitude|mixpanel|segment)" \
  src/ src-tauri/src/ crates/ 2>/dev/null || true)
if [ -n "$TELEMETRY" ]; then
  echo "FAIL: Telemetry/analytics code found:"
  echo "$TELEMETRY"
  exit 1
fi
echo "PASS: no telemetry code"

echo ""
echo "[3/3] Checking for hardcoded local paths..."
PATHS=$(grep -rn --include="*.rs" --include="*.ts" --include="*.svelte" \
  -E "/Users/|/home/|C:\\\\" \
  src/ src-tauri/src/ crates/ 2>/dev/null || true)
if [ -n "$PATHS" ]; then
  echo "FAIL: Hardcoded local paths found:"
  echo "$PATHS"
  exit 1
fi
echo "PASS: no hardcoded local paths"

echo ""
echo "=== All privacy gate checks passed ==="
