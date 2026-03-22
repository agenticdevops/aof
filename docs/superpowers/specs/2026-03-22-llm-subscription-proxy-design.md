# LLM Subscription Proxy — Design Spec

**Date:** 2026-03-22
**Status:** Draft
**Phase:** New capability (post Phase 22)

---

## Goal

Allow agents to use existing LLM subscriptions (Claude Pro/Max, ChatGPT Plus, Gemini Advanced) instead of paying for separate API access. Pure OAuth/session-based authentication — no API keys required for subscription mode.

## Motivation

Users already pay for LLM subscriptions. Requiring separate API keys with separate billing creates unnecessary cost duplication. The subscription proxy lets agents piggyback on existing subscriptions for cost reduction.

---

## Architecture

### New Modules

```
agentix-llm/src/provider/
├── anthropic.rs              # existing API provider (unchanged)
├── openai.rs                 # existing API provider (unchanged)
├── google.rs                 # existing API provider (unchanged)
├── subscription/
│   ├── mod.rs                # shared types, token store trait, rate limiter
│   ├── auth.rs               # OAuth2 helpers (PKCE, token exchange, refresh)
│   ├── anthropic_sub.rs      # Claude web API adapter
│   ├── openai_sub.rs         # ChatGPT backend-api adapter
│   └── google_sub.rs         # Gemini OAuth2 adapter
```

### Provider Factory Routing

```rust
// ProviderFactory::create() extended
match (provider, config.mode) {
    (Anthropic, Some(ProviderMode::Subscription)) => subscription::anthropic_sub::create(config),
    (Anthropic, _)                                 => anthropic::create(config),
    (OpenAI, Some(ProviderMode::Subscription))     => subscription::openai_sub::create(config),
    (OpenAI, _)                                    => openai::create(config),
    (Google, Some(ProviderMode::Subscription))     => subscription::google_sub::create(config),
    (Google, _)                                    => google::create(config),
}
```

Each subscription provider implements `Model` trait — agents are unaware of the difference.

---

## Configuration

### Workspace Config (`agentix.yaml`)

```yaml
spec:
  providers:
    anthropic:
      mode: subscription
      # No api_key needed — uses OAuth/session auth
    openai:
      mode: subscription
    google:
      mode: subscription
      oauth:
        client_id: "${GOOGLE_OAUTH_CLIENT_ID}"
        client_secret: "${GOOGLE_OAUTH_CLIENT_SECRET}"
```

### ProviderConfig Extension

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    /// "api" (default) or "subscription"
    #[serde(default)]
    pub mode: Option<ProviderMode>,
    /// OAuth settings for subscription mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    Api,
    Subscription,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}
```

---

## Provider Details

### 1. Google Gemini (Official OAuth2)

**Status:** Officially supported, production-ready.

**Auth flow:**
1. Gateway generates authorization URL:
   ```
   https://accounts.google.com/o/oauth2/v2/auth?
     client_id=...&
     redirect_uri=http://localhost:7777/api/v1/oauth/google/callback&
     response_type=code&
     scope=https://www.googleapis.com/auth/generative-language&
     access_type=offline&
     prompt=consent
   ```
2. User authorizes in browser.
3. Google redirects to callback with `?code=AUTH_CODE`.
4. Gateway exchanges code for `access_token` + `refresh_token`.
5. Refresh token stored in SQLite. Access token refreshed automatically.

**API endpoint:**
```
POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
Authorization: Bearer ya29.a0...
```

**Streaming:**
```
POST https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?alt=sse
```

**Rate limits (free tier):**
- Gemini 2.0 Flash: 15 RPM, 1M TPM
- Gemini 2.5 Pro: 5 RPM, 250K TPM

**Implementation:** `google_sub.rs` — wraps the existing `GoogleProvider` request/response format but swaps API key auth for OAuth bearer token.

### 2. Anthropic Claude (Subscription OAuth / setup-token)

**Status:** Semi-official. Uses the same OAuth mechanism as Claude Code to call the official Messages API, billed to the user's Claude Pro/Max subscription.

**Auth flow — Option A: setup-token (simplest):**
1. User runs `claude setup-token` on any machine with Claude Code authenticated.
2. Copies the generated token string.
3. Pastes it into Command Center settings or `agentix.yaml`.
4. Gateway stores the OAuth token pair and uses it for API calls.

**Auth flow — Option B: Direct OAuth:**
1. Gateway initiates OAuth flow against `console.anthropic.com`.
2. User authorizes in browser.
3. Gateway receives callback with authorization code.
4. Exchanges code for access_token + refresh_token.

**API endpoint (same official Messages API as API key mode):**
```
POST https://api.anthropic.com/v1/messages
Authorization: Bearer {oauth_access_token}
```

Key insight: subscription mode uses the **same official API endpoint** as API key mode. Only the auth header changes (OAuth bearer vs `x-api-key`). Billing goes through the Claude subscription.

**Adapter responsibilities:**
- Swap `x-api-key` header for `Authorization: Bearer` header
- Handle OAuth token refresh automatically
- Re-prompt user via WebSocket event if refresh token expires
- Handle subscription rate limits (different from API tier limits)

**Rate limits:** Rolling message caps per model, not published. Implement adaptive rate limiting — track 429s and back off.

### 3. OpenAI ChatGPT (Unofficial Backend API)

**Status:** Unofficial. Uses chatgpt.com internal API. Actively defended with Cloudflare/CAPTCHAs.

**Auth flow:**
1. Gateway initiates Auth0 login flow against `auth0.openai.com/authorize`.
2. User authenticates in browser.
3. Gateway captures the JWT access token from the Auth0 callback.
4. Access token stored in SQLite. Refreshed via Auth0 refresh token flow.

**Auth0 details:**
- Authorize URL: `https://auth0.openai.com/authorize`
- Token URL: `https://auth0.openai.com/oauth/token`
- Client ID: Extracted from ChatGPT web app (changes periodically)
- Scopes: `openid profile email offline_access`

