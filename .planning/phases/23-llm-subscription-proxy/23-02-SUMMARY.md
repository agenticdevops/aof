---
phase: 23-llm-subscription-proxy
plan: 02
subsystem: auth
tags: [oauth, pkce, openai, gemini, anthropic, token-refresh, reqwest]

requires:
  - phase: 23-llm-subscription-proxy
    plan: 01
    provides: "AuthProfilesStore, TokenSet, PkceState, generate_pkce_state, url_encode, parse_query_params, profile_id, select_profile_id"

provides:
  - "openai_oauth module: PKCE authorize URL, loopback receiver on :1455, token exchange, refresh, device code flow"
  - "gemini_oauth module: PKCE authorize URL, loopback receiver on :1456, token exchange, refresh, device code flow"
  - "anthropic_token module: AnthropicAuthKind enum, detect_auth_kind with JWT-shape detection"
  - "AuthService: store/get/refresh tokens for all three providers with per-profile async locking"
  - "normalize_provider: maps openai/codex → openai, anthropic/claude → anthropic, gemini/google → gemini"
  - "Token refresh: 3 retries at 350ms × attempt delay, 10s failure backoff, per-profile async mutex"

affects:
  - 23-03
  - 23-04
  - 23-05
  - 23-06
  - 23-07

tech-stack:
  added:
    - "reqwest (workspace) added to agentix-core dependencies — needed for OAuth token exchange HTTP calls"
  patterns:
    - "auth/ module directory: mod.rs (foundation) + openai_oauth.rs + gemini_oauth.rs + anthropic_token.rs"
    - "auth_service.rs: standalone coordinator, not embedded in auth module"
    - "Per-provider loopback ports: OpenAI=1455, Gemini=1456 (copied from zeroclaw)"
    - "Gemini credentials from env vars: GEMINI_OAUTH_CLIENT_ID, GEMINI_OAUTH_CLIENT_SECRET"
    - "JWT-shape detection: 2+ dots = Authorization mode for Anthropic"
    - "with_env_vars() test helper serializes env var mutations via global mutex"

key-files:
  created:
    - "crates/agentix-core/src/auth/openai_oauth.rs — OpenAI OAuth2 flow (405 lines)"
    - "crates/agentix-core/src/auth/gemini_oauth.rs — Google/Gemini OAuth2 flow (553 lines)"
    - "crates/agentix-core/src/auth/anthropic_token.rs — Anthropic auth kind detection (78 lines)"
    - "crates/agentix-core/src/auth_service.rs — AuthService coordinator (567 lines)"
    - "crates/agentix-core/tests/auth_oauth_test.rs — 18 OAuth integration tests (315 lines)"
  modified:
    - "crates/agentix-core/src/auth/mod.rs — renamed from auth.rs, added submodule declarations"
    - "crates/agentix-core/src/lib.rs — added auth_service module, re-exported AuthService, AnthropicAuthKind, detect_auth_kind, normalize_provider"
    - "crates/agentix-core/Cargo.toml — added reqwest workspace dependency"

key-decisions:
  - "Converted auth.rs to auth/ module directory to keep provider files under 500 lines (CLAUDE.md modular design requirement)"
  - "normalize_provider uses openai (not openai-codex) as canonical name — cleaner UX for OpenAgentiX vs zeroclaw's internal naming"
  - "auth_service.rs is a standalone module (not a submodule of auth/) to keep auth/ focused on low-level flows"
  - "with_env_vars() test helper acquires a global mutex to prevent race conditions between Gemini env var tests running in parallel"

patterns-established:
  - "Provider loopback port convention: OpenAI=1455, Gemini=1456 — matches browser OAuth redirect URIs"
  - "Token refresh locking: static OnceLock<Mutex<HashMap>> for per-profile async locks prevents duplicate refreshes"
  - "Refresh backoff: REFRESH_BACKOFFS static tracks deadline instants; returns remaining seconds if in cooldown"
  - "Preserve refresh_token on refresh: if new TokenSet lacks refresh_token, copy from previous"

requirements-completed: [SUB-01, SUB-03]

duration: 8min
completed: 2026-03-22
---

# Phase 23 Plan 02: Per-Provider OAuth Flows and AuthService Summary

**OpenAI/Gemini PKCE OAuth2 flows with loopback servers, Anthropic JWT detection, and AuthService coordinator with 3-retry/350ms-backoff token refresh**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-22T05:11:30Z
- **Completed:** 2026-03-22T05:20:05Z
- **Tasks:** 2
- **Files modified:** 8 (3 new provider modules + auth_service + tests + 3 modified)

## Accomplishments

