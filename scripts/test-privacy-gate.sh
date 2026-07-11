#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

test_dir=".privacy-gate-check-$$"
cleanup() {
  rm -rf "$test_dir"
}
trap cleanup EXIT

./scripts/verify-public-release.sh >/dev/null

mkdir "$test_dir"
credential_name='api_''key'
credential_value='release-fixture-value-1234'
test_files=(
  '.env'
  'fixture.pem'
  'fixture.key'
  'fixture.txt'
  'fixture'
  '.github/fixture.yml'
)

for test_file in "${test_files[@]}"; do
  mkdir -p "$(dirname "$test_dir/$test_file")"
  printf '%s="%s"\n' "$credential_name" "$credential_value" > "$test_dir/$test_file"
  if output=$(./scripts/verify-public-release.sh 2>&1); then
    echo "FAIL: privacy gate missed $test_file"
    exit 1
  fi
  if [[ "$output" != *"possible hardcoded credential found"* ]]; then
    echo "FAIL: privacy gate failed for an unexpected reason while scanning $test_file"
    echo "$output"
    exit 1
  fi
  rm "$test_dir/$test_file"
done

binary_file="$test_dir/fixture.bin"
printf '\0%s="%s"\n' "$credential_name" "$credential_value" > "$binary_file"
./scripts/verify-public-release.sh >/dev/null
rm "$binary_file"

echo "900API privacy gate self-test passed"
