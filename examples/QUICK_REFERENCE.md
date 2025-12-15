# AOF Examples - Quick Reference

## 📁 Directory Structure

```
examples/
├── quickstart/          # Start here!
│   ├── hello-world-agent.yaml
│   └── hello-world-flow.yaml
│
├── agents/              # Single-purpose agents
│   ├── kubernetes-ops-agent.yaml
│   ├── github-assistant-agent.yaml
│   ├── dockerfile-generator-agent.yaml
│   ├── terraform-planner-agent.yaml
│   ├── log-analyzer-agent.yaml
│   └── security-scanner-agent.yaml
│
├── fleets/              # Multi-agent teams
│   ├── code-review-fleet.yaml
│   └── sre-oncall-fleet.yaml
│
└── flows/               # Complete workflows
    ├── slack-qa-bot-flow.yaml
    ├── pr-review-flow.yaml
    ├── incident-auto-remediation-flow.yaml
    ├── daily-standup-report-flow.yaml
    ├── deploy-notification-flow.yaml
    └── cost-optimization-flow.yaml
```

---

## 🎯 Quick Start (5 minutes)

### 1. Setup
```bash
export ANTHROPIC_API_KEY=sk-ant-xxxxx
```

### 2. Deploy your first agent
```bash
cd examples
aof apply -f quickstart/hello-world-agent.yaml
```

### 3. Test it
```bash
aof query hello "What's your name?"
```

### 4. See it in action
```bash
aof logs hello
```

---

## 🚀 Common Use Cases

### "I want to automate Kubernetes operations"
→ [`agents/kubernetes-ops-agent.yaml`](agents/kubernetes-ops-agent.yaml)

```bash
aof apply -f agents/kubernetes-ops-agent.yaml
aof query k8s-ops "List failing pods in production"
```

### "I want to automate PR reviews"
→ [`flows/pr-review-flow.yaml`](flows/pr-review-flow.yaml)

```bash
export GITHUB_TOKEN=ghp_xxxxx
aof apply -f flows/pr-review-flow.yaml
# Auto-reviews new PRs
```

### "I want incident auto-remediation"
→ [`flows/incident-auto-remediation-flow.yaml`](flows/incident-auto-remediation-flow.yaml)

```bash
export PAGERDUTY_TOKEN=xxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f flows/incident-auto-remediation-flow.yaml
```

### "I want a Slack bot to answer questions"
→ [`flows/slack-qa-bot-flow.yaml`](flows/slack-qa-bot-flow.yaml)

```bash
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f flows/slack-qa-bot-flow.yaml
# Mention @qa-bot in Slack
```

### "I want daily cost analysis"
→ [`flows/cost-optimization-flow.yaml`](flows/cost-optimization-flow.yaml)

```bash
export AWS_ACCESS_KEY_ID=xxxxx
export AWS_SECRET_ACCESS_KEY=xxxxx
aof apply -f flows/cost-optimization-flow.yaml
# Runs at 8am daily
```

### "I want security scanning"
→ [`agents/security-scanner-agent.yaml`](agents/security-scanner-agent.yaml)

```bash
aof apply -f agents/security-scanner-agent.yaml
aof query sec-scanner "Scan Docker image myapp:latest"
```

---

## 📊 Complexity Guide

### ⭐ Beginner (Start here!)
- `quickstart/hello-world-agent.yaml` - Basic agent
- `quickstart/hello-world-flow.yaml` - Basic flow
- `agents/github-assistant-agent.yaml` - Simple API integration

### ⭐⭐ Intermediate
- `agents/kubernetes-ops-agent.yaml` - K8s operations
- `agents/dockerfile-generator-agent.yaml` - Code analysis
- `agents/terraform-planner-agent.yaml` - Infrastructure review
- `agents/log-analyzer-agent.yaml` - Log parsing
- `flows/daily-standup-report-flow.yaml` - Multi-source aggregation

### ⭐⭐⭐ Advanced
- `agents/security-scanner-agent.yaml` - Multi-tool integration
- `fleets/code-review-fleet.yaml` - Parallel agent coordination
- `fleets/sre-oncall-fleet.yaml` - Sequential workflows
- `flows/pr-review-flow.yaml` - Event-driven automation
- `flows/incident-auto-remediation-flow.yaml` - Decision trees + actions
- `flows/cost-optimization-flow.yaml` - Complex analytics

---

## 🔧 Environment Variables Cheat Sheet

