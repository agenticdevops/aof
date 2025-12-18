# Quickstart Guide

Get your first agentic workflow running in **5 minutes**. This guide walks you through creating a simple Kubernetes operations bot triggered by Telegram.

## Prerequisites

- **Terminal** with bash/zsh
- **API Key** from one of:
  - [Google AI Studio](https://aistudio.google.com/apikey) (Gemini) - Recommended
  - [OpenAI](https://platform.openai.com/api-keys) (GPT-4)
  - [Ollama](https://ollama.ai/) (local, no key needed)
- **Telegram** account (we'll create a bot)

## Step 1: Install AOF

Run the installation script:

```bash
curl -sSL https://docs.aof.sh/install.sh | bash
```

This detects your platform (Linux/macOS/Windows) and installs the latest `aofctl` binary.

**Verify installation:**
```bash
aofctl version
# Output: aofctl version 0.1.15
```

**Alternative - Build from source:**
```bash
git clone https://github.com/agenticdevops/aof.git
cd aof
cargo build --release
sudo cp target/release/aofctl /usr/local/bin/
```

## Step 2: Set Up API Key

Export your LLM provider API key:

```bash
# Google Gemini (recommended)
export GOOGLE_API_KEY="AIza..."

# OR OpenAI
export OPENAI_API_KEY="sk-..."

# OR Ollama (local, no key needed)
# brew install ollama && ollama serve
```

**💡 Tip**: Add to `~/.zshrc` or `~/.bashrc` to persist:
```bash
echo 'export GOOGLE_API_KEY="AIza..."' >> ~/.zshrc
source ~/.zshrc
```

## Step 3: Your First Agent

Create a simple Kubernetes operations agent:

**File: `agents/k8s-ops.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-ops
  labels:
    category: infrastructure
spec:
  model: google:gemini-2.5-flash
  temperature: 0.3
  max_tokens: 4096

  instructions: |
    You are an expert Kubernetes operations engineer.
    Help users manage clusters, diagnose issues, and troubleshoot problems.

    ## Your Capabilities
    - Execute kubectl commands
    - Check pod/deployment status
    - View logs and describe resources
    - Provide troubleshooting advice

    ## Safety Rules
    For commands that modify or delete resources, output:
    ```
    requires_approval: true
    command: "kubectl delete pod nginx-12345"
    reason: "This will terminate the running pod"
    ```

  tools:
    - kubectl
    - shell

  memory:
    type: InMemory
    config:
      max_messages: 20
```

**Test it:**
```bash
mkdir -p agents
# Save the above YAML to agents/k8s-ops.yaml

# Run with a query
aofctl run agent agents/k8s-ops.yaml --input "Show me all pods in the default namespace"
```

You should see the agent respond with kubectl output!

## Step 4: Create a Telegram Bot

We'll use Telegram as the trigger for our agent.

### 4.1 Create Bot with BotFather

1. Open Telegram and search for [@BotFather](https://t.me/botfather)
2. Send `/newbot`
3. Follow prompts to name your bot (e.g., "My K8s Ops Bot")
4. **Copy the bot token** (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 4.2 Get Your Chat ID

1. Start a chat with your new bot
2. Send any message (e.g., "hello")
3. Visit: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
4. Find `"chat":{"id":123456789}` in the JSON response
5. **Copy your chat ID**

### 4.3 Set Environment Variables

```bash
export TELEGRAM_BOT_TOKEN="123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
export TELEGRAM_CHAT_ID="123456789"
```

## Step 5: Create a Flow

Create an AgentFlow that connects your Telegram bot to the k8s-ops agent:

**File: `flows/telegram-k8s-bot.yaml`**
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: telegram-k8s-bot
  description: Kubernetes operations via Telegram
  labels:
    platform: telegram
    category: ops
spec:
  trigger:
    type: Telegram
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}
      allowed_chat_ids:
        - ${TELEGRAM_CHAT_ID}  # Only you can use this bot

  nodes:
    - id: parse
      type: Transform
      config:
        script: |
          # Extract message details
          export MESSAGE_TEXT="${event.message.text}"
          export CHAT_ID="${event.message.chat.id}"
          export USER="${event.message.from.username}"
          echo "Received from $USER: $MESSAGE_TEXT"

    - id: agent
      type: Agent
      config:
        agent: k8s-ops  # Reference to agents/k8s-ops.yaml
        input: ${MESSAGE_TEXT}

    - id: respond
      type: Telegram
      config:
        chat_id: ${CHAT_ID}
        message: ${agent.output}
        parse_mode: Markdown

  connections:
    - from: trigger
      to: parse
    - from: parse
      to: agent
    - from: agent
      to: respond
```

## Step 6: Start the Daemon

Now start the AOF daemon to serve your flow:

```bash
# Create flows directory
mkdir -p flows
# Save the flow YAML to flows/telegram-k8s-bot.yaml

# Start daemon
aofctl serve \
  --agents-dir ./agents \
  --flows-dir ./flows \
  --port 3000
```

You should see:
```
🚀 AOF Daemon starting...
✓ Loaded agent: k8s-ops
✓ Loaded flow: telegram-k8s-bot
✓ Telegram bot connected: @YourBotName
🎯 Listening on http://0.0.0.0:3000
```

## Step 7: Test Your Bot

Open Telegram and send a message to your bot:

```
/start
```

The bot should respond with a greeting!

Try a real query:
```
Show me all pods
```

The agent will:
1. Receive your message
2. Run `kubectl get pods -A`
3. Send results back to Telegram

**Example interaction:**
```
You: Show me all failing pods

Bot: Here are the failing pods I found:

NAMESPACE    NAME                    STATUS      RESTARTS
production   api-deployment-xyz      CrashLoop   5
staging      worker-abc123           Error       1

The api-deployment pod is crash looping. Let me check the logs...

Error: OOMKilled (Out of Memory)

Recommendation: Increase memory limits in the deployment.
```

## Step 8: Add Context (Environment-Specific Config)

Let's configure different contexts for production and development:

**File: `flows/telegram-k8s-bot.yaml`** (update)
```yaml
spec:
  trigger:
    type: Telegram
    config:
      bot_token: ${TELEGRAM_BOT_TOKEN}
      allowed_chat_ids: [${TELEGRAM_CHAT_ID}]

  # Add context configuration
  context:
    kubeconfig: ${KUBECONFIG_DEV}  # Point to dev cluster
    namespace: default
    env:
      ENVIRONMENT: "development"
      REQUIRE_APPROVAL: "false"  # No approval needed in dev

  nodes:
    # ... same as before
```

**For production:**
```yaml
  context:
    kubeconfig: ${KUBECONFIG_PROD}
    namespace: production
    env:
      ENVIRONMENT: "production"
      REQUIRE_APPROVAL: "true"  # Approval needed for prod
```

## Step 9: Create a FlowBinding (Future)

> **Note**: FlowBinding is planned for v1alpha2. For now, we configure everything in the AgentFlow spec.

In the future, you'll be able to separate concerns:

```yaml
---
# flows/telegram-k8s-bot.yaml
apiVersion: aof.dev/v1alpha2
kind: Flow
metadata:
  name: k8s-bot-flow
spec:
  nodes:
    - id: agent
      type: Agent
      config:
        agent: { ref: k8s-ops }  # Reference
---
# contexts/dev-cluster.yaml
apiVersion: aof.dev/v1alpha2
kind: Context
metadata:
  name: dev-cluster
spec:
  kubeconfig: ${KUBECONFIG_DEV}
  namespace: default
---
# bindings/telegram-dev-binding.yaml
apiVersion: aof.dev/v1alpha2
kind: FlowBinding
metadata:
  name: telegram-dev-binding
spec:
  trigger: { ref: telegram-bot }
  flow: { ref: k8s-bot-flow }
  agent: { ref: k8s-ops }
  context: { ref: dev-cluster }
```

This makes it easy to swap contexts (dev → staging → prod) without changing the flow logic.

## Next Steps

Congratulations! You've built your first agentic workflow. 🎉

### Learn More

- **[Core Concepts](concepts.md)** - Understand the 6 resource types
- **[Multi-Tenant Setup](../guides/enterprise-setup.md)** - Deploy across environments
- **[Agent YAML Reference](../reference/agent-spec.md)** - Complete spec

### Build More

Try these examples:

1. **Add Approval Workflow**
   - Require approval for destructive kubectl commands
   - See [Approval Workflow Guide](../guides/approval-workflow.md)

2. **Multi-Environment Setup**
   - Create separate flows for prod/staging/dev
   - Route based on Telegram chat ID
   - See [Enterprise Setup](../guides/enterprise-setup.md)

3. **Add More Agents**
   - Create a security scanner agent
   - Add incident response agent
   - Build an AgentFleet for code review

4. **Switch Platforms**
   - Replace Telegram with Slack
   - Add Discord bot
   - Set up WhatsApp integration

### Common Issues

**Bot not responding?**
- Check `aofctl serve` logs for errors
- Verify `TELEGRAM_BOT_TOKEN` is correct
- Ensure your chat ID is in `allowed_chat_ids`

**kubectl commands failing?**
- Verify `kubectl` is installed: `which kubectl`
- Check kubeconfig: `echo $KUBECONFIG`
- Test manually: `kubectl get pods`

**API rate limits?**
- Use Ollama locally (no rate limits)
- Or upgrade to paid tier on your LLM provider

### Example Directory Structure

After completing this guide, you should have:

```
my-aof-project/
├── agents/
│   └── k8s-ops.yaml          # Your K8s agent
├── flows/
│   └── telegram-k8s-bot.yaml # Telegram → Agent flow
└── .env                       # Environment variables

# Optional for multi-environment
├── contexts/
│   ├── dev-cluster.yaml      # Dev environment config
│   ├── staging-cluster.yaml  # Staging config
│   └── prod-cluster.yaml     # Production config
└── bindings/
    ├── dev-binding.yaml      # Wire dev components
    ├── staging-binding.yaml  # Wire staging components
    └── prod-binding.yaml     # Wire prod components
```

### Running in Production

For production deployments:

1. **Use persistent memory**:
   ```yaml
   memory:
     type: PostgreSQL
     config:
       url: postgres://user:pass@localhost/aof
   ```

2. **Configure logging**:
   ```bash
   aofctl serve --log-level info --log-file /var/log/aof/daemon.log
   ```

3. **Run as systemd service**:
   ```ini
   [Unit]
   Description=AOF Daemon
   After=network.target

   [Service]
   Type=simple
   User=aof
   ExecStart=/usr/local/bin/aofctl serve --agents-dir /etc/aof/agents --flows-dir /etc/aof/flows
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

4. **Set up monitoring**:
   - Track `agentflow_requests_total` metric
   - Monitor `agentflow_latency_seconds`
   - Alert on `agentflow_errors_total`

---

**Ready for enterprise deployment?** → [Enterprise Setup Guide](../guides/enterprise-setup.md)
