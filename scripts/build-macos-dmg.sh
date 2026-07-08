#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS DMG builds require macOS." >&2
  exit 1
fi

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

APP_NAME="900API"
APP_PATH="$ROOT_DIR/target/release/bundle/macos/${APP_NAME}.app"
DMG_DIR="$ROOT_DIR/target/release/bundle/dmg"
STAGING_DIR="$DMG_DIR/${APP_NAME}.dmg-staging"

if [[ ! -d "$APP_PATH" ]]; then
  echo "Missing app bundle: $APP_PATH" >&2
  echo "Run npm run tauri:build:app before building the DMG." >&2
  exit 1
fi

VERSION="$(node -e "const fs = require('fs'); const cfg = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8')); process.stdout.write(cfg.version);")"
case "$(uname -m)" in
  arm64) ARCH="aarch64" ;;
  x86_64) ARCH="x64" ;;
  *) ARCH="$(uname -m)" ;;
esac

DMG_PATH="$DMG_DIR/${APP_NAME}_${VERSION}_${ARCH}.dmg"

rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR" "$DMG_DIR"
trap 'rm -rf "$STAGING_DIR"' EXIT

ditto "$APP_PATH" "$STAGING_DIR/${APP_NAME}.app"
ln -s /Applications "$STAGING_DIR/Applications"

rm -f "$DMG_PATH"
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDZO \
  "$DMG_PATH"

echo "Built DMG at: $DMG_PATH"