```bash
# Core (required for all)
export ANTHROPIC_API_KEY=sk-ant-xxxxx

# GitHub integration
export GITHUB_TOKEN=ghp_xxxxx

# Slack integration
export SLACK_BOT_TOKEN=xoxb-xxxxx
export SLACK_SIGNING_SECRET=xxxxx

# AWS
export AWS_ACCESS_KEY_ID=xxxxx
export AWS_SECRET_ACCESS_KEY=xxxxx
export AWS_DEFAULT_REGION=us-west-2

# GCP
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/creds.json

# Azure
export AZURE_CLIENT_ID=xxxxx
export AZURE_CLIENT_SECRET=xxxxx
export AZURE_TENANT_ID=xxxxx

# PagerDuty
export PAGERDUTY_TOKEN=xxxxx

# Jira
export JIRA_TOKEN=xxxxx
export JIRA_URL=https://yourcompany.atlassian.net

# Datadog
export DATADOG_API_KEY=xxxxx
export DATADOG_APP_KEY=xxxxx
```

---

## 🎨 Customization Quick Tips

### Change the AI model
```yaml
spec:
  model: claude-3-5-sonnet-20241022  # Balanced (default)
  # model: claude-3-opus-20240229    # Most powerful
  # model: claude-3-haiku-20240307   # Fastest/cheapest
```

### Adjust creativity
```yaml
spec:
  temperature: 0    # Deterministic (for ops)
  # temperature: 0.7  # Balanced
  # temperature: 1    # Creative (for content)
```

### Set resource limits
```yaml
spec:
  resources:
    max_tokens: 4096
    timeout: 60s
```

### Add safety guardrails
```yaml
spec:
  guardrails:
    - type: command_filter
      config:
        blocked_patterns:
          - "rm -rf"
          - "kubectl delete ns production"
```

### Enable memory
```yaml
spec:
  memory:
    type: conversation
    max_messages: 50
```

---

## 📝 Common Commands

```bash
# Apply configuration
aof apply -f <file.yaml>

# Query an agent
aof query <agent-name> "<question>"

# List resources
aof list agents
aof list fleets
aof list flows

# View logs
aof logs <resource-name>

# Get status
aof status <resource-name>

# Delete resource
aof delete agent <name>
aof delete fleet <name>
aof delete flow <name>

# Trigger flow manually
aof trigger <flow-name> --data '{"key": "value"}'

# Validate YAML
aof validate -f <file.yaml>

# Dry run
aof apply -f <file.yaml> --dry-run
```

---

## 🐛 Troubleshooting

### "Agent not responding"
```bash
# Check if agent is running
aof status <agent-name>

# View logs
aof logs <agent-name>

# Verify API key
echo $ANTHROPIC_API_KEY
```

### "Permission denied"
```bash
# Check tool permissions in agent spec
# Ensure kubectl/docker/etc. are accessible
which kubectl
```

### "Rate limit exceeded"
```yaml
# Add rate limiting to agent
spec:
  guardrails:
    - type: rate_limit
      config:
        max_requests_per_minute: 20
```

### "Agent response too slow"
```yaml
# Use faster model
spec:
  model: claude-3-haiku-20240307

# Or reduce max_tokens
spec:
  resources:
    max_tokens: 2048
```

---

## 📚 Learning Path

**Day 1: Basics**
1. Run `quickstart/hello-world-agent.yaml`
2. Modify the system prompt
3. Try different queries

**Day 2: Single Agents**
1. Deploy `agents/kubernetes-ops-agent.yaml`
2. Test with real K8s cluster
3. Add custom commands

**Day 3: Fleets**
1. Deploy `fleets/code-review-fleet.yaml`
2. See parallel execution
3. Customize reviewers

**Day 4: Flows**
1. Deploy `flows/slack-qa-bot-flow.yaml`
2. Test trigger → agent → action
3. Add error handling

**Day 5: Production**
1. Deploy `flows/incident-auto-remediation-flow.yaml`
2. Test with real alerts
3. Add monitoring

---

## 🔗 Helpful Links

- [Full README](README.md) - Detailed documentation
- [AOF Documentation](../docs/README.md) - Complete framework docs
- [YAML Schema](../docs/schema.md) - API reference
- [Best Practices](../docs/best-practices.md) - Production tips

---

## 💡 Pro Tips

1. **Start simple** - Begin with hello-world, then gradually add complexity
2. **Use dry-run** - Test configurations with `--dry-run` before applying
3. **Monitor tokens** - Track usage with `aof stats` to control costs
4. **Version control** - Store your YAML configs in git
5. **Environment-specific** - Use separate configs for dev/staging/prod
6. **Test locally** - Validate agents with `aof query` before deploying flows
7. **Add guardrails** - Always include safety checks for production agents
8. **Log everything** - Enable detailed logging for debugging
9. **Start with read-only** - Test with read-only tools before allowing writes
10. **Iterate quickly** - Make small changes and test frequently

---

**Ready to get started?** Pick an example and `aof apply -f` it! 🚀
