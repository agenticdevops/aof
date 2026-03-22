# CLI Reference: Auth Commands

Manage LLM subscription authentication for Anthropic, OpenAI, and Google/Gemini providers. Enables agents to use your existing LLM subscriptions instead of requiring separate API keys.

## `agentix auth start <provider>`

Initiate an OAuth flow for the specified provider.

### Synopsis

```
agentix auth start <provider> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `provider` | Provider name: `anthropic`, `openai`, `gemini` (aliases: `claude`, `codex`, `google`) |

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--device-code` | false | Use device code flow instead of browser popup (for headless environments) |

### Provider Behavior

**Anthropic:**
Prompts for a token from `claude setup-token` or a standard `sk-ant-api*` API key. Detects the credential type automatically.

```bash
agentix auth start anthropic
# Paste your Anthropic token (from `claude setup-token` or an API key):
# <enter token>
# Anthropic authentication saved (mode: bearer)
```

**OpenAI:**
Opens your browser to the OpenAI OAuth login page. Starts a local callback listener on port 1455. On success, stores OAuth tokens with automatic refresh.

```bash
agentix auth start openai
# Opening browser for OpenAI authentication...
# Waiting for callback at http://localhost:1455/auth/callback
# OpenAI authentication successful! Account: acct_abc123
```

**Gemini:**
Requires `GEMINI_OAUTH_CLIENT_ID` and `GEMINI_OAUTH_CLIENT_SECRET` environment variables. Opens browser to Google OAuth. Callback listener on port 1456.

```bash
export GEMINI_OAUTH_CLIENT_ID=...
export GEMINI_OAUTH_CLIENT_SECRET=...
agentix auth start gemini
# Opening browser for Gemini authentication...
# Gemini authentication successful! Account: user@example.com
```

### Device Code Flow (Headless)

For servers without a browser, pass `--device-code`:

```bash
agentix auth start openai --device-code
# Device code: ABCD-1234
# Visit: https://auth.openai.com/device
# Waiting for authorization...
# OpenAI authentication successful!

agentix auth start gemini --device-code
# Device code: ABCD-1234
# Visit: https://www.google.com/device
# Waiting for authorization...
```

---

## `agentix auth status`

Show authentication status for all providers.

### Synopsis

```
agentix auth status
```

### Output

```
Provider    Status          Mode     Account          Expires
----------  --------------  -------  ---------------  ----------
Anthropic   Connected       token    -                Never
OpenAI      Connected       oauth    acct_abc123      23h 15m
Gemini      Not connected   -        -                -
```

If no providers are connected:

```
No providers authenticated. Run `agentix auth start <provider>` to get started.
```

---

## `agentix auth disconnect <provider>`

Remove stored credentials for a provider.

### Synopsis

```
agentix auth disconnect <provider>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `provider` | Provider name: `anthropic`, `openai`, `gemini` |

### Examples

```bash
# Remove OpenAI credentials
agentix auth disconnect openai
# Disconnected from openai.

# Remove credentials that don't exist
agentix auth disconnect gemini
# No gemini credentials found.
```

---

## Provider Name Aliases

| Canonical | Aliases |
|-----------|---------|
| `anthropic` | `claude`, `claude-code` |
| `openai` | `codex`, `openai-codex` |
| `gemini` | `google`, `vertex` |

---

## Token Storage

Credentials are stored encrypted at `~/.agentix/auth-profiles.json` using AES-256-GCM. The encryption key is stored at `~/.agentix/.secret_key` with 0600 permissions.

OAuth tokens are refreshed automatically before expiry (90-second skew window) with up to 3 retries.

---

## Related Commands

- `agentix gateway start` — Start the gateway server (uses stored auth credentials for subscription mode)
- See [LLM Subscription Proxy](../guides/llm-subscription.md) for agent configuration
