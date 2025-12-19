# Tutorial: Build a Slack Bot with AOF

Build an AI-powered Slack bot that helps your team with Kubernetes operations, answers questions, and executes commands with human approval.

**What you'll learn:**
- Set up AOF with Slack integration
- Create agents with tools (kubectl, helm, etc.)
- Implement human-in-the-loop approval workflow
- Use conversation memory for context

**Time:** 15 minutes

## Quick Start

```bash
# Set up environment
export SLACK_BOT_TOKEN=xoxb-xxxxx
export SLACK_SIGNING_SECRET=xxxxx
export GOOGLE_API_KEY=xxxxx

# Start AOF server
aofctl serve --config examples/config/slack-daemon.yaml

# In another terminal, expose with cloudflared (free, no signup)
brew install cloudflared
cloudflared tunnel --url http://localhost:3000

# Use the URL from cloudflared output for Slack webhook:
# https://random-words.trycloudflare.com/webhook/slack
```

See [Slack App Setup Guide](../guides/slack-app-setup.md) for detailed Slack configuration.

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Slack workspace with admin access
- Google AI API key (or Anthropic/OpenAI)
- Optional: Kubernetes cluster for kubectl commands

## Step 1: Create Slack App

1. Go to [api.slack.com/apps](https://api.slack.com/apps)
2. Click **Create New App** > **From scratch**
3. Name: `K8s Ops Bot`, select your workspace

### Configure OAuth Scopes

Go to **OAuth & Permissions** and add these Bot Token Scopes:

| Scope | Purpose |
|-------|---------|
| `chat:write` | Send messages |
| `app_mentions:read` | Respond to @mentions |
| `reactions:read` | Read approval reactions |
| `reactions:write` | Add approval buttons |

### Install and Get Tokens

1. Click **Install to Workspace**
2. Copy **Bot User OAuth Token** (`xoxb-...`)
3. Go to **Basic Information** > copy **Signing Secret**

```bash
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_SIGNING_SECRET=your-signing-secret
```

### Enable Event Subscriptions

1. Go to **Event Subscriptions** > Enable Events
2. Request URL: `https://your-url/webhook/slack` (set up in Step 4)
3. Subscribe to bot events:
   - `app_mention` - Bot mentions
   - `message.channels` - Channel messages
   - `message.im` - Direct messages
   - `reaction_added` - **Required for approval workflow**

## Step 2: Create Agent Configuration

Create `agents/k8s-ops.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    platform: slack
    capability: kubernetes

spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "Kubernetes operations assistant"

  tools:
    - kubectl
    - helm

  system_prompt: |
    You are a Kubernetes operations assistant in Slack.

    ## Your Role
    - Answer K8s questions clearly and concisely
    - Run kubectl/helm commands when requested
    - Troubleshoot cluster issues
    - Format responses for Slack (use code blocks)

    ## Safety Rules
    For destructive operations (delete, scale to 0, rollout restart):

    1. Warn the user about the impact
    2. Return this format to trigger approval:

    requires_approval: true
    command: "kubectl delete pod nginx-xyz -n production"

    The system will then ask for human approval before executing.

    ## Response Format
    - Keep responses concise (this is chat, not email)
    - Use code blocks for command output
    - Use emoji for status: ✅ success, ⚠️ warning, ❌ error
```

## Step 3: Create Daemon Configuration

Create `config/slack-daemon.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: slack-k8s-bot

spec:
  server:
    port: 3000
    host: "0.0.0.0"

  platforms:
    slack:
      enabled: true
      bot_token_env: SLACK_BOT_TOKEN
      signing_secret_env: SLACK_SIGNING_SECRET

      # Optional: Restrict who can approve commands
      # approval_allowed_users:
      #   - U12345678  # Slack user IDs

  agents:
    directory: "./agents"

  runtime:
    default_agent: k8s-ops
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

## Step 4: Start the Server

```bash
# Start AOF server
aofctl serve --config config/slack-daemon.yaml --agents-dir agents/

# In another terminal, expose with cloudflared
cloudflared tunnel --url http://localhost:3000
# Output: https://random-words.trycloudflare.com

# Set the Slack webhook URL to:
# https://random-words.trycloudflare.com/webhook/slack
```

## Step 5: Test the Bot

### Test 1: Simple Question

In Slack, mention your bot:

```
@K8s Ops Bot what's the difference between a Deployment and StatefulSet?
```

Response:
```
**Deployment** - For stateless apps:
- Pods are interchangeable
- Easy horizontal scaling
- Examples: web servers, APIs

**StatefulSet** - For stateful apps:
- Stable pod identities (pod-0, pod-1)
- Persistent storage per pod
- Examples: databases, message queues
```

### Test 2: Run a Command

```
@K8s Ops Bot show me pods in the default namespace
```

Response:
```
🔍 Checking pods in default namespace...

NAME                        READY   STATUS    AGE
nginx-7d5b4c8b9-abc12       1/1     Running   5d
redis-master-0              1/1     Running   10d
```

### Test 3: Approval Workflow

Ask for a destructive operation:

```
@K8s Ops Bot delete the nginx pod
```

The bot responds with an approval request:

```
⚠️ This action requires approval
`kubectl delete pod nginx-7d5b4c8b9-abc12 -n default`

React with ✅ to approve or ❌ to deny.
```

The bot adds ✅ and ❌ reactions. Click ✅ to approve:

```
⚡ Executing approved command...
`kubectl delete pod nginx-7d5b4c8b9-abc12 -n default`

✅ Command completed successfully
pod "nginx-7d5b4c8b9-abc12" deleted
Approved by: @yourname
```

Or click ❌ to deny:

```
❌ Action denied by @yourname
`kubectl delete pod nginx-7d5b4c8b9-abc12 -n default`
```

## How Approval Works

1. **Agent detects destructive command** - Based on system prompt guidelines
2. **Agent outputs approval request**:
   ```
   requires_approval: true
   command: "kubectl delete pod nginx-xyz"
   ```
3. **Handler parses output** - Detects `requires_approval: true`
4. **Approval message sent** - With ✅/❌ reactions
5. **User reacts** - ✅ approves, ❌ denies
6. **Command executes** - Only on approval

### Configure Approvers

Restrict who can approve commands:

```yaml
platforms:
  slack:
    enabled: true
    bot_token_env: SLACK_BOT_TOKEN
    signing_secret_env: SLACK_SIGNING_SECRET

    # Only these users can approve
    approval_allowed_users:
      - U12345678  # SRE Lead
      - U87654321  # Platform Lead
```

Find Slack User IDs: Click user profile > More (...) > Copy member ID

## Conversation Memory

AOF maintains conversation context per channel/thread. The bot remembers previous messages:

```
You: @bot show pods in production
Bot: [shows pods including nginx-abc123]

You: @bot describe the nginx one
Bot: [describes nginx-abc123 - knows which pod you mean from context]
```

## Production Setup

### 1. Use HTTPS Endpoint

Replace cloudflared/ngrok with a proper deployment:

```bash
# Deploy behind nginx/Caddy with SSL
aofctl serve --config config/slack-daemon.yaml --port 3000

# Configure reverse proxy to https://your-domain.com/webhook/slack
```

### 2. Configure Logging

```bash
RUST_LOG=info aofctl serve --config config/slack-daemon.yaml
```

### 3. Add More Agents

Create specialized agents in `agents/`:

```
agents/
├── k8s-ops.yaml      # Kubernetes
├── docker-ops.yaml   # Docker
├── aws-agent.yaml    # AWS CLI
└── devops.yaml       # Full-stack
```

### 4. Systemd Service

```ini
# /etc/systemd/system/aof-slack.service
[Unit]
Description=AOF Slack Bot
After=network.target

[Service]
Type=simple
User=aof
Environment=SLACK_BOT_TOKEN=xoxb-xxx
Environment=SLACK_SIGNING_SECRET=xxx
Environment=GOOGLE_API_KEY=xxx
ExecStart=/usr/local/bin/aofctl serve --config /etc/aof/slack-daemon.yaml
Restart=always

[Install]
WantedBy=multi-user.target
```

## Troubleshooting

### Bot doesn't respond

```bash
# Check webhook is set
curl "https://api.slack.com/api/apps.connections.open" # Should return ok

# Check server logs
RUST_LOG=debug aofctl serve --config config/slack-daemon.yaml

# Verify webhook URL ends with /webhook/slack
```

### Approval reactions not working

1. Ensure `reaction_added` event is subscribed in Slack app
2. Check `reactions:read` and `reactions:write` scopes are added
3. Verify bot was invited to the channel

### "Unauthorized to approve"

Add the user's Slack ID to `approval_allowed_users` in config, then restart:

```yaml
approval_allowed_users:
  - U12345678
```

### Commands timing out

Increase timeout in config:

```yaml
runtime:
  task_timeout_secs: 600  # 10 minutes
```

## Next Steps

- [Approval Workflow Guide](../guides/approval-workflow.md) - Advanced approval configuration
- [Telegram Bot Tutorial](./telegram-ops-bot.md) - Mobile-friendly read-only bot
- [Multi-Model RCA](./multi-model-rca.md) - Root cause analysis with multiple AI models
