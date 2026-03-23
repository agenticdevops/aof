# LLM Subscription Proxy

Use your existing LLM subscriptions — Claude Pro/Max, ChatGPT Plus, Gemini Advanced — to power agents instead of requiring separate API keys.

## Overview

OpenAgentiX supports two authentication modes per LLM provider:

- **API Key mode** — standard API key (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`)
- **Subscription mode** — OAuth token from your existing LLM subscription

In subscription mode, agents send requests using your OAuth token. The provider bills against your subscription, not a pay-per-token API account. This is useful for:

- Teams with Claude Max or ChatGPT Plus subscriptions already in use
- Developers who want to avoid separate API billing
- Environments where managing API keys is operationally complex

## Supported Providers

| Provider | Subscription Required | Auth Method | Endpoint |
|----------|-----------------------|-------------|----------|
| Anthropic | Claude Pro or Max | Token paste from `claude setup-token` | Official Messages API |
| OpenAI | ChatGPT Plus or higher | OAuth2 PKCE via `auth.openai.com` | Official Chat API |
| Google | Gemini Advanced | OAuth2 PKCE via `accounts.google.com` | Generative Language API |

Both modes can be configured simultaneously — different agents can use different modes.

## How It Works

### OAuth Flow

For OpenAI and Google, the authentication flow uses OAuth2 with PKCE:

1. `agentix auth start <provider>` opens your browser to the provider's OAuth consent page
2. A local callback server listens on a dedicated port (OpenAI: 1455, Gemini: 1456)
3. After you authorize, the callback captures the authorization code
4. The code is exchanged for access + refresh tokens using PKCE verification
5. Tokens are encrypted and stored at `~/.agentix/auth-profiles.json`

For Anthropic, a token paste flow is used:

1. Run `claude setup-token` (or obtain a Claude API key) to get your token
2. `agentix auth start anthropic` prompts you to paste the token
3. The token is encrypted and stored alongside OAuth tokens

### Token Refresh

OAuth tokens are automatically refreshed before expiry:

- Tokens are refreshed when they are within 90 seconds of expiry
- Up to 3 refresh attempts with exponential backoff (350ms × attempt)
- If refresh fails after all retries, the agent run fails with a clear error message
- Per-profile async locking prevents concurrent refresh races

There is no automatic fallback to API key if refresh fails. This is by design — silent mode switching would cause unexpected billing behavior.

### Transparent to Agents

Agents do not know whether they are using an API key or subscription. The `Model` trait implementation is the same — the only difference is the authentication header:

- Anthropic subscription: `Authorization: Bearer <token>` (instead of `x-api-key`)
- OpenAI subscription: `Authorization: Bearer <token>` (unchanged — OpenAI already uses Bearer)
- Google subscription: `Authorization: Bearer <token>` (instead of `?key=<api-key>` URL param)

### Provider Mode Routing

When `mode: subscription` is set in `agentix.yaml`, the runtime injects `provider_mode=subscription` into the model configuration. The `ProviderFactory` routes this to the subscription adapter for the provider.

## Configuration

Add subscription mode to your `agentix.yaml`:

```yaml
spec:
  providers:
    anthropic:
      mode: subscription   # Use stored OAuth/token credentials
    openai:
      mode: subscription   # Use stored OAuth credentials
    google:
      mode: subscription   # Use stored OAuth credentials
      oauth:
        client_id: "${GEMINI_OAUTH_CLIENT_ID}"
        client_secret: "${GEMINI_OAUTH_CLIENT_SECRET}"
```

Without `mode: subscription`, providers default to API key mode:

```yaml
spec:
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
    openai:
      api_key: "${OPENAI_API_KEY}"
    google:
      api_key: "${GOOGLE_API_KEY}"
```

You can mix modes — for example, use subscription for Anthropic and API key for OpenAI.

## Token Storage

Credentials are stored at `~/.agentix/auth-profiles.json` using AES-256-GCM encryption.

The encryption key is stored at `~/.agentix/.secret_key` with 0600 permissions. Each credential entry uses the `enc2:` prefix to indicate encrypted content:

```
enc2:<hex(nonce || ciphertext)>
```

Plaintext credentials are never written to disk. If an existing plaintext entry is detected during load, it is automatically re-encrypted and persisted.

Profile selection priority (when multiple profiles exist for a provider):

1. Explicit override (via API or programmatic use)
2. Active profile (`active_profiles[provider]` in the store)
3. Profile named `default`
4. First available profile for the provider

## Security Considerations

- Encrypted token storage: AES-256-GCM with a per-machine key
- No network calls from the storage layer — all encryption/decryption is local
- Tokens are not logged; `Debug` implementations redact `TokenSet` fields
- OAuth callback servers listen on loopback only (`127.0.0.1`), not accessible remotely
- PKCE prevents authorization code interception attacks

## Limitations

- **Anthropic**: The Anthropic platform does not expose a standard OAuth2 endpoint. The token paste flow uses the same credential as `claude setup-token` — this means it is tied to your account quota.
- **OpenAI**: The subscription endpoint is the standard Chat Completions API. Rate limits from your subscription tier apply.
- **Google Gemini**: Requires your own OAuth2 client credentials (`GEMINI_OAUTH_CLIENT_ID` and `GEMINI_OAUTH_CLIENT_SECRET`). Create an OAuth2 client in Google Cloud Console with redirect URI `http://localhost:1456/auth/callback`.
- **No auto-fallback**: If OAuth tokens expire and cannot be refreshed, the agent run fails. There is no silent switch back to API key mode.
- **Subscription limits**: Your agent usage counts against your subscription's usage limits (tokens, requests). Heavy agent workloads may hit rate limits faster than a dedicated API account.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| `Not authenticated for anthropic` | No stored credentials | Run `agentix auth start anthropic` |
| `OAuth token expired and refresh failed` | Network error or subscription cancelled | Run `agentix auth start <provider>` to re-authenticate |
| `GEMINI_OAUTH_CLIENT_ID not set` | Missing env var for Gemini | Set the required environment variables |
| `401 Unauthorized` from provider | Token rejected (subscription inactive) | Check your subscription status, then re-authenticate |

## Related

- [CLI Auth Reference](../reference/cli-auth.md) — `agentix auth` command reference
- [Subscription Setup Tutorial](../tutorials/subscription-setup.md) — step-by-step setup guide
- [Quickstart Config](../../quickstart/agentix.yaml) — example `agentix.yaml` with subscription mode
