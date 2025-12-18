# Single Bot Example - Simplest Setup

This is the **simplest possible AOF setup** - a single Slack bot for Kubernetes operations in development.

## What You Get

A Slack bot that responds to @mentions and messages in your #dev channel, helping with kubectl commands and K8s troubleshooting.

## Files

- `agent.yaml` - Single K8s operations agent
- `daemon-config.yaml` - Daemon configuration to run the bot

## Prerequisites

1. Slack App with Bot Token (see [Slack Setup Guide](../../../docs/tutorials/slack-bot.md))
2. kubectl configured with cluster access
3. AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)

## Quick Start

### 1. Set Environment Variables

```bash
export SLACK_BOT_TOKEN=xoxb-your-token-here
export SLACK_SIGNING_SECRET=your-signing-secret
export GOOGLE_API_KEY=your-gemini-api-key
```

### 2. Start the Bot

```bash
# From this directory
aofctl serve --config daemon-config.yaml
```

### 3. Expose Webhook (Development)

```bash
# In another terminal
ngrok http 3000

# Configure Slack App webhook URL:
# https://xxxx.ngrok.io/webhook/slack
```

### 4. Test in Slack

```
@k8s-bot get pods
@k8s-bot help me troubleshoot a crashing pod
@k8s-bot what's using the most CPU?
```

## Customization

### Change the Kubernetes Cluster

Edit `daemon-config.yaml`:
```yaml
contexts:
  kubeconfig: /path/to/your/kubeconfig
```

### Change the Slack Channel

Edit `daemon-config.yaml`:
```yaml
triggers:
  slack:
    channels:
      - your-channel-name
```

### Add More Capabilities

Add tools to `agent.yaml`:
```yaml
spec:
  tools:
    - kubectl
    - helm
    - docker  # Add this
```

## Next Steps

- Try the [Enterprise Example](../enterprise/) for multi-environment setup
- Read about [Agent Composition](../../agents/README.md)
- Learn [Flow Orchestration](../../flows/README.md)

## Production Deployment

**DO NOT use this configuration in production!**

For production, use:
- Separate contexts for each environment
- Approval workflows for destructive operations
- Audit logging
- Rate limiting
- User access controls

See [Enterprise Example](../enterprise/) for production-ready setup.
