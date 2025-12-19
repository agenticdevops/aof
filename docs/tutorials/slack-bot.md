# Tutorial: Build a Slack Bot with AOF

Build an AI-powered Slack bot that helps your team with DevOps operations, answers questions, and executes commands with human approval.

**What you'll build:**
- A Slack bot powered by AI (Gemini, Claude, or GPT)
- Full DevOps capabilities: kubectl, docker, helm, terraform, git
- Human-in-the-loop approval for destructive commands
- Conversation memory for context

**Time:** 15 minutes

## Quick Start

```bash
# Set environment variables
export SLACK_BOT_TOKEN=xoxb-xxxxx
export SLACK_SIGNING_SECRET=xxxxx
export GOOGLE_API_KEY=xxxxx

# Start AOF server
aofctl serve --config examples/config/daemon.yaml

# In another terminal, expose with ngrok
brew install ngrok
ngrok http 3000

# Use the URL from ngrok output for Slack webhook:
# https://xxxx-xx-xx-xx-xx.ngrok-free.app/webhook/slack
```

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Slack workspace with admin access
- Google AI API key (or Anthropic/OpenAI)
- Optional: Kubernetes cluster, Docker for kubectl/docker commands

---

## Step 1: Create Slack App

1. Go to [api.slack.com/apps](https://api.slack.com/apps)
2. Click **Create New App** → **From scratch**
3. Name: `AOF Bot` (or your preferred name)
4. Select your workspace and click **Create App**

### Configure OAuth Scopes

Go to **OAuth & Permissions** → **Bot Token Scopes** and add these scopes:

| Scope | Purpose |
|-------|---------|
| `app_mentions:read` | Respond to @mentions |
| `chat:write` | Send messages |
| `commands` | Slash commands (`/aof`) |
| `im:history` | Read DM history |
| `reactions:read` | Read reactions for approval workflow |
| `reactions:write` | Add ✅/❌ reactions for approval |

### Install App and Get Tokens

1. Click **Install to Workspace** and authorize
2. Copy the **Bot User OAuth Token** (`xoxb-...`)
3. Go to **Basic Information** → copy **Signing Secret**

```bash
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_SIGNING_SECRET=your-signing-secret
```

### Enable Event Subscriptions

1. Go to **Event Subscriptions** → Toggle **Enable Events** to **On**
2. Set **Request URL** to: `https://your-tunnel-url/webhook/slack`
   - You'll set up the tunnel in Step 3
3. Under **Subscribe to bot events**, add:

| Event | Purpose | Required Scope |
|-------|---------|----------------|
| `app_mention` | Respond when @mentioned | `app_mentions:read` |
| `message.im` | Respond to DMs | `im:history` |
| `reaction_added` | Process approval reactions | `reactions:read` |
| `reaction_removed` | Track approval changes | `reactions:read` |

4. Click **Save Changes**

### Create Slash Command

Create a slash command for quick access without @mentioning the bot. You can use any command name you prefer:

1. Go to **Slash Commands** → **Create New Command**
2. Fill in:
   - **Command:** `/aof` (or `/devops`, `/k8s`, `/ops`, etc.)
   - **Request URL:** `https://your-ngrok-url/webhook/slack`
   - **Short Description:** `Ask the DevOps AI assistant`
   - **Usage Hint:** `[question or command]`
3. Click **Save**

You can create multiple slash commands (e.g., `/k8s`, `/docker`, `/infra`) - they all route to the same AOF backend.

**Note:** Both @mentions and slash commands work identically. Choose based on preference:
- `@AOF Bot show me pods` - Mention-based (good for threaded conversations)
- `/aof show me pods` - Slash command (quick, no need to type bot name)

### Invite Bot to Channels

In Slack, invite your bot to channels:
```
/invite @AOF Bot
```

---

## Step 2: Set Up Environment

### Required Environment Variables

```bash
# Slack credentials
export SLACK_BOT_TOKEN=xoxb-xxxxx-xxxxx-xxxxx
export SLACK_SIGNING_SECRET=xxxxx

# LLM API key (choose one)
export GOOGLE_API_KEY=xxxxx           # Recommended for speed
# export ANTHROPIC_API_KEY=sk-ant-xxx # Claude
# export OPENAI_API_KEY=sk-xxx        # GPT-4

# Optional: For kubectl commands
export KUBECONFIG=~/.kube/config
```

**Tip:** Add these to `~/.zshrc` or `~/.bashrc` to persist across sessions.

---

## Step 3: Start the Server

AOF includes a ready-to-use config at `examples/config/daemon.yaml` with the `devops` agent as default.

### Start with ngrok

```bash
# Terminal 1: Start AOF server
aofctl serve --config examples/config/daemon.yaml

# Terminal 2: Expose with ngrok (free account required)
brew install ngrok
ngrok http 3000
```

ngrok will output a URL like:
```
https://xxxx-xx-xx-xx-xx.ngrok-free.app
```

Set this URL in **two places** in your Slack app:

1. **Event Subscriptions** → Request URL:
   ```
   https://xxxx-xx-xx-xx-xx.ngrok-free.app/webhook/slack
   ```

2. **Slash Commands** → Request URL (for each command):
   ```
   https://xxxx-xx-xx-xx-xx.ngrok-free.app/webhook/slack
   ```

---

## Step 4: Test the Bot

### Test 1: Simple Question

In Slack, mention your bot or use the slash command:
```
@AOF Bot what's the difference between a Deployment and a StatefulSet?
```
or:
```
/aof what's the difference between a Deployment and a StatefulSet?
```

The bot should respond with a clear explanation.

### Test 2: Run a Command

```
@AOF Bot show me running pods
```
or:
```
/aof show me running pods
```

Response:
```
🔍 Checking pods...

NAME                        READY   STATUS    AGE
nginx-7d5b4c8b9-abc12       1/1     Running   5d
redis-master-0              1/1     Running   10d
```

### Test 3: Approval Workflow

Ask for a destructive operation:
```
@AOF Bot delete the nginx pod
```

The bot responds with an approval request:
```
⚠️ This action requires approval
`kubectl delete pod nginx-7d5b4c8b9-abc12 -n default`

React with ✅ to approve or ❌ to deny.
```

The bot adds ✅ and ❌ reactions. React with:
- **✅** to approve and execute
- **❌** to deny

After approval:
```
⚡ Executing approved command...
`kubectl delete pod nginx-7d5b4c8b9-abc12 -n default`

✅ Command completed successfully
pod "nginx-7d5b4c8b9-abc12" deleted
Approved by: @yourname
```

---

## The Default Agent: devops

The config uses the `devops` agent from `examples/agents/devops.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: devops

spec:
  model: google:gemini-2.5-flash
  temperature: 0  # Deterministic for operations

  tools:
    - kubectl
    - docker
    - helm
    - terraform
    - git
    - shell

  system_prompt: |
    You are a senior DevOps engineer with expertise across the entire stack.
    You help with infrastructure, deployments, automation, and operations.

    ## Safety Guardrails
    For destructive operations (delete, destroy, prune, force):
    - ALWAYS warn the user about potential impact
    - Request approval before executing: requires_approval: true
    - Suggest testing in non-prod first
```

The agent has access to: `kubectl`, `docker`, `helm`, `terraform`, `git`, and `shell`.

---

## How Approval Works

1. **Agent detects destructive command** - Based on system prompt guidelines
2. **Agent outputs approval request**:
   ```
   requires_approval: true
   command: "kubectl delete pod nginx-xyz"
   ```
3. **Handler posts approval message** - With ✅/❌ reactions
4. **User reacts** - ✅ approves, ❌ denies
5. **Command executes** - Only on approval

### Supported Reactions

**Approve:** ✅ `white_check_mark`, ✔️ `heavy_check_mark`, 👍 `+1`

**Deny:** ❌ `x`, ⛔ `no_entry`, 👎 `-1`

### Restrict Who Can Approve

Edit `examples/config/daemon.yaml` to add approver whitelist:

```yaml
platforms:
  slack:
    enabled: true
    bot_token_env: SLACK_BOT_TOKEN
    signing_secret_env: SLACK_SIGNING_SECRET

    # Only these users can approve destructive commands
    approval_allowed_users:
      - U12345678  # Find ID: Click profile → More → Copy member ID
      - U87654321
```

Restart the server after changing config.

---

## Conversation Memory

AOF maintains conversation context per channel/thread:

```
You: @bot show pods in production
Bot: [shows pods including nginx-abc123]

You: @bot describe the nginx one
Bot: [describes nginx-abc123 - knows which pod from context]
```

Memory is configured in the daemon config and persists across messages.

---

## Production Deployment

### 1. Use a Proper HTTPS Endpoint

Replace ngrok with a production deployment:

```bash
# Deploy behind nginx/Caddy with SSL
aofctl serve --config config/daemon.yaml --port 3000

# Configure reverse proxy to https://your-domain.com/webhook/slack
```

### 2. Systemd Service

```ini
# /etc/systemd/system/aof-bot.service
[Unit]
Description=AOF Slack Bot
After=network.target

[Service]
Type=simple
User=aof
Environment=SLACK_BOT_TOKEN=xoxb-xxx
Environment=SLACK_SIGNING_SECRET=xxx
Environment=GOOGLE_API_KEY=xxx
ExecStart=/usr/local/bin/aofctl serve --config /etc/aof/daemon.yaml
Restart=always

[Install]
WantedBy=multi-user.target
```

### 3. Docker Deployment

```bash
docker run -d \
  -e SLACK_BOT_TOKEN=$SLACK_BOT_TOKEN \
  -e SLACK_SIGNING_SECRET=$SLACK_SIGNING_SECRET \
  -e GOOGLE_API_KEY=$GOOGLE_API_KEY \
  -p 3000:3000 \
  aof:latest serve --config /config/daemon.yaml
```

---

## Troubleshooting

### Bot doesn't respond

1. Check bot is invited to the channel: `/invite @AOF Bot`
2. Verify Request URL in Event Subscriptions ends with `/webhook/slack`
3. Check server logs:
   ```bash
   RUST_LOG=debug aofctl serve --config examples/config/daemon.yaml
   ```
4. Ensure ngrok is running

### "URL verification failed"

- Server must be running before setting Request URL
- URL must end with `/webhook/slack`
- Check signing secret is correct

### Approval reactions not working

1. Ensure `reaction_added` event is subscribed
2. Check `reactions:read` and `reactions:write` scopes
3. Verify bot was invited to the channel

### "Unauthorized to approve"

Add the user's Slack ID to `approval_allowed_users`:
```yaml
approval_allowed_users:
  - U12345678  # Your user ID
```

Find ID: Click profile → More (⋯) → Copy member ID

### Commands timing out

Increase timeout in config:
```yaml
runtime:
  task_timeout_secs: 600  # 10 minutes
```

---

## Next Steps

- [Telegram Bot Tutorial](telegram-ops-bot.md) - Mobile-friendly read-only bot
- [Approval Workflow Guide](../guides/approval-workflow.md) - Advanced approval configuration
- [Agent Spec Reference](../reference/agent-spec.md) - Create custom agents
- [DaemonConfig Reference](../reference/daemon-config.md) - Full configuration options
