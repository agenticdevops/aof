# AOF Examples

Copy-paste ready YAML configurations for common use cases.

## Quick Start Examples

### 1. Kubernetes Operations Agent
**File:** `kubernetes-agent.yaml`
**Use Case:** Interactive K8s cluster management and troubleshooting

**Features:**
- Safe kubectl command execution
- MCP server integration
- Pod/deployment diagnostics
- Service health checks

**Quick Start:**
```bash
# Set your API key
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...

# Apply and run
aofctl apply -f kubernetes-agent.yaml
aofctl run agent kubernetes-agent
```

**Try it:**
```
> Show me all failing pods
> Why is my nginx deployment stuck?
> Scale the api deployment to 5 replicas
```

---

### 2. GitHub PR Review Agent
**File:** `github-pr-reviewer.yaml`
**Use Case:** Automated code review for pull requests

**Features:**
- Security vulnerability detection
- Performance analysis
- Code quality checks
- Best practices enforcement
- Automated PR comments

**Quick Start:**
```bash
# Set GitHub token
export GITHUB_TOKEN=ghp_...

# Apply agent
aofctl apply -f github-pr-reviewer.yaml

# Manual review
aofctl exec agent github-pr-reviewer -- "Review PR #123 in myorg/myrepo"

# Or apply the flow for automation
aofctl apply -f github-pr-reviewer.yaml
aofctl run agentflow auto-pr-review --daemon
```

---

### 3. Incident Response System
**File:** `incident-responder.yaml`
**Use Case:** Auto-remediation of production incidents

**Features:**
- PagerDuty integration
- Intelligent diagnostics
- Auto-remediation with approval
- Slack notifications
- Incident tracking

**Quick Start:**
```bash
# Set credentials
export PAGERDUTY_WEBHOOK_TOKEN=...
export PAGERDUTY_API_KEY=...
export SLACK_BOT_TOKEN=xoxb-...

# Apply agents and flow
aofctl apply -f incident-responder.yaml
aofctl apply -f incident-responder.yaml

# Start the flow
aofctl run agentflow incident-auto-response --daemon
```

---

### 4. Slack Bot with Interactive Features
**File:** `slack-bot-flow.yaml`
**Use Case:** Conversational K8s assistant in Slack

**Features:**
- @mention and DM support
- Slash commands
- Human-in-the-loop approvals
- Interactive buttons
- Daily reports

**Quick Start:**
```bash
# Set Slack credentials
export SLACK_BOT_TOKEN=xoxb-...
export SLACK_SIGNING_SECRET=...
export SLACK_BOT_USER_ID=U...

# Apply and run
aofctl apply -f slack-bot-flow.yaml
aofctl apply -f slack-bot-flow.yaml
aofctl run agentflow slack-k8s-bot --daemon

# Test in Slack
# @k8s-assistant show me all pods
```

---

### 5. Daily/Weekly Reports
**File:** `daily-report-flow.yaml`
**Use Case:** Automated operational reports

**Features:**
- Daily cluster health reports
- Weekly summaries
- Resource usage analysis
- Incident statistics
- Custom on-demand reports

**Quick Start:**
```bash
# Apply and run
aofctl apply -f daily-report-flow.yaml
aofctl apply -f daily-report-flow.yaml

# Start scheduled flows
aofctl run agentflow daily-cluster-report --daemon
aofctl run agentflow weekly-summary-report --daemon

# Custom report via Slack
# /report health 24h production
```

---

## Example Comparison

| Example | Complexity | Best For | Prerequisites |
|---------|------------|----------|---------------|
| **kubernetes-agent** | ⭐ Simple | Learning AOF | kubectl, API key |
| **github-pr-reviewer** | ⭐⭐ Medium | Code reviews | GitHub token |
| **incident-responder** | ⭐⭐⭐ Advanced | Production ops | PagerDuty, Slack |
| **slack-bot-flow** | ⭐⭐ Medium | Team automation | Slack app |
| **daily-report-flow** | ⭐⭐ Medium | Operations reporting | Slack (optional) |

---

## Customization Tips

### Change the Model

```yaml
spec:
  model: openai:gpt-4              # Original

  # Alternatives:
  model: anthropic:claude-3-5-sonnet-20241022  # Claude Sonnet
  model: openai:gpt-3.5-turbo      # Cheaper/faster
  model: ollama:llama3             # Local (free)
```

### Add More Tools

```yaml
tools:
  # Add filesystem access
  - type: FileSystem
    config:
      allowed_paths: [/etc/kubernetes]

  # Add custom HTTP endpoints
  - type: HTTP
    config:
      base_url: https://api.company.com
      headers:
        Authorization: "Bearer ${API_TOKEN}"
```

### Adjust Memory

```yaml
memory:
  type: InMemory              # Development (default)

  # OR production:
  type: PostgreSQL
  config:
    url: postgres://user:pass@localhost/aof
```

---

## Environment Variables

Common variables used across examples:

```bash
# LLM Providers
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...

# Kubernetes
export KUBECONFIG=~/.kube/config

# GitHub
export GITHUB_TOKEN=ghp_...

# Slack
export SLACK_BOT_TOKEN=xoxb-...
export SLACK_SIGNING_SECRET=...
export SLACK_BOT_USER_ID=U...

# PagerDuty
export PAGERDUTY_API_KEY=...
export PAGERDUTY_WEBHOOK_TOKEN=...

# Custom APIs
export API_TOKEN=...
```

Add to your `~/.zshrc` or `~/.bashrc`:
```bash
# Source AOF environment
source ~/.aof/env
```

---

## Combining Examples

Mix and match for powerful workflows:

### Example: PR Review + Slack Notifications

```yaml
# Use GitHub PR reviewer with Slack notifications
nodes:
  - id: review
    type: Agent
    config:
      agent: github-pr-reviewer

  - id: notify-team
    type: Slack
    config:
      channel: "#code-reviews"
      message: ${review.output}
```

### Example: Incident Response + Daily Reports

```yaml
# Include incident stats in daily reports
nodes:
  - id: fetch-incidents
    type: HTTP
    config:
      url: https://api.company.com/incidents/daily

  - id: generate-report
    type: Agent
    config:
      agent: report-generator
      input: |
        Include incident summary: ${fetch-incidents.output}
```

---

## Testing Examples

### Validate YAML
```bash
aofctl apply -f kubernetes-agent.yaml --dry-run
```

### Dry Run Flow
```bash
aofctl run agentflow my-flow --dry-run
```

### Test Agent Locally
```bash
aofctl run agent kubernetes-agent.yaml --input "test query"
```

---

## Getting Help

- **Tutorials**: See [Tutorials](/docs/tutorials/first-agent)
- **Reference**: See [Reference](/docs/reference/agent-spec)
- **Issues**: [GitHub Issues](https://github.com/gshah/my-framework/issues)

---

## Contributing Examples

Have a useful agent configuration? Submit it!

1. Create your YAML file
2. Add inline documentation
3. Test it thoroughly
4. Submit a PR with:
   - YAML file
   - Description
   - Setup instructions
   - Example usage

---

**Ready to build?** Start with `kubernetes-agent.yaml` and customize from there!
