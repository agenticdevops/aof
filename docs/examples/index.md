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
aofctl agent apply -f kubernetes-agent.yaml
aofctl agent chat kubernetes-agent
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
aofctl agent apply -f github-pr-reviewer.yaml

# Manual review
aofctl agent exec github-pr-reviewer "Review PR #123 in myorg/myrepo"

# Or apply the flow for automation
aofctl flow apply -f github-pr-reviewer.yaml
aofctl flow run auto-pr-review --daemon
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
aofctl agent apply -f incident-responder.yaml
aofctl flow apply -f incident-responder.yaml

# Start the flow
aofctl flow run incident-auto-response --daemon
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
aofctl agent apply -f slack-bot-flow.yaml
aofctl flow apply -f slack-bot-flow.yaml
aofctl flow run slack-k8s-bot --daemon

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
aofctl agent apply -f daily-report-flow.yaml
aofctl flow apply -f daily-report-flow.yaml

# Start scheduled flows
aofctl flow run daily-cluster-report --daemon
aofctl flow run weekly-summary-report --daemon

# Custom report via Slack
# /report health 24h production
```

---

## AgentFleet Examples

AgentFleet enables multi-agent coordination for complex tasks. Here are production-ready fleet configurations:

### 6. Kubernetes RCA Team
**File:** `examples/fleets/k8s-rca-team.yaml`
**Use Case:** Root cause analysis for failing Kubernetes pods

**Agents:**
- **pod-analyzer** - Analyzes pod states and events
- **log-investigator** - Examines container logs for errors
- **resource-analyst** - Checks resources and Prometheus metrics
- **rca-synthesizer** - Combines findings into actionable report

**Features:**
- kubectl-ai MCP integration for K8s operations
- Parallel agent execution
- Consensus-based result aggregation
- Structured RCA reports

**Quick Start:**
```bash
aofctl run fleet examples/fleets/k8s-rca-team.yaml \
  --input '{"namespace": "production", "issue": "Pods failing with CrashLoopBackOff"}'
```

---

### 7. Dockerizer Team
**File:** `examples/fleets/dockerizer-team.yaml`
**Use Case:** Containerize applications with best practices

**Agents (Pipeline):**
1. **app-analyzer** - Analyzes app structure, dependencies, runtime
2. **dockerfile-writer** - Generates optimized multi-stage Dockerfiles
3. **security-scanner** - Scans for vulnerabilities (8/10 average score)
4. **dockerfile-reviewer** - Reviews quality and best practices
5. **compose-writer** - Generates docker-compose for dev/prod

**Features:**
- Pipeline coordination (sequential stages)
- Security vulnerability scanning
- Multi-stage Docker builds
- Development and production compose files

**Quick Start:**
```bash
aofctl run fleet examples/fleets/dockerizer-team.yaml \
  --input '{"app_name": "my-api", "language": "nodejs", "ports": [3000]}'
```

**Output includes:**
- Optimized Dockerfile with security hardening
- .dockerignore file
- docker-compose.yml (development)
- docker-compose.prod.yml (production)
- .env.example

---

### 8. Code Review Team
**File:** `examples/fleets/code-review-team.yaml`
**Use Case:** Comprehensive code review with multiple perspectives

**Agents:**
- **security-reviewer** - Security vulnerabilities, auth issues
- **performance-reviewer** - Big O, memory, query optimization
- **quality-reviewer** - SOLID principles, maintainability

**Features:**
- Peer coordination with consensus voting
- Parallel execution for speed
- Multiple review perspectives

**Quick Start:**
```bash
aofctl run fleet examples/fleets/code-review-team.yaml \
  --input '{"code": "def add(a, b): return a + b", "language": "python"}'
```

---

### Fleet Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **peer** | Agents work as equals with consensus | Code review, analysis |
| **pipeline** | Sequential stages | ETL, containerization |
| **hierarchical** | Manager coordinates workers | Complex orchestration |
| **swarm** | Emergent behavior | Exploratory tasks |

---

## Example Comparison

| Example | Complexity | Best For | Prerequisites |
|---------|------------|----------|---------------|
| **kubernetes-agent** | ⭐ Simple | Learning AOF | kubectl, API key |
| **github-pr-reviewer** | ⭐⭐ Medium | Code reviews | GitHub token |
| **incident-responder** | ⭐⭐⭐ Advanced | Production ops | PagerDuty, Slack |
| **slack-bot-flow** | ⭐⭐ Medium | Team automation | Slack app |
| **daily-report-flow** | ⭐⭐ Medium | Operations reporting | Slack (optional) |
| **k8s-rca-team** | ⭐⭐⭐ Advanced | K8s troubleshooting | kubectl-ai MCP |
| **dockerizer-team** | ⭐⭐⭐ Advanced | Containerization | Docker |
| **code-review-team** | ⭐⭐ Medium | Code reviews | API key |

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
aofctl agent validate -f kubernetes-agent.yaml
```

### Dry Run Flow
```bash
aofctl flow run my-flow --dry-run
```

### Test Agent Locally
```bash
aofctl agent run kubernetes-agent.yaml --input "test query"
```

---

## Getting Help

- **Tutorials**: See [First Agent Tutorial](../tutorials/first-agent)
- **Reference**: See [aofctl CLI Reference](../reference/aofctl)
- **Issues**: [GitHub Issues](https://github.com/yourusername/aof/issues)

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
