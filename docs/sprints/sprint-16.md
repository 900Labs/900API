# Sprint 16: Internationalization

## Scope
- Created i18n module (`src/lib/i18n.ts`) with:
  - 6 supported locales: English, Spanish, French, German, Japanese, Chinese
  - ~60 translation keys covering nav items, common actions, request fields, mock server, sync, tests, docs, settings
  - `locale` writable store with localStorage persistence
  - `t` derived store for translation function
  - `locales` array with flags and labels for UI
  - Fallback to English for missing keys
- Integrated i18n into Sidebar:
  - Nav item labels use `$t(labelKey)` instead of hardcoded strings
  - Added language switcher button at bottom with Globe icon
  - Dropdown menu with all 6 locales and flags
  - Selected locale highlighted with accent color
- Integrated i18n into Settings page:
  - Added Language section with 6 locale buttons in a grid
  - Section titles use translated strings
  - Version label translated
- Locale preference persisted to localStorage (`900api-locale`)

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 62 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Used Svelte stores (`writable` + `derived`) for reactive i18n
- Locale persisted to localStorage for cross-session persistence
- English as fallback for missing translation keys
- Translation keys use dot notation (e.g., `nav.requests`, `common.send`)
- Flags used as visual indicators in locale selector
- All 6 locales have complete translations for all keys

## Known Issues
- Not all UI components use i18n (only Sidebar and Settings fully integrated)
- No RTL (right-to-left) support
- No date/number formatting per locale
- No pluralization rules
- No dynamic language loading (all translations bundled)
- RequestBuilder, MockServer, GitSync, TestRunner, ApiDocs still use hardcoded English

## Next Sprint
- Sprint 17: Plugin System
