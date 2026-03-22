---
phase: 23-llm-subscription-proxy
plan: 01
subsystem: auth
tags: [oauth, aes-gcm, pkce, token-storage, encryption, provider-mode]

requires:
  - phase: 22-command-center
    provides: "Completed command center baseline — no direct code dependency"

provides:
  - "ProviderMode enum (Api | Subscription) in ProviderConfig"
  - "OAuthConfig struct for optional OAuth client credentials in ProviderConfig"
  - "AuthProfile and TokenSet types for OAuth2 credential storage"
  - "AuthProfilesStore: encrypted JSON profile store with AES-256-GCM"
  - "PkceState, generate_pkce_state, url_encode/decode, parse_query_params OAuth utilities"
  - "profile_id and select_profile_id with 4-level priority chain"

affects:
  - 23-02
  - 23-03
  - 23-04
  - 23-05
  - 23-06
  - 23-07

tech-stack:
  added:
    - "anyhow 1.0 (added to agentix-core dependencies)"
    - "tokio fs and time features (added to agentix-core)"
  patterns:
    - "enc2: prefix for AES-256-GCM encrypted values stored as hex strings"
    - "File-level locking for concurrent profile store access (50ms poll, 10s timeout)"
    - "Atomic writes via temp file + rename for profile store"
    - "Debug impl redacts token_set and token fields to prevent log leaks"
    - "Profile ID format: {provider}:{profile_name}"
    - "Profile selection priority: explicit override > active > default > first"

key-files:
  created:
    - "crates/agentix-core/src/auth.rs"
    - "crates/agentix-core/tests/auth_types_test.rs"
  modified:
    - "crates/agentix-core/src/config.rs"
    - "crates/agentix-core/src/lib.rs"
    - "crates/agentix-core/Cargo.toml"

key-decisions:
  - "Used AES-256-GCM (already in workspace) instead of ChaCha20-Poly1305 (zeroclaw) — avoids new dependency while maintaining equivalent security and enc2: prefix convention"
  - "AuthProfilesStore is standalone (not wrapping SecretStore struct) to support enc2: hex string pattern needed for JSON storage"
  - "Added anyhow to agentix-core deps — auth module uses Result<T> with context chaining"
  - "ProviderMode is Option<ProviderMode> on ProviderConfig — None treated as Api for backward compatibility"

patterns-established:
  - "enc2: prefix: AES-256-GCM nonce || ciphertext hex-encoded, stored as string in JSON"
  - "Secret key at <state_dir>/.secret_key — hex-encoded 32-byte random key, 0600 permissions on Unix"
  - "Token fields excluded from Debug output via manual impl with finish_non_exhaustive"
  - "Plaintext-to-enc2 migration: on load, plaintext values are re-encrypted and persisted"

requirements-completed: [SUB-01, SUB-02]

duration: 7min
completed: 2026-03-22
---

# Phase 23 Plan 01: Auth Core Types and Encrypted Profile Store Summary

**AES-256-GCM encrypted OAuth profile store with PKCE utilities — foundation types for LLM subscription proxy across all three providers**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-22T05:01:56Z
- **Completed:** 2026-03-22T05:09:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Extended `ProviderConfig` with `ProviderMode` (Api/Subscription) and `OAuthConfig` — backward compatible with all existing YAML configs
- Created `auth.rs` with full OAuth credential lifecycle: `AuthProfile`, `TokenSet`, `AuthProfilesStore` with AES-256-GCM encryption, file locking, and atomic writes
- Ported PKCE utilities from zeroclaw: `PkceState`, `generate_pkce_state`, `url_encode`, `url_decode`, `parse_query_params`
- 10 tests passing: profile ID format, token expiry, PKCE S256 math, URL roundtrip, profile selection priority, encrypted roundtrip (verifies enc2: in raw JSON), atomic write

## Task Commits

1. **Task 1: Extend ProviderConfig with ProviderMode and OAuthConfig** - `87e7557` (feat)
2. **Task 2: Port auth types, encrypted profile store, and OAuth utils** - `e021ed5` (feat)

## Files Created/Modified

- `crates/agentix-core/src/auth.rs` — AuthProfile, TokenSet, AuthProfilesStore, PkceState, OAuth utilities (475 lines)
- `crates/agentix-core/tests/auth_types_test.rs` — 10 integration tests for all auth types
- `crates/agentix-core/src/config.rs` — Added ProviderMode enum and OAuthConfig struct, extended ProviderConfig
- `crates/agentix-core/src/lib.rs` — Added `pub mod auth`, re-exported ProviderMode, OAuthConfig, and all auth types
- `crates/agentix-core/Cargo.toml` — Added anyhow dependency, tokio fs/time features, sha2/base64 dev-deps

## Decisions Made

- **AES-256-GCM instead of ChaCha20-Poly1305:** The zeroclaw reference uses ChaCha20-Poly1305 but that crate isn't in the workspace. AES-256-GCM (already present) provides equivalent authenticated encryption with the same `enc2:` prefix convention.
- **Standalone AuthProfilesStore:** Rather than wrapping the existing `SecretStore` struct (which uses `EncryptedSecret` structs), implemented a dedicated store that encrypts directly to hex strings. This matches zeroclaw's pattern of `enc2:<hex>` storage in JSON.
- **Option<ProviderMode>:** Using `Option<ProviderMode>` (not `ProviderMode`) on `ProviderConfig` preserves backward compatibility — `None` is treated as `Api` mode by callers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added anyhow dependency to agentix-core**
- **Found during:** Task 2 (auth.rs creation)
- **Issue:** auth.rs uses `anyhow::Result` and `Context` trait but agentix-core had no anyhow dependency
- **Fix:** Added `anyhow = { workspace = true }` to agentix-core Cargo.toml
- **Files modified:** `crates/agentix-core/Cargo.toml`
- **Committed in:** e021ed5 (Task 2 commit)

**2. [Rule 3 - Blocking] Added tokio fs/time features to agentix-core**
- **Found during:** Task 2 (auth.rs creation)
- **Issue:** auth.rs uses `tokio::fs` and `tokio::time::sleep` but only `sync` feature was enabled
- **Fix:** Added `fs` and `time` to tokio features in agentix-core Cargo.toml
- **Files modified:** `crates/agentix-core/Cargo.toml`
- **Committed in:** e021ed5 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes required for compilation. No scope creep.

## Issues Encountered

- Pre-existing flaky test `approval::tests::test_approval_request_is_expired_with_zero_timeout` fails intermittently under timing pressure but passes in isolation. Not caused by this plan's changes.

## Next Phase Readiness

- All auth foundation types ready for plan 02 (OAuth flows: Anthropic, OpenAI, Gemini)
- `AuthProfilesStore` ready to be used by `AuthService` in plan 02
- `select_profile_id` and profile selection priority tested and committed
- `ProviderMode::Subscription` available for plan 03 (gateway OAuth endpoints)

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
