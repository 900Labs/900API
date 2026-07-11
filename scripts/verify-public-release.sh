#!/usr/bin/env bash
set -euo pipefail

echo "900API public release privacy gate"

release_files() {
  git ls-files -z --cached --others --exclude-standard | while IFS= read -r -d '' file; do
    if [[ -f "$file" ]]; then
      printf '%s\0' "$file"
    fi
  done
}

release_text_files() {
  release_files | while IFS= read -r -d '' file; do
    if [[ ! -s "$file" ]] || LC_ALL=C grep -Iq . "$file"; then
      printf '%s\0' "$file"
    fi
  done
}

scan_pattern() {
  local label="$1"
  local pattern="$2"
  local matches
  matches=$(release_text_files | xargs -0 rg -n --no-heading --color never --no-messages -e "$pattern" -- 2>/dev/null || true)
  if [[ -n "$matches" ]]; then
    echo "FAIL: $label"
    echo "$matches"
    exit 1
  fi
}

scan_pattern "private key material or known credential formats found" \
  '-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9_-]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}'

scan_pattern "personal local filesystem path found" \
  '/Users/[A-Za-z0-9._-]+/|/home/[A-Za-z0-9._-]+/|[A-Za-z]:\\Users\\[A-Za-z0-9._-]+\\'

scan_pattern "known personal username or machine detail found" \
  'samir''usani|samr''usani|Samir'' Usani'

EMAILS=$(release_text_files | xargs -0 rg -n --no-heading --color never \
  --no-messages -e '\b[A-Za-z][A-Za-z0-9._%+-]*@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' -- 2>/dev/null \
  | rg -v 'security@900labs\.com|@example\.(com|org|net)|@test\.com' || true)
if [[ -n "$EMAILS" ]]; then
  echo "FAIL: unintended email address found"
  echo "$EMAILS"
  exit 1
fi

ASSIGNED_SECRETS=$(release_text_files | xargs -0 rg -n --no-heading --color never \
  --no-messages -e '(?i)(api[_-]?key|access[_-]?token|client[_-]?secret|password)\s*[:=]\s*[\x22\x27][^\x22\x27]{12,}[\x22\x27]' -- 2>/dev/null || true)
if [[ -n "$ASSIGNED_SECRETS" ]]; then
  echo "FAIL: possible hardcoded credential found"
  echo "$ASSIGNED_SECRETS"
  exit 1
fi

TELEMETRY=$(release_text_files | xargs -0 rg -n --no-heading --color never \
  --no-messages -e '(?i)post''hog|amp''litude|mix''panel|seg''ment\.com|sentry_''sdk|sentry-''browser' -- 2>/dev/null || true)
if [[ -n "$TELEMETRY" ]]; then
  echo "FAIL: telemetry dependency or endpoint found"
  echo "$TELEMETRY"
  exit 1
fi

EM_DASH=$'\u2014'
PUBLIC_EM_DASHES=$(release_text_files | xargs -0 rg -n --no-heading --color never --no-messages -e "$EM_DASH" -- 2>/dev/null || true)
if [[ -n "$PUBLIC_EM_DASHES" ]]; then
  echo "FAIL: em dash found in public documentation or GitHub template"
  echo "$PUBLIC_EM_DASHES"
  exit 1
fi

echo "900API public release privacy gate passed"
