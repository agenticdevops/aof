# AOF Examples Library

Welcome to the AOF (Agentic Ops Framework) examples library! This collection demonstrates how to build production-ready AI agents for DevOps and SRE workflows using Kubernetes-style YAML.

## 📚 Table of Contents

- [Quick Start](#quick-start)
- [Single Agents](#single-agents)
- [Agent Fleets](#agent-fleets)
- [Agent Flows](#agent-flows)
- [Prerequisites](#prerequisites)
- [Usage Guide](#usage-guide)

---

## 🚀 Quick Start

Perfect for learning the basics:

### [`quickstart/hello-world-agent.yaml`](quickstart/hello-world-agent.yaml)
**The simplest possible agent** - great for testing your setup.
```bash
aof apply -f quickstart/hello-world-agent.yaml
aof query hello "What's your name?"
```

### [`quickstart/hello-world-flow.yaml`](quickstart/hello-world-flow.yaml)
**The simplest possible flow** - webhook → agent → response.
```bash
aof apply -f quickstart/hello-world-flow.yaml
curl -X POST http://localhost:8080/webhook/hello-flow \
  -d '{"message": "Hello AOF!"}'
```

---

## 🤖 Single Agents

Individual AI agents for specific tasks:

### 1. [Kubernetes Operations Agent](agents/kubernetes-ops-agent.yaml)
**Kubernetes diagnostics and management**

**What it does:**
- Diagnose pod failures and crashes
- Scale deployments up/down
- Check resource usage (CPU, memory)
- Analyze cluster health

**Example usage:**
```bash
aof query k8s-ops "What pods are failing in the default namespace?"
aof query k8s-ops "Scale the nginx deployment to 5 replicas"
```

**Prerequisites:** `kubectl` configured with cluster access

---

### 2. [GitHub Assistant Agent](agents/github-assistant-agent.yaml)
**GitHub PR and issue management**

**What it does:**
- Summarize pull requests
- Review code changes
- Triage issues
- Suggest reviewers

**Example usage:**
```bash
aof query github-helper "Summarize PR #42 in kubernetes/kubernetes"
aof query github-helper "What open issues are labeled 'good-first-issue'?"
```

**Prerequisites:** `GITHUB_TOKEN` environment variable

---

### 3. [Dockerfile Generator Agent](agents/dockerfile-generator-agent.yaml)
**Automated Dockerfile creation**

**What it does:**
- Analyze repository structure
- Detect language/framework
- Generate optimized multi-stage Dockerfiles
- Suggest .dockerignore patterns

**Example usage:**
```bash
aof query dockerfile-gen "Generate a Dockerfile for /workspace/myapp"
```

**Prerequisites:** Access to repository directory

---

### 4. [Terraform Planner Agent](agents/terraform-planner-agent.yaml)
**Infrastructure code review and security**

**What it does:**
- Analyze Terraform plans for security risks
- Identify cost optimization opportunities
- Check compliance (SOC2, GDPR)
- Suggest best practices

**Example usage:**
```bash
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json
aof query tf-planner "Review the plan in /workspace/plan.json"
```

**Prerequisites:** Terraform installed, cloud credentials configured

---

### 5. [Log Analyzer Agent](agents/log-analyzer-agent.yaml)
**Intelligent log parsing and analysis**

**What it does:**
- Parse and analyze log files
- Identify error patterns
- Detect anomalies
- Provide actionable insights

**Example usage:**
```bash
aof query log-analyzer "Analyze /var/log/app/error.log for the last hour"
aof query log-analyzer "Find all 500 errors in nginx access logs"
```

**Prerequisites:** Access to log files

---

### 6. [Security Scanner Agent](agents/security-scanner-agent.yaml)
**Comprehensive security scanning**

**What it does:**
- Scan containers for vulnerabilities (using trivy)
- Analyze code for security issues (using semgrep)
- Check dependency CVEs
- Assess infrastructure security

**Example usage:**
```bash
aof query sec-scanner "Scan the Docker image myapp:latest"
aof query sec-scanner "Check /workspace/app for security issues"
```

**Prerequisites:** Security tools installed (trivy, semgrep, etc.)

---

## 👥 Agent Fleets

Teams of agents working together:

### 7. [Code Review Fleet](fleets/code-review-fleet.yaml)
**Multi-perspective code review team**

**Agents:**
- **Security Reviewer**: Finds vulnerabilities and security issues
- **Performance Reviewer**: Identifies performance bottlenecks
- **Quality Reviewer**: Checks code quality and maintainability

**Strategy:** Parallel execution with merged results

**Example usage:**
```bash
aof query code-review-team "Review PR #123 in myorg/myrepo"
aof query code-review-team "Analyze changes in /workspace/app/src"
```

**Output:** Comprehensive review combining security, performance, and quality insights.

---

### 8. [SRE On-Call Fleet](fleets/sre-oncall-fleet.yaml)
**Automated incident response team**

**Agents:**
1. **Diagnostic Agent**: Identifies root cause
2. **Remediation Agent**: Executes fixes
3. **Communication Agent**: Updates stakeholders

**Strategy:** Sequential execution (diagnose → remediate → communicate)

**Example usage:**
```bash
aof query sre-oncall "Pod myapp-7d8f9c is CrashLoopBackOff in production"
aof query sre-oncall "High CPU usage on node ip-10-0-1-42"
```

**Output:** Diagnosis, automated fix, and status updates posted to Slack.

---

## 🔄 Agent Flows

End-to-end automated workflows:

### 9. [Slack Q&A Bot Flow](flows/slack-qa-bot-flow.yaml)
**Interactive Slack assistant**

**Flow:**
```
Slack mention → Q&A Agent → Reply in thread
```

**What it does:**
- Answer questions about codebase and infrastructure
- Provide deployment instructions
- Help with debugging

**Setup:**
```bash
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f slack-qa-bot-flow.yaml
# In Slack: @qa-bot How do I deploy to production?
```

**Prerequisites:** Slack bot token, bot invited to channels

---

### 10. [PR Review Flow](flows/pr-review-flow.yaml)
**Automated pull request review**

**Flow:**
```
GitHub PR opened → Code Review Fleet → Post review comment
```

**What it does:**
- Parallel security and quality checks
- Post comprehensive review
- Add labels based on findings
- Auto-approve safe changes

**Setup:**
```bash
export GITHUB_TOKEN=ghp_xxxxx
aof apply -f pr-review-flow.yaml
# Opens automatically on PRs
```

**Output:** PR comments with detailed feedback and auto-labels.

---

### 11. [Incident Auto-Remediation Flow](flows/incident-auto-remediation-flow.yaml)
**Full incident response pipeline**

**Flow:**
```
PagerDuty alert → SRE Fleet (diagnose) → Decision tree → Auto-fix or escalate
```

**Decision Logic:**
- **P2/P3 + High confidence**: Auto-remediate
- **P1 + Medium confidence**: Request approval
- **P0 or Low confidence**: Escalate to human

**Setup:**
```bash
export PAGERDUTY_TOKEN=xxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f incident-auto-remediation-flow.yaml
```

**Output:** Automated diagnosis, fix attempt, and full audit trail.

---

### 12. [Daily Standup Report Flow](flows/daily-standup-report-flow.yaml)
**Automated team status report**

**Flow:**
```
Cron (9am weekdays) → Multiple analyzers (parallel) → Slack summary
```

**Analyzers:**
- GitHub activity (PRs, commits)
- Jira tickets (completed, in progress, blocked)
- CI/CD pipelines (deployments, test results)
- Team calendar (PTO, meetings)
- System metrics (errors, performance)

**Setup:**
```bash
export GITHUB_TOKEN=ghp_xxxxx
export JIRA_TOKEN=xxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f daily-standup-report-flow.yaml
```

**Output:** Comprehensive daily standup posted to #daily-standup at 9am.

---

### 13. [Deploy Notification Flow](flows/deploy-notification-flow.yaml)
**Deployment tracking and communication**

**Flow:**
```
GitHub deployment event → Analyze changes → Slack notification + metrics
```

**What it does:**
- Summarize what's being deployed
- Identify breaking changes
- Provide rollback instructions
- Update status page and metrics

**Setup:**
```bash
export GITHUB_TOKEN=ghp_xxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f deploy-notification-flow.yaml
```

**Output:** Real-time deployment notifications with rollback commands.

---

### 14. [Cost Optimization Flow](flows/cost-optimization-flow.yaml)
**Daily cloud cost analysis**

**Flow:**
```
Cron (8am daily) → Cost analyzer → Anomaly detection → Alerts/optimizations
```

**What it does:**
- Detect unusual spending spikes
- Identify unused resources
- Recommend reserved instances
- Forecast monthly costs
- Auto-remediate safe optimizations

**Severity Levels:**
- 🚨 **Critical**: >50% increase or >1.5x budget
- ⚠️ **Notable**: >20% increase or >1.2x budget
- 💡 **Optimization**: $1000+/month savings available

**Setup:**
```bash
export AWS_ACCESS_KEY_ID=xxxxx
export AWS_SECRET_ACCESS_KEY=xxxxx
export SLACK_BOT_TOKEN=xoxb-xxxxx
aof apply -f cost-optimization-flow.yaml
```

**Output:** Daily cost reports, anomaly alerts, and optimization recommendations.

---

## 📋 Prerequisites

### Required for all examples:
```bash
export ANTHROPIC_API_KEY=sk-ant-xxxxx
```

### Optional (depending on example):
```bash
# GitHub integration
export GITHUB_TOKEN=ghp_xxxxx

# Slack integration
export SLACK_BOT_TOKEN=xoxb-xxxxx

# AWS (for K8s/cost examples)
export AWS_ACCESS_KEY_ID=xxxxx
export AWS_SECRET_ACCESS_KEY=xxxxx

# PagerDuty (for incident flow)
export PAGERDUTY_TOKEN=xxxxx

# Jira (for standup flow)
export JIRA_TOKEN=xxxxx
```

### Tools (install as needed):
```bash
# Kubernetes
kubectl

# Docker
docker

# Terraform
terraform

# Security scanning
trivy
semgrep

# Cloud CLIs
aws-cli
gcloud
azure-cli
```

---

## 🎯 Usage Guide

### Basic Commands

**Apply a configuration:**
```bash
aof apply -f examples/agents/kubernetes-ops-agent.yaml
```

**Query an agent:**
```bash
aof query <agent-name> "<your question>"
```

**List active agents/flows:**
```bash
aof list agents
aof list flows
```

**View agent logs:**
```bash
aof logs <agent-name>
```

**Delete an agent/flow:**
```bash
aof delete agent <agent-name>
aof delete flow <flow-name>
```

---

## 📊 Example Selection Guide

**Choose based on your use case:**

| Use Case | Example | Complexity |
|----------|---------|------------|
| Learning AOF basics | Hello World Agent/Flow | ⭐ Beginner |
| K8s troubleshooting | Kubernetes Ops Agent | ⭐⭐ Intermediate |
| Code reviews | Code Review Fleet + PR Review Flow | ⭐⭐⭐ Advanced |
| Incident response | SRE On-Call Fleet + Auto-Remediation Flow | ⭐⭐⭐ Advanced |
| Team automation | Daily Standup Report Flow | ⭐⭐ Intermediate |
| Cost management | Cost Optimization Flow | ⭐⭐⭐ Advanced |
| Security scanning | Security Scanner Agent | ⭐⭐ Intermediate |
| Infrastructure review | Terraform Planner Agent | ⭐⭐ Intermediate |

---

## 🛠️ Customization Tips

### 1. Adjust Model Selection
```yaml
spec:
  model: claude-3-5-sonnet-20241022  # Fast and cost-effective
  # Or use:
  # model: claude-3-opus-20240229    # More powerful but slower
  # model: claude-3-haiku-20240307   # Fastest and cheapest
```

### 2. Modify System Prompts
Add your company-specific context:
```yaml
spec:
  system: |
    You are an SRE at ACME Corp.

    Our stack:
    - Kubernetes on AWS EKS
    - PostgreSQL on RDS
    - Redis for caching
    - Datadog for monitoring

    [rest of prompt...]
```

### 3. Add Custom Tools
```yaml
spec:
  tools:
    - name: shell
      config:
        allowed_commands:
          - your-custom-cli
          - company-tool
```

### 4. Adjust Resource Limits
```yaml
spec:
  resources:
    max_tokens: 8192    # Increase for detailed analysis
    timeout: 300s       # Increase for long-running tasks
```

### 5. Enable Guardrails
```yaml
spec:
  guardrails:
    - type: command_filter
      config:
        blocked_patterns:
          - "rm -rf /"
          - "kubectl delete ns production"
```

---

## 🤝 Contributing

Have a great example? Submit a PR!

**Guidelines:**
1. Use descriptive names and comments
2. Include prerequisites and usage instructions
3. Provide realistic example queries
4. Follow the existing YAML schema
5. Test before submitting

---

## 📚 Additional Resources

- [AOF Documentation](../docs/README.md)
- [YAML Schema Reference](../docs/schema.md)
- [Best Practices Guide](../docs/best-practices.md)
- [Troubleshooting](../docs/troubleshooting.md)

---

## 📄 License

All examples are provided under the MIT License. Feel free to use and modify for your organization.

---

**Happy automating!** 🚀

If you have questions or need help, open an issue or reach out to the community.