- Ported all three provider OAuth flows from zeroclaw with OpenAgentiX branding (success pages say "OpenAgentiX login complete")
- OpenAI OAuth: PKCE authorize URL with auth.openai.com, loopback on :1455, code exchange, refresh, device code flow
- Gemini OAuth: PKCE authorize URL with accounts.google.com, loopback on :1456 with stdin fallback, code exchange, refresh, device code
- Anthropic: AnthropicAuthKind enum with JWT-shape detection (2+ dots = Authorization, sk-ant-api prefix = ApiKey)
- AuthService: full coordinator with token refresh (3 retries, 350ms×attempt backoff, 10s failure cooldown, per-profile async lock)
- 18 new integration tests + 270 total agentix-core tests passing

## Task Commits

1. **Task 1: Port per-provider OAuth flows (OpenAI, Gemini, Anthropic)** — `ad4bab1` (feat)
2. **Task 2: AuthService coordinator with token refresh and profile management** — `d17c69c` (feat)
3. **Fix: Serialize Gemini env var tests with global mutex** — `1f38c73` (fix)

## Files Created/Modified

- `crates/agentix-core/src/auth/mod.rs` — Renamed from auth.rs, added submodule declarations (openai_oauth, gemini_oauth, anthropic_token)
- `crates/agentix-core/src/auth/openai_oauth.rs` — OpenAI OAuth2 PKCE flow with auth.openai.com endpoints
- `crates/agentix-core/src/auth/gemini_oauth.rs` — Google/Gemini OAuth2 PKCE flow with accounts.google.com endpoints
- `crates/agentix-core/src/auth/anthropic_token.rs` — AnthropicAuthKind enum + detect_auth_kind function
- `crates/agentix-core/src/auth_service.rs` — AuthService: all three provider token management with retry/backoff/locking
- `crates/agentix-core/tests/auth_oauth_test.rs` — 18 integration tests for OAuth flows and auth service
- `crates/agentix-core/src/lib.rs` — Added auth_service module + new re-exports
- `crates/agentix-core/Cargo.toml` — Added reqwest workspace dependency

## Decisions Made

- **auth/ module directory instead of adding to auth.rs:** auth.rs was already 38KB (1084 lines). Adding three provider files inline would violate CLAUDE.md's 500-line file guideline. Converting to a module directory keeps each file focused.
- **openai as canonical (not openai-codex):** Zeroclaw uses "openai-codex" internally because it's a Codex-specific tool. OpenAgentiX is a general framework; "openai" is the cleaner provider name. "codex" and "openai-codex" remain as aliases.
- **auth_service.rs outside auth/ directory:** AuthService is a higher-level coordinator that imports from the auth module. Keeping it at the crate root level (alongside other service files) makes the dependency direction clearer.
- **with_env_vars() mutex-based test helper:** Parallel test execution raced on GEMINI_OAUTH_CLIENT_ID when one test set it and another removed it simultaneously. A global Mutex<()> ensures env var mutation tests run serially.
- **reqwest added to agentix-core:** Required for OAuth token exchange HTTP calls. Already in workspace dependencies, just not listed in agentix-core's Cargo.toml.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added reqwest dependency to agentix-core**
- **Found during:** Task 1 (OpenAI OAuth flow creation)
- **Issue:** auth/openai_oauth.rs and auth/gemini_oauth.rs use `reqwest::Client` for HTTP calls, but agentix-core had no reqwest dependency
- **Fix:** Added `reqwest = { workspace = true }` to agentix-core Cargo.toml
- **Files modified:** `crates/agentix-core/Cargo.toml`
- **Verification:** cargo check passes cleanly
- **Committed in:** ad4bab1 (Task 1 commit)

**2. [Rule 1 - Bug] Test race condition in Gemini env var tests**
- **Found during:** Task 2 verification (cargo test)
- **Issue:** `gemini_authorize_url_contains_pkce` and `gemini_authorize_url_fails_without_env_vars` run in parallel and race on GEMINI_OAUTH_CLIENT_ID env var — first test sets it, second removes it, causing intermittent failures
- **Fix:** Replaced per-test EnvGuard approach with `with_env_vars()` helper that holds a global `Mutex<()>` for the duration of each test body, serializing all env var mutations
- **Files modified:** `crates/agentix-core/tests/auth_oauth_test.rs`
- **Verification:** All 18 tests pass consistently
- **Committed in:** 1f38c73 (fix commit)

---

**Total deviations:** 2 auto-fixed (1 blocking dependency, 1 test race bug)
**Impact on plan:** Both fixes necessary for compilation and reliable test execution. No scope creep.

## Issues Encountered

- Pre-existing intermittent test `approval::tests::test_approval_request_is_expired_with_zero_timeout` occasionally fails under timing pressure (reported in plan 01 summary). Not caused by this plan's changes.

## Next Phase Readiness

- AuthService fully functional for all three providers — ready for plan 03 (gateway OAuth endpoints)
- `normalize_provider` handles all provider aliases — ready for use in CLI auth commands (plan 06)
- `openai_oauth::build_authorize_url` and `gemini_oauth::build_authorize_url` ready for plan 03 API endpoints
- `anthropic_token::detect_auth_kind` ready for LLM adapter use in plan 04/05

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
