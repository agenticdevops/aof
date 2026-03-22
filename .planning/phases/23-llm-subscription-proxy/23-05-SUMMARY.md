---
phase: 23-llm-subscription-proxy
plan: 05
subsystem: auth
tags: [oauth, cli, pkce, openai, gemini, anthropic, clap, dirs, device-code]

requires:
  - phase: 23-llm-subscription-proxy
    plan: 02
    provides: "AuthService, normalize_provider, openai_oauth, gemini_oauth, anthropic_token, generate_pkce_state"

provides:
  - "agentix auth start <provider> — browser OAuth flow with PKCE or device-code for headless"
  - "agentix auth status — table of all three providers with connection/mode/account/expiry"
  - "agentix auth disconnect <provider> — remove stored credentials from ~/.agentix/"
  - "commands/auth.rs with default_state_dir() and make_auth_service() helpers"
  - "Auth subcommand registered in cli.rs AuthCommands enum and main.rs dispatcher"
  - "docs/reference/cli-auth.md — full CLI auth reference with examples"

affects:
  - 23-06
  - 23-07

tech-stack:
  added:
    - "dirs = 5.0 in agentix crate — home dir resolution for ~/.agentix state path"
  patterns:
    - "default_state_dir() returns ~/.agentix for auth credential storage"
    - "make_auth_service() factory creates AuthService with encrypt_secrets=true"
    - "open_browser() uses platform-appropriate command (open/xdg-open/cmd)"
    - "Status table: Provider/Status/Mode/Account/Expires columns with colored output"
    - "find_active_profile() follows active_profiles > {provider}:default > first-for-provider chain"

key-files:
  created:
    - "crates/agentix/src/commands/auth.rs — CLI auth command (start/status/disconnect) (310 lines)"
    - "crates/agentix/tests/auth_cli_test.rs — 8 tests for auth logic via AuthService (159 lines)"
    - "docs/reference/cli-auth.md — full CLI auth reference documentation"
  modified:
    - "crates/agentix/src/commands/mod.rs — added pub mod auth"
    - "crates/agentix/src/cli.rs — added Auth command + AuthCommands enum"
    - "crates/agentix/src/main.rs — dispatched Auth/Start/Status/Disconnect commands"
    - "crates/agentix/Cargo.toml — added dirs = 5.0 dependency"

key-decisions:
  - "Browser open via platform subprocess (open/xdg-open) not a crate — keeps deps minimal"
  - "find_active_profile() mirrors AuthService selection priority (active > default > first) for status display"
  - "Anthropic start reads from stdin (no browser) — token paste matches plan spec"
  - "Status table shows 'Never' expiry for token-based credentials (no TokenSet expiry)"

patterns-established:
  - "CLI auth commands use make_auth_service() — always encrypt_secrets=true in production"
  - "Provider aliases resolved early via normalize_provider() before any AuthService calls"

requirements-completed: [SUB-01]

duration: 11min
completed: 2026-03-22
---

# Phase 23 Plan 05: CLI Auth Commands Summary

**`agentix auth` subcommand with PKCE browser OAuth for OpenAI/Gemini, token-paste for Anthropic, device-code fallback, and status/disconnect management**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-22T05:46:10Z
- **Completed:** 2026-03-22T05:57:42Z
- **Tasks:** 2
- **Files modified:** 7 (3 created + 4 modified)

## Accomplishments

- `agentix auth start <provider>` opens browser for OpenAI/Gemini OAuth PKCE flow, prompts token paste for Anthropic — all three providers fully wired to AuthService
- `--device-code` flag routes to `start_device_code_flow()` + `poll_device_code_tokens()` for headless environments on OpenAI and Gemini
- `agentix auth status` renders a colored table (Provider/Status/Mode/Account/Expires) for all three providers; shows "No providers authenticated" when empty
- `agentix auth disconnect <provider>` prints provider-specific "Disconnected" or "No credentials found" messages
- 8 tests verifying AuthService contract: normalization aliases, empty status, store+retrieve, disconnect returns bool, multi-provider isolation

## Task Commits

