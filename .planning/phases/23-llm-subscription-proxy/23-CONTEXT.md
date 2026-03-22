# Phase 23: LLM Subscription Proxy - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable agents to authenticate with LLM providers (Anthropic, Google, OpenAI) via users' existing subscriptions using OAuth, instead of requiring separate API keys. Covers OAuth flows, token persistence, provider adapters, rate limiting, CLI auth command, and Command Center UI controls.

**Critical constraint:** Copy authentication logic from `/Users/gshah/work/opsflow-sh/experiments/zeroclaw` — all three provider OAuth flows are already implemented and working there. Do NOT reinvent auth patterns.

</domain>

<decisions>
## Implementation Decisions

### Auth Flow UX
- All three providers use OAuth2 with PKCE (copied from zeroclaw)
- Google: Browser popup from Command Center + CLI fallback (`agentix auth start google`)
- Anthropic: Paste setup-token in Command Center settings OR CLI. If OAuth is supported, use that.
- OpenAI: Standard OAuth2 with PKCE — uses `auth.openai.com` endpoints (NOT unofficial)
- CLI command: uniform `agentix auth start <provider>` for all providers (kubectl-style)
- Both UI popup and CLI paths must work for all providers

### Token Storage
- Copy zeroclaw's encrypted JSON profile store (~/.agentix/auth-profiles.json)
- ChaCha20-Poly1305 AEAD encryption with master key at ~/.agentix/.secret_key
- NOT SQLite — use zeroclaw's proven pattern
- TokenSet struct: access_token, refresh_token, id_token, expires_at, token_type, scope
- AuthProfile struct: provider, profile_name, kind (OAuth/Token), token_set, metadata

### OAuth Callback
- Copy zeroclaw's per-provider localhost ports (OpenAI:1455, Gemini:1456)
- Separate loopback servers per provider, NOT routed through gateway
- Device code flow fallback for headless environments (copy zeroclaw)

### Provider Priority & Coexistence
- Ship all three providers (Google, Anthropic, OpenAI) in the same phase
- User explicitly chooses active mode per provider (no automatic switching)
- Segmented control in UI: "API Key" | "Subscription" per provider
- API key and subscription are at the same level — API key is optional, not the default

### Failure & Fallback
- Auto-refresh tokens using zeroclaw's pattern: 90s skew window, 3 retries, 350ms × attempt backoff, 10s failure delay
- If refresh fails, the agent run FAILS — no automatic fallback to API key
- Per-profile async locking to prevent concurrent refresh

### Rate Limiting
- Copy zeroclaw's token refresh retry pattern (3 attempts, exponential backoff)
- No additional adaptive RPM limiting at this stage

### Subscription UI in Settings
- Segmented control per provider: "API Key" | "Subscription"
- Selecting "Subscription" shows authorize button, "API Key" shows key input
- Auth status display: Claude's discretion on exact design
- No WebSocket auth events — poll via GET endpoint when user visits settings
- First-run wizard shows subscription and API key at same level — let users choose. API key is NOT the default/primary path

### Claude's Discretion
- Auth status badge design (connected/expiry display)
- Exact provider card layout in settings
- Loading states during OAuth flow
- Error message wording for failed auth
- PKCE code verifier/challenge generation details (copy zeroclaw)

</decisions>

<specifics>
## Specific Ideas

- Copy auth logic wholesale from `/Users/gshah/work/opsflow-sh/experiments/zeroclaw/src/auth/` — this includes:
  - `openai_oauth.rs` — OAuth2 with PKCE, client_id `app_EMoamEEZ73f0CkXaXp7hrann`
  - `gemini_oauth.rs` — OAuth2 with PKCE, env vars for client_id/secret
  - `anthropic_token.rs` — Hybrid API key / Bearer token with JWT detection
  - `oauth_common.rs` — Shared PKCE, URL encoding, loopback server, query parsing
  - `profiles.rs` — AuthProfile, TokenSet, encrypted storage, profile selection
  - `mod.rs` — AuthService for managing all provider credentials
- Zeroclaw's profile selection priority: explicit override → active profile → default → first profile
- Zeroclaw's JWT detection for Anthropic: 2+ dots = Bearer, sk-ant-api prefix = API key

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 23-llm-subscription-proxy*
*Context gathered: 2026-03-22*
