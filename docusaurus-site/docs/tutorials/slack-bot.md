# Tutorial: Build a Slack Bot Agent

Build an AI-powered Slack bot that helps your team with Kubernetes operations, answers questions, and can execute commands on approval.

**What you'll learn:**
- Create a Slack-integrated agent
- Handle Slack events and commands
- Implement human-in-the-loop approvals
- Deploy as an AgentFlow

**Time:** 20 minutes

## Prerequisites

- `aofctl` installed
- Slack workspace (admin access to create apps)
- Kubernetes cluster (for kubectl commands)
- OpenAI or Anthropic API key

## Step 1: Create Slack App

1. Go to [https://api.slack.com/apps](https://api.slack.com/apps)
2. Click **Create New App** → **From scratch**
3. Name: `K8s Assistant Bot`
4. Select your workspace

### Configure Bot

1. **OAuth & Permissions** → Add scopes:
   - `chat:write` - Send messages
   - `app_mentions:read` - Receive mentions
   - `commands` - Slash commands

2. **Install App** → Install to workspace
3. Copy **Bot User OAuth Token** (starts with `xoxb-`)

### Configure Event Subscriptions

1. **Event Subscriptions** → Enable Events
2. Subscribe to bot events:
   - `app_mention` - When bot is mentioned
   - `message.channels` - Channel messages

3. Request URL: `https://your-domain.com/slack/events` (we'll set this up later)

### Create Slash Command

1. **Slash Commands** → Create New Command
2. Command: `/k8s`
3. Request URL: `https://your-domain.com/slack/commands`
4. Description: "Ask the K8s assistant"

Save your tokens:
```bash
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_SIGNING_SECRET=your-signing-secret
```

## Step 2: Create the Base Agent

Create `slack-k8s-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: slack-k8s-bot
  labels:
    platform: slack
    purpose: operations

spec:
  model: openai:gpt-4

  model_config:
    temperature: 0.3
    max_tokens: 1500

  instructions: |
    You are a helpful Kubernetes assistant in a Slack channel.

    Your role:
    - Answer K8s questions clearly and concisely
    - Help troubleshoot cluster issues
    - Run kubectl commands when requested (with approval for destructive ops)
    - Format responses for Slack (use markdown, code blocks)

    Guidelines:
    - Keep responses short - this is Slack, not email
    - Use emoji occasionally for readability 🚀
    - For complex answers, offer to run commands instead
    - Always ask for approval before destructive operations (delete, scale down, etc.)
    - Use Slack's code block formatting: ```text```

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        timeout_seconds: 30

    - type: MCP
      config:
        name: kubectl-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-kubectl"]
        env:
          KUBECONFIG: "${KUBECONFIG}"

    - type: Slack
      config:
        bot_token: ${SLACK_BOT_TOKEN}
        signing_secret: ${SLACK_SIGNING_SECRET}

  memory:
    type: SQLite
    config:
      path: ./slack-bot-memory.db
      # Separate memory per Slack channel
      context_key: "slack_channel_${SLACK_CHANNEL_ID}"
```

## Step 3: Create AgentFlow for Event Handling

The agent handles logic, but we need a flow to route Slack events:

Create `slack-bot-flow.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: slack-k8s-bot-flow

spec:
  # Listen for Slack events
  trigger:
    type: Slack
    config:
      events:
        - app_mention          # @bot-name
        - message             # Direct messages
        - slash_command       # /k8s command
      bot_token: ${SLACK_BOT_TOKEN}
      signing_secret: ${SLACK_SIGNING_SECRET}

  # Define the workflow
  nodes:
    # Extract and parse the message
    - id: parse-message
      type: Transform
      config:
        script: |
          # Extract text, user, channel
          export MESSAGE_TEXT="${event.text}"
          export SLACK_USER="${event.user}"
          export SLACK_CHANNEL="${event.channel}"
          export SLACK_TIMESTAMP="${event.ts}"

    # Route to agent
    - id: agent-process
      type: Agent
      config:
        agent: slack-k8s-bot
        input: ${MESSAGE_TEXT}
        context:
          slack_channel: ${SLACK_CHANNEL}
          slack_user: ${SLACK_USER}

    # Check if approval needed
    - id: check-approval
      type: Conditional
      config:
        condition: |
          # If agent wants to run destructive command
          ${agent-process.requires_approval} == true

    # Request approval via Slack
    - id: request-approval
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        message: |
          :warning: **Approval Required**

          User: <@${SLACK_USER}>
          Command: `${agent-process.command}`

          React with :white_check_mark: to approve or :x: to deny
        wait_for_reaction: true
        timeout_seconds: 300  # 5 minutes

    # Execute approved command
    - id: execute-command
      type: Agent
      config:
        agent: slack-k8s-bot
        input: "Execute the approved command: ${agent-process.command}"
      conditions:
        - from: check-approval
          value: true
        - from: request-approval
          reaction: white_check_mark

    # Send response back to Slack
    - id: send-response
      type: Slack
      config:
        channel: ${SLACK_CHANNEL}
        thread_ts: ${SLACK_TIMESTAMP}  # Reply in thread
        message: ${agent-process.output}

  # Define connections
  connections:
    - from: parse-message
      to: agent-process

    - from: agent-process
      to: check-approval

    - from: check-approval
      to: request-approval
      when: requires_approval == true

    - from: check-approval
      to: send-response
      when: requires_approval == false

    - from: request-approval
      to: execute-command

    - from: execute-command
      to: send-response
```

## Step 4: Deploy the Bot

```bash
# Deploy the agent
aofctl apply -f slack-k8s-agent.yaml

# Deploy the flow
aofctl apply -f slack-bot-flow.yaml

# Start the flow (listens for Slack events)
aofctl run agentflow slack-k8s-bot-flow --daemon

# Check status
aofctl describe agentflow slack-k8s-bot-flow
```

## Step 5: Set Up Webhook Endpoint

You need a public HTTPS endpoint for Slack to send events to.

### Option A: Use ngrok (Development)

```bash
# Install ngrok
brew install ngrok

# Start ngrok
ngrok http 3000

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
# Update Slack Event Subscriptions URL to:
# https://abc123.ngrok.io/slack/events
```

### Option B: Deploy to Production

```bash
# Deploy AOF server (built-in to aofctl)
aofctl run agentflow slack-k8s-bot-flow --port 3000

# Or use a reverse proxy (nginx, Caddy)
# Point Slack webhook to: https://your-domain.com/slack/events
```

## Step 6: Test the Bot

### Test 1: Simple Question

In Slack:
```
@K8s Assistant What's the difference between a Deployment and a StatefulSet?
```

Expected response:
```
🤖 Great question!

**Deployment**: Best for stateless apps
- Pods are interchangeable
- Can scale up/down easily
- No stable network identity
- Examples: web servers, APIs

**StatefulSet**: For stateful apps
- Pods have stable identities (pod-0, pod-1...)
- Persistent storage per pod
- Ordered deployment/scaling
- Examples: databases, message queues

Need help choosing which to use? 🚀
```

### Test 2: Command Execution

In Slack:
```
@K8s Assistant Show me all pods in the production namespace
```

Expected response:
```
🔍 Fetching pods in production namespace...

```
NAME                        READY   STATUS    RESTARTS   AGE
nginx-deployment-abc123     2/2     Running   0          5d
postgres-statefulset-0      1/1     Running   0          10d
redis-deployment-xyz789     1/1     Running   1          3d
```

All pods look healthy! ✅
```

### Test 3: Approval Flow

In Slack:
```
@K8s Assistant Scale the nginx deployment to 0 replicas
```

Expected response:
```
⚠️ **Approval Required**

User: @yourname
Command: `kubectl scale deployment nginx-deployment --replicas=0 -n production`

React with ✅ to approve or ❌ to deny
```

After you react with ✅:
```
✅ Approved and executed!

deployment.apps/nginx-deployment scaled to 0 replicas

Note: This will cause downtime. Monitor in #production-alerts
```

### Test 4: Slash Command

In Slack:
```
/k8s show failing pods
```

Expected response:
```
🔍 Checking for failing pods...

Found 2 pods with issues:

1. **api-deployment-abc123** (default)
   Status: CrashLoopBackOff
   Restarts: 5
   Error: Error: ECONNREFUSED connecting to database

2. **worker-xyz789** (production)
   Status: ImagePullBackOff
   Error: Failed to pull image "myregistry/worker:v2.0.0"

Want me to investigate further? 🔧
```

## Step 7: Add Advanced Features

### Feature 1: Daily Cluster Report

Add to `slack-bot-flow.yaml`:

```yaml
spec:
  # Add schedule trigger
  triggers:
    - type: Schedule
      config:
        cron: "0 9 * * *"  # 9 AM daily
        timezone: America/New_York

  nodes:
    - id: daily-report
      type: Agent
      config:
        agent: slack-k8s-bot
        input: |
          Generate a daily cluster health report:
          - Total pods and their status
          - Any failing deployments
          - Resource usage summary
          - Top 5 pods by CPU/memory

    - id: send-report
      type: Slack
      config:
        channel: "#platform-daily"
        message: |
          📊 **Daily K8s Cluster Report**

          ${daily-report.output}
```

### Feature 2: Auto-Remediation

Add incident detection and auto-fix:

```yaml
nodes:
  - id: detect-incident
    type: Agent
    config:
      agent: slack-k8s-bot
      input: "Check for any failing pods or deployments"

  - id: auto-fix
    type: Conditional
    config:
      condition: ${detect-incident.has_issues} == true

  - id: attempt-remediation
    type: Agent
    config:
      agent: slack-k8s-bot
      input: "Try to auto-remediate: ${detect-incident.issues}"

  - id: notify-team
    type: Slack
    config:
      channel: "#incidents"
      message: |
        🚨 **Auto-Remediation Attempt**

        Issue: ${detect-incident.issues}
        Action: ${attempt-remediation.action}
        Result: ${attempt-remediation.result}

        cc: @oncall
```

### Feature 3: Interactive Buttons

Add Slack interactive components:

```yaml
nodes:
  - id: send-interactive
    type: Slack
    config:
      channel: ${SLACK_CHANNEL}
      blocks:
        - type: section
          text:
            type: mrkdwn
            text: "Found failing pod: `nginx-abc123`"

        - type: actions
          elements:
            - type: button
              text: "Restart Pod"
              action_id: restart_pod
              value: "nginx-abc123"

            - type: button
              text: "View Logs"
              action_id: view_logs
              value: "nginx-abc123"

            - type: button
              text: "Describe Pod"
              action_id: describe_pod
              value: "nginx-abc123"
```

## Step 8: Monitor and Debug

```bash
# View flow logs
aofctl logs agentflow slack-k8s-bot-flow -f

# Check agent performance
aofctl describe agent slack-k8s-bot

# Debug failed events
aofctl describe agentflow slack-k8s-bot-flow

# View Slack API errors
aofctl logs agentflow slack-k8s-bot-flow --filter="error"
```

## Production Checklist

Before deploying to production:

- [ ] Set up HTTPS endpoint (not ngrok)
- [ ] Configure proper Slack scopes
- [ ] Set up rate limiting (Slack has API limits)
- [ ] Add error handling and retries
- [ ] Configure persistent memory (SQLite → PostgreSQL)
- [ ] Set up monitoring and alerts
- [ ] Document available commands for team
- [ ] Test approval flows thoroughly
- [ ] Set up log retention
- [ ] Configure backup for memory database

## Troubleshooting

### Bot doesn't respond to mentions

```bash
# Check Slack event subscriptions
# Verify Request URL is reachable
curl https://your-domain.com/slack/events

# Check flow is running
aofctl describe agentflow slack-k8s-bot-flow

# View flow logs
aofctl logs agentflow slack-k8s-bot-flow -f
```

### Approval reactions not working

```bash
# Verify Slack scopes include reactions
# Check reaction timeout (default 5 minutes)
# Test with simple emoji reactions first
```

### Commands timing out

```bash
# Increase timeout in flow config
timeout_seconds: 60  # Increase from 30

# Check kubectl is working
kubectl get pods

# Test agent directly
aofctl exec agent slack-k8s-bot -- "show pods"
```

## Next Steps

- **[Incident Response Tutorial](incident-response.md)** - Auto-remediation flows
- **[AgentFlow Reference](../reference/agentflow-spec.md)** - Advanced flow patterns
- **[Example Flows](../examples/)** - More Slack bot examples

---

**🎉 Your Slack bot is live!** Your team can now get K8s help without leaving Slack.
