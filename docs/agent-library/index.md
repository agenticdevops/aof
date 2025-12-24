---
sidebar_position: 1
sidebar_label: Overview
---

# Agent Library

AOF includes a production-ready library of 30+ pre-built agents organized by operational domain. These agents are designed by DevOps and SRE experts to solve common operational challenges.

## Quick Start

```bash
# List available agents
aofctl get agents --library

# Run an agent from the library
aofctl run agent library://kubernetes/pod-doctor

# Run with custom prompt
aofctl run agent library://kubernetes/pod-doctor \
  --prompt "Debug CrashLoopBackOff in namespace production"
```

## Agent Domains

| Domain | Agents | Focus Area |
|--------|--------|------------|
| [Kubernetes](./kubernetes.md) | 5 | Pod debugging, HPA tuning, Network policies |
| [Observability](./observability.md) | 5 | Alerts, SLOs, Dashboards, Logs, Traces |
| [Incident](./incident.md) | 5 | RCA, Postmortems, Runbooks, Escalation |
| [CI/CD](./cicd.md) | 5 | Pipelines, Tests, Builds, Releases |
| [Security](./security.md) | 5 | Scanning, Compliance, Secrets, Threats |
| [Cloud](./cloud.md) | 5 | Cost, IAM, Right-sizing, Drift |

## Using Library Agents

### Direct Execution

```bash
# Run with default prompt
aofctl run agent library://observability/log-analyzer

# Run with custom prompt
aofctl run agent library://incident/rca-agent \
  --prompt "Investigate the API latency spike at 14:30 UTC"

# Run interactively
aofctl run agent library://kubernetes/pod-doctor --interactive
```

### Extend with Custom Agents

Copy a library agent as a starting point:

```bash
# Copy agent to local file
aofctl get agent library://kubernetes/pod-doctor -o yaml > my-pod-doctor.yaml

# Customize and run
aofctl run agent -f my-pod-doctor.yaml
```

### Use in Fleets

Reference library agents in fleet definitions:

```yaml
apiVersion: aof.sh/v1alpha1
kind: Fleet
metadata:
  name: incident-response
spec:
  agents:
    - ref: library://incident/incident-commander
    - ref: library://incident/rca-agent
    - ref: library://observability/log-analyzer
  strategy:
    type: sequential
```

## Agent Configuration

All library agents use a consistent configuration:

- **Model**: `google:gemini-2.5-flash` (optimized for DevOps tasks)
- **Max Tokens**: 8192
- **Temperature**: 0.1-0.3 (low for operational reliability)

Override defaults in your agent definition:

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: my-custom-agent
spec:
  # Inherit from library agent
  extends: library://kubernetes/pod-doctor

  # Override settings
  model: anthropic:claude-3-sonnet
  temperature: 0.1
```

## Environment Variables

Library agents may require environment variables for tool access:

```bash
# Kubernetes (uses kubeconfig)
export KUBECONFIG=~/.kube/config

# Observability
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_TOKEN=your-token
export PROMETHEUS_URL=https://prometheus.example.com

# CI/CD
export GITHUB_TOKEN=ghp_xxxx
export GITLAB_TOKEN=glpat-xxxx

# Cloud
export AWS_PROFILE=production
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json
```

## Best Practices

1. **Start with library agents** - They encode expert knowledge
2. **Customize prompts** - Tailor to your specific environment
3. **Extend, don't rewrite** - Use `extends` for modifications
4. **Combine in fleets** - Orchestrate multiple agents for complex tasks
5. **Monitor execution** - Review agent decisions in audit logs

## Contributing Agents

Submit new agents to the library:

1. Follow the [Agent Specification](../reference/agent-spec.md)
2. Include comprehensive system prompts
3. Document required tools and environment variables
4. Add examples and use cases
5. Submit PR to the [AOF repository](https://github.com/agenticdevops/aof)

## Next Steps

- [Kubernetes Agents](./kubernetes.md) - Pod debugging, resource optimization
- [Observability Agents](./observability.md) - Alerting, SLOs, log analysis
- [First Agent Tutorial](../tutorials/first-agent.md) - Build your own agent
