---
phase: 23-llm-subscription-proxy
plan: 06
subsystem: ui
tags: [svelte, oauth, api-client, command-center, settings, subscription]

# Dependency graph
requires:
  - phase: 23-llm-subscription-proxy plan 04
    provides: Gateway OAuth endpoints (GET /api/v1/auth/:provider/start|status, DELETE /api/v1/auth/:provider, POST /api/v1/auth/:provider/token)
provides:
  - AuthStatus and AuthStartResponse TypeScript interfaces in api/types.ts
  - api.auth.start/status/disconnect/submitToken client methods
  - Settings page segmented provider mode toggle (API Key | Subscription)
  - OAuth popup flow for OpenAI and Gemini (2s polling)
  - Anthropic token paste flow (setup-token compatible)
  - Auth status display with connection indicator, account_id, and expiry
  - First-run wizard updated to present API Key and Subscription at equal level
affects: [23-07-command-center-docs, command-center-ui, subscription-proxy-e2e]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "OAuth popup flow using window.open + poll pattern (2s interval, 5min timeout)"
    - "Segmented control as pill with two buttons per provider card"
    - "Per-provider mode state in Record<string, 'api' | 'subscription'>"
    - "Auth status badge: green dot for connected, plain text for disconnected"

key-files:
  created: []
  modified:
    - apps/command-center/src/lib/api/types.ts
    - apps/command-center/src/lib/api/client.ts
    - apps/command-center/src/routes/settings/+page.svelte
    - apps/command-center/src/lib/components/first-run-wizard.svelte

key-decisions:
  - "OAuth popup uses window.open() with named target 'agentix-auth' so second authorize click reuses same window"
  - "Polling interval 2s with 5-minute timeout for popup OAuth completion"
  - "First-run wizard step 2 presents API Key and Subscription at equal level — no default mode forced"
  - "Anthropic subscription mode shows token paste (not popup) matching CLI `agentix auth start anthropic` UX"
  - "formatRelativeTime() calculates 'in Xh Ym' from ISO expiry string for human-readable display"

patterns-established:
  - "Segmented control per provider: pill with 'API Key' | 'Subscription' buttons, active button has solid background"
  - "Auth status shows green dot + 'Connected' with optional account_id + 'expires in Xh Ym' when authenticated"
  - "OAuth popup flow: open window, poll status, close window on success, update local state"
  - "Disconnect clears authStatuses entry and resets mode to 'api' as fallback"

requirements-completed: [SUB-05]

# Metrics
duration: 10min
completed: 2026-03-22
---

# Phase 23 Plan 06: Command Center Auth UI Summary

**Segmented API Key / Subscription toggle per LLM provider in Command Center settings, with OAuth popup flow for OpenAI/Gemini and token paste for Anthropic**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-22T05:46:45Z
- **Completed:** 2026-03-22T05:50:08Z
- **Tasks:** 3 (including 1 checkpoint)
- **Files modified:** 4

## Accomplishments

- Added `AuthStatus` and `AuthStartResponse` TypeScript interfaces to the API types module
- Added `api.auth.start/status/disconnect/submitToken` methods to the API client, wired to gateway OAuth endpoints
- Refactored LLM Provider Keys section into a full "LLM Providers" section with per-provider segmented control
- OAuth popup flow (OpenAI/Gemini): opens popup, polls every 2s until authenticated or 5min timeout
- Anthropic token paste flow: text input with instructions matching `agentix auth start anthropic` CLI UX
- Auth status badge displays green dot + "Connected" + optional account_id + "expires in Xh Ym"
- First-run wizard step 2 updated — API Key and Subscription presented at equal level, no default forced

## Task Commits

Each task was committed atomically:

1. **Task 1: Add auth API types and client methods** - `fc14456` (feat)
2. **Task 2: Update settings page with segmented provider mode toggle and OAuth flow** - `0febb27` (feat)
3. **Task 3: Verify settings page subscription UI** - checkpoint approved by user (no code commit)

## Files Created/Modified

- `apps/command-center/src/lib/api/types.ts` - Added AuthStatus and AuthStartResponse interfaces
- `apps/command-center/src/lib/api/client.ts` - Added api.auth namespace with start/status/disconnect/submitToken
- `apps/command-center/src/routes/settings/+page.svelte` - Full LLM Providers section with segmented control, OAuth/paste flows, auth status display
- `apps/command-center/src/lib/components/first-run-wizard.svelte` - Step 2 updated with equal-level API Key and Subscription choices

## Decisions Made

- OAuth popup uses `window.open()` with named target `agentix-auth` so clicking "Authorize" again reuses the same popup window instead of opening a duplicate
- Anthropic uses token paste flow (not popup) to match the CLI `agentix auth start anthropic` stdin UX — consistent cross-interface behavior
- No default mode is forced in the first-run wizard — both API Key and Subscription are presented as equal options per CONTEXT.md

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Command Center subscription UI is complete for all three providers
- Gateway OAuth endpoints (plan 04) are the integration target — functional end-to-end requires gateway running with OAuth credentials configured
- Ready for plan 07 (final docs/CHANGELOG for phase 23)

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