1. **Task 1: Implement agentix auth CLI command** — `3422e61` (feat)
2. **Task 2: CLI auth tests** — `a8ea714` (test)

## Files Created/Modified

- `crates/agentix/src/commands/auth.rs` — start/status/disconnect handlers with browser open, device code, Anthropic stdin, status table
- `crates/agentix/tests/auth_cli_test.rs` — 8 integration tests (no live OAuth server needed)
- `docs/reference/cli-auth.md` — full CLI auth reference with provider behavior, device code examples, token storage notes
- `crates/agentix/src/commands/mod.rs` — added `pub mod auth`
- `crates/agentix/src/cli.rs` — added `Auth { command: AuthCommands }` variant and `AuthCommands` enum
- `crates/agentix/src/main.rs` — dispatched all three Auth subcommands
- `crates/agentix/Cargo.toml` — added `dirs = "5.0"` for home dir resolution

## Decisions Made

- **Browser open via subprocess:** Used `std::process::Command::new("open")` / `xdg-open` / `cmd /c start` rather than an `open` crate. Keeps deps minimal; all three provider OAuth flows already work via agentix-core.
- **find_active_profile() in status handler:** Duplicates the priority chain (active_profiles > default profile > first profile for provider) rather than calling AuthService.get_profile() per provider. Avoids repeated async load of the profile store and allows single-pass table rendering.
- **Anthropic start reads from stdin:** Token paste aligns with plan spec and user flow from `claude setup-token`. No browser OAuth for Anthropic in CLI path (Command Center handles that via gateway endpoints in plan 04/06).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added dirs = "5.0" to agentix Cargo.toml**
- **Found during:** Task 1 (commands/auth.rs creation)
- **Issue:** `dirs::home_dir()` used in `default_state_dir()` but `dirs` not in agentix dependencies
- **Fix:** Added `dirs = "5.0"` to `[dependencies]` in `crates/agentix/Cargo.toml`
- **Files modified:** `crates/agentix/Cargo.toml`
- **Verification:** `cargo build -p agentix` exits 0
- **Committed in:** `3422e61` (Task 1 commit)

**2. [Rule 1 - Bug] Fixed BTreeMap vs HashMap type mismatch in find_active_profile**
- **Found during:** Task 1 first build attempt
- **Issue:** `AuthProfilesData` uses `BTreeMap<String, AuthProfile>` for `profiles` and `BTreeMap<String, String>` for `active_profiles`, but `find_active_profile()` was typed with `HashMap`
- **Fix:** Updated `find_active_profile()` parameter types to `&BTreeMap<...>`
- **Files modified:** `crates/agentix/src/commands/auth.rs`
- **Verification:** `cargo build -p agentix` exits 0 with no type errors
- **Committed in:** `3422e61` (Task 1 commit)

**3. [Rule 1 - Bug] Fixed double reference &&AuthProfile in status rows**
- **Found during:** Task 1 first build (same compilation)
- **Issue:** `find_active_profile()` returns `Option<&AuthProfile>`, passing to `StatusRow::from_profile()` via `.as_ref()` yielded `Option<&&AuthProfile>` — type mismatch
- **Fix:** Removed `.as_ref()` call — `find_active_profile` already returns a reference
- **Files modified:** `crates/agentix/src/commands/auth.rs`
- **Verification:** Compiler accepted after fix; `cargo build` exits 0
- **Committed in:** `3422e61` (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 blocking dependency, 2 type bugs)
**Impact on plan:** All three fixes required for compilation. No scope creep.

## Issues Encountered

- Pre-existing broken tests in `cli_tests.rs` reference commands that no longer exist (`get`, `delete`) — these 12 failures are pre-existing and unrelated to this plan's changes. Auth-specific tests all pass.

## User Setup Required

None — `agentix auth start` handles provider-specific setup instructions inline (Gemini env var check, Anthropic token paste instructions).

## Next Phase Readiness

- `agentix auth start|status|disconnect` fully functional — ready for plan 06 (Command Center auth UI) to complement CLI path
- `make_auth_service()` available for any additional CLI commands that need credential access
- `default_state_dir()` establishes `~/.agentix` as canonical CLI state location

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
