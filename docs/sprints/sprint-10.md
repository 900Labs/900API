# Sprint 10: Advanced Authentication

## Scope
- Added `sha2`, `hmac`, and `base64` dependencies for cryptographic signing
- Created auth module (`src-tauri/src/auth/mod.rs`) with centralized `apply_auth` function:
  - **OAuth 2.0**: Sends access token as `Authorization: <token_type> <token>` header
  - **OAuth 1.0a**: Full HMAC-SHA256 signing with OAuth params (consumer key, token, nonce, timestamp, signature)
  - **AWS Sig v4**: Complete AWS Signature v4 signing (canonical request, string to sign, signing key derivation, Authorization header)
  - **Hawk**: Hawk auth header with HMAC-SHA256, timestamp, nonce, and mac
  - Percent-encoding utility for OAuth parameter encoding
  - 5 unit tests covering percent-encoding, OAuth2, AWS Sig v4, and Hawk
- Extended `AuthType` enum with `OAuth2`, `OAuth1`, `AwsSigV4`, `Hawk` variants
- Extended `AuthConfig` struct with fields for all new auth types
- Refactored HTTP engine to use centralized `auth::apply_auth` for both REST and GraphQL
- Updated `RequestBuilder.svelte` with:
  - New auth type buttons (OAuth 2.0, OAuth 1.0a, AWS Sig v4, Hawk)
  - OAuth 2.0: access token, token type selector, refresh token
  - OAuth 1.0a: consumer key/secret, token/secret
  - AWS Sig v4: access key ID, secret access key, region, service
  - Hawk: ID, key, algorithm selector (SHA-256/SHA-1)
  - All new fields wired into request config, save/load, and auth config JSON
- Updated API docs with authentication section

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 41 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Centralized all auth logic in `auth::apply_auth` — single function handles all auth types
- OAuth 1.0a uses HMAC-SHA256 (not HMAC-SHA1 which is less secure)
- AWS Sig v4 implements full signing with host, x-amz-date, x-amz-content-sha256 headers
- Hawk uses HMAC-SHA256 by default, SHA-1 available as option
- OAuth 2.0 is token-based (no token flow/OAuth dance) — user provides the access token
- Auth fields stored in `AuthConfig` struct with serde defaults for backward compatibility

## Known Issues
- No OAuth 2.0 token flow (authorization code, client credentials, password grant)
- No OAuth 2.0 token refresh automation
- No NTLM support (requires platform-specific TLS integration)
- AWS Sig v4 doesn't sign with all request headers (only host, x-amz-date, x-amz-content-sha256)
- OAuth 1.0a doesn't support RSA-SHA256 or PLAINTEXT signature methods
- Hawk doesn't support payload validation (hash attribute)

## Next Sprint
- Sprint 11: Mock Server