**API endpoint:**
```
POST https://chatgpt.com/backend-api/conversation
Authorization: Bearer eyJhbGciOi...
```

**Request format:**
```json
{
  "action": "next",
  "messages": [{
    "id": "uuid",
    "author": { "role": "user" },
    "content": { "content_type": "text", "parts": ["Hello"] }
  }],
  "model": "gpt-4o",
  "parent_message_id": "uuid"
}
```

**Response:** SSE stream with `data: {json}` lines, terminated by `data: [DONE]`.

**Adapter responsibilities:**
- Translate `ModelRequest` → ChatGPT backend-api format
- Manage conversation state (parent_message_id chain)
- Parse SSE stream → `ModelResponse`
- Handle Cloudflare challenges (retry with delay, log warning)
- Re-authenticate on 401/403
- Periodically refresh the Auth0 client_id from the web app

**Rate limits:** Rolling message caps per model. Implement adaptive rate limiting.

---

## Token Storage

### SQLite Auth Store

New SQLite database: `{data_dir}/auth.db`

```sql
CREATE TABLE oauth_tokens (
    provider TEXT PRIMARY KEY,
    access_token TEXT,
    refresh_token TEXT,
    session_token TEXT,       -- for claude.ai session key
    token_type TEXT,          -- "oauth2" | "session"
    expires_at TEXT,          -- ISO 8601
    metadata TEXT,            -- JSON blob for provider-specific data (org_id, client_id, etc.)
    updated_at TEXT NOT NULL
);
```

**Auth store trait:**
```rust
pub trait AuthStore: Send + Sync {
    fn get_token(&self, provider: &str) -> Result<Option<StoredToken>>;
    fn save_token(&self, provider: &str, token: StoredToken) -> Result<()>;
    fn delete_token(&self, provider: &str) -> Result<()>;
}
```

**Token refresh logic:**
- Before each LLM call, check if access token is expired
- If expired and refresh token exists → exchange for new access token
- If no refresh token or refresh fails → mark provider as "needs re-auth"
- Gateway broadcasts `provider_auth_expired` WebSocket event so Command Center can prompt user

---

## Gateway API

### OAuth Flow Endpoints

```
GET  /api/v1/oauth/:provider/start     → Returns { "auth_url": "https://..." }
GET  /api/v1/oauth/:provider/callback   → Handles OAuth redirect, stores tokens
GET  /api/v1/oauth/:provider/status     → Returns auth status for a provider
DELETE /api/v1/oauth/:provider           → Revoke/delete stored tokens
```

### `GET /api/v1/oauth/:provider/start`

Generates the authorization URL and returns it. The Command Center opens this in a popup.

**Response:**
```json
{
  "auth_url": "https://accounts.google.com/o/oauth2/v2/auth?client_id=...&redirect_uri=...&scope=...",
  "provider": "google"
}
```

For providers without official OAuth (Anthropic, OpenAI):
- Returns a local gateway page URL that handles the login flow
- The page guides the user through authenticating and capturing the session/token

### `GET /api/v1/oauth/:provider/callback`

Handles the OAuth redirect. Exchanges authorization code for tokens, stores them, and redirects back to Command Center.

### `GET /api/v1/oauth/:provider/status`

```json
{
  "provider": "google",
  "authenticated": true,
  "token_type": "oauth2",
  "expires_at": "2026-03-22T15:30:00Z",
  "needs_reauth": false
}
```

