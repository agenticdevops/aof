# Tutorial: Setting Up LLM Subscription Authentication

This tutorial walks through authenticating OpenAgentiX with your existing LLM subscription so your agents can use Anthropic, OpenAI, or Google without a separate API key.

## Prerequisites

- `agentix` CLI installed (`cargo install agentix`)
- An active LLM subscription: Claude Pro/Max, ChatGPT Plus, or Gemini Advanced
- For Gemini: a Google Cloud OAuth2 client ID and secret (see Step 3 — Google section)

---

## Step 1: Choose Your Provider

Each provider has a slightly different setup path:

| Provider | What you need | Setup time |
|----------|---------------|------------|
| **Anthropic** | `claude setup-token` output or an `sk-ant-api*` key | ~1 minute |
| **OpenAI** | Browser + ChatGPT Plus account | ~2 minutes |
| **Google Gemini** | Browser + Google Cloud OAuth2 client | ~5 minutes |

You can set up multiple providers — each is independent.

---

## Step 2: Authenticate

### Anthropic (Claude)

Anthropic uses a token paste flow. The token comes from `claude setup-token` (if you have Claude CLI installed) or a standard `sk-ant-api*` API key.

**Get your token:**

```bash
# If you have the claude CLI:
claude setup-token
# Outputs a JWT token starting with eyJ...

# Or use a standard Anthropic API key from console.anthropic.com
```

**Authenticate:**

```bash
agentix auth start anthropic
# Paste your Anthropic token (from `claude setup-token` or an API key):
# <paste token here and press Enter>
# Anthropic authentication saved (mode: bearer)
```

The tool automatically detects the credential type:
- JWT tokens (starting with `eyJ`) are stored as Bearer tokens
- API keys (starting with `sk-ant-api`) are stored as API keys

---

### OpenAI (ChatGPT Plus)

OpenAI uses browser-based OAuth2 with PKCE.

```bash
agentix auth start openai
# Opening browser for OpenAI authentication...
# Waiting for callback at http://localhost:1455/auth/callback
```

Your browser opens to `auth.openai.com`. Log in with your ChatGPT account and approve the authorization request.

```
# OpenAI authentication successful! Account: acct_abc123
```

If your environment has no browser (remote server, CI), use the device code flow:

```bash
agentix auth start openai --device-code
# Device code: ABCD-1234
# Visit: https://auth.openai.com/device
# Enter the code at that URL, then press Enter here...
```

---

### Google Gemini

Gemini requires OAuth2 client credentials from Google Cloud Console.

**One-time setup: create OAuth2 credentials**

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a project (or select an existing one)
3. Enable the **Generative Language API**
4. Under **APIs & Services → Credentials**, create an **OAuth 2.0 Client ID**
5. Application type: **Desktop app**
6. Add `http://localhost:1456/auth/callback` to Authorized redirect URIs
7. Copy the Client ID and Client Secret

**Set environment variables:**

```bash
export GEMINI_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GEMINI_OAUTH_CLIENT_SECRET="your-client-secret"
```

**Authenticate:**

```bash
agentix auth start gemini
# Opening browser for Gemini authentication...
# Waiting for callback at http://localhost:1456/auth/callback
```

Your browser opens to Google's OAuth consent page. Authorize access with your Google account (that has Gemini Advanced).

```
# Gemini authentication successful! Account: you@example.com
```

---

## Step 3: Verify Authentication

Check that all providers are authenticated:

```bash
agentix auth status
```

Example output:

```
Provider    Status          Mode     Account          Expires
----------  --------------  -------  ---------------  ----------
Anthropic   Connected       token    -                Never
OpenAI      Connected       oauth    acct_abc123      23h 15m
Gemini      Not connected   -        -                -
```

- **Connected** with **Never** expiry: token-based credential (Anthropic)
- **Connected** with **expiry time**: OAuth token (auto-refreshes before expiry)
- **Not connected**: not yet authenticated for this provider

---

## Step 4: Configure agentix.yaml

Set the provider mode in your workspace configuration:

```yaml
# quickstart/agentix.yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-workspace

spec:
  defaults:
    model: anthropic/claude-sonnet-4-6

  providers:
    anthropic:
      mode: subscription   # Use stored OAuth/token credentials
    openai:
      mode: subscription
    google:
      mode: subscription
      oauth:
        client_id: "${GEMINI_OAUTH_CLIENT_ID}"
        client_secret: "${GEMINI_OAUTH_CLIENT_SECRET}"

  gateway:
    host: "127.0.0.1"
    port: 7777

  agents_dir: "./agents"
```

You can use different modes per provider — for example, Anthropic subscription + OpenAI API key:

```yaml
spec:
  providers:
    anthropic:
      mode: subscription
    openai:
      api_key: "${OPENAI_API_KEY}"
```

---

## Step 5: Run an Agent

Start the gateway and run an agent — it uses your subscription transparently:

```bash
# Start gateway
agentix gateway start --config quickstart/agentix.yaml

# Run an agent
agentix run agent agents/my-agent
```

The agent makes requests using your stored OAuth token. No API key environment variable is needed for providers in subscription mode.

---

## Setting Up via Command Center UI

If you prefer a graphical interface, use the Command Center settings page.

**Access settings:**

1. Open the Command Center at `http://localhost:7777` (after starting the gateway)
2. Navigate to **Settings → LLM Providers**

**Per-provider controls:**

Each provider has a segmented control: **API Key** | **Subscription**

- Select **API Key** to enter or update an API key directly
- Select **Subscription** to use OAuth authentication

**Authenticate via UI:**

- **OpenAI / Gemini**: Click **Authorize** — a popup opens to the provider's OAuth page. After you authorize, the popup closes and status shows "Connected"
- **Anthropic**: A token paste input appears. Paste your token from `claude setup-token` and click **Save Token**

**Auth status display:**

After connecting, each provider card shows:

```
● Connected  (acct_abc123)  expires in 23h 14m
```

---

## Disconnecting

To remove stored credentials for a provider:

```bash
# CLI
agentix auth disconnect openai
# Disconnected from openai.
```

Or in Command Center: click **Disconnect** in the provider card (visible when connected).

---

## Troubleshooting

**`GEMINI_OAUTH_CLIENT_ID not set`**
Set the required environment variables before running `agentix auth start gemini`.

**Browser does not open**
Try the device code flow: `agentix auth start openai --device-code`

**`OAuth token expired and refresh failed`**
Your subscription may have expired or the token was revoked. Re-run `agentix auth start <provider>`.

**Agent fails with `Not authenticated for <provider>`**
Run `agentix auth status` to check which providers are connected, then run `agentix auth start <provider>`.

**`401 Unauthorized` from provider API**
The token may be expired. Run `agentix auth start <provider>` to get fresh credentials.

---

## Next Steps

- [Subscription Proxy Feature Overview](../features/subscription-proxy.md) — architecture, security, limitations
- [CLI Auth Reference](../reference/cli-auth.md) — full `agentix auth` command reference