### `DELETE /api/v1/oauth/:provider`

Revokes the token (if provider supports revocation) and deletes from auth store.

---

## Rate Limiting

Each subscription provider gets an adaptive rate limiter:

```rust
pub struct SubscriptionRateLimiter {
    /// Maximum requests per minute (starts conservative, adjusts)
    rpm_limit: AtomicU32,
    /// Current window request count
    window_count: AtomicU32,
    /// When the current window started
    window_start: Mutex<Instant>,
    /// Consecutive 429 count (triggers limit reduction)
    consecutive_429s: AtomicU32,
}
```

**Behavior:**
- Start with conservative limits (5 RPM for Claude/ChatGPT, 15 RPM for Gemini)
- On 429, halve the RPM limit and back off
- On sustained success, gradually increase toward the initial limit
- Expose current rate limit status in `GET /api/v1/oauth/:provider/status`

---

## Command Center UI

### Settings Page — Provider Cards

Each provider card (already built in Item 1) gets extended:

```
┌─ Anthropic ─────────────────────────────────────┐
│ Mode:  [API ▼] / [Subscription ▼]              │
│                                                  │
│ [API mode]                                       │
│ API Key: [sk-ant-***key1]  [Save]               │
│                                                  │
│ [Subscription mode]                              │
│ Status: ● Authenticated (expires in 23h)        │
│ [Re-authorize]  [Disconnect]                     │
└──────────────────────────────────────────────────┘
```

### Wizard Step 2

After "Test Connection" succeeds:
1. Provider key inputs (already built)
2. NEW: Toggle per provider for "Use my subscription instead"
3. If subscription selected → "Authorize" button opens OAuth popup

---

## WebSocket Events

New event types for auth lifecycle:

```json
{"type": "provider_auth_expired", "provider": "anthropic", "message": "Session expired, re-authorization needed"}
{"type": "provider_auth_success", "provider": "google", "message": "OAuth token refreshed"}
{"type": "provider_rate_limited", "provider": "openai", "retry_after_secs": 30}
```

---

## Error Handling

| Scenario | Behavior |
|---|---|
| Token expired, refresh succeeds | Transparent to agent, retry the LLM call |
| Token expired, refresh fails | Return `AuthExpired` error, broadcast WS event |
| 429 rate limited | Back off, retry once after delay, then propagate error |
| Provider web API changed | Return `ProviderError` with details, log warning |
| Cloudflare challenge (OpenAI) | Return `ProviderBlocked` error, suggest re-auth |

All subscription errors implement `std::error::Error` and map to `AgentixError::Provider(...)`.

---

## Testing Strategy

### Unit Tests
- Token storage (SQLite auth store) CRUD
- Rate limiter behavior (429 backoff, recovery)
- Request/response translation per provider (mock HTTP)

### Integration Tests
- OAuth flow with mock OAuth server (wiremock)
- Token refresh lifecycle
- Fallback from subscription → API key when auth fails

### Manual Testing
- Actual OAuth flow against Google (requires real credentials)
- Session capture for Claude/OpenAI (requires real subscriptions)

---

## Implementation Order

1. **Core infrastructure** — `ProviderMode` enum, `OAuthConfig`, `AuthStore` SQLite
2. **Google Gemini OAuth** — Official, stable, proves the architecture
3. **Gateway OAuth endpoints** — `/oauth/:provider/start|callback|status`
4. **Command Center UI** — Mode toggle, authorize button, status display
5. **Anthropic Claude adapter** — Unofficial web API + session capture
6. **OpenAI ChatGPT adapter** — Unofficial backend-api + Auth0 flow
7. **Rate limiting** — Adaptive per-provider rate limiters
8. **WebSocket events** — Auth lifecycle events

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Anthropic/OpenAI change web APIs | Subscription providers break | Isolated modules, easy to update. API mode always available as fallback. |
| Cloudflare blocks automated ChatGPT access | OpenAI subscription unusable | Document as known limitation. User can fall back to API mode. |
| Claude session expires frequently | User must re-authenticate often | Cache sessions aggressively. Clear UX for re-auth in Command Center. |
| ToS violations | Account suspension risk | Document risk clearly. User's choice to enable. |
| OAuth client_id changes (OpenAI) | Auth flow breaks | Fetch client_id dynamically from web app, with hardcoded fallback. |

---

## Non-Goals

- No browser automation (Playwright/Puppeteer) — too fragile and heavy
- No token scraping from user's browser — user authenticates through our flow
- No paid proxy services (OpenRouter, LiteLLM) — direct subscription access only
- No changes to the `Model` trait — subscription providers conform to existing interface
