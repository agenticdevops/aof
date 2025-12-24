# AOF Agent Library

Production-ready agents for DevOps and SRE operations.

## Quick Start

```bash
# List available agents
aofctl get agents --library

# Run an agent
aofctl run agent library://kubernetes/pod-doctor

# Run with custom prompt
aofctl run agent library://kubernetes/pod-doctor \
  --prompt "Debug CrashLoopBackOff in namespace production"
```

## Domains

| Domain | Agents | Description |
|--------|--------|-------------|
| [kubernetes/](./kubernetes/) | 5 | Pod debugging, HPA tuning, NetworkPolicies |
| [observability/](./observability/) | 5 | Alerting, SLOs, Dashboards, Logs, Traces |
| [incident/](./incident/) | 5 | RCA, Postmortems, Runbooks, Escalation |
| [cicd/](./cicd/) | 5 | Pipelines, Tests, Builds, Releases |
| [security/](./security/) | 5 | Scanning, Compliance, Secrets, Threats |
| [cloud/](./cloud/) | 5 | Cost, IAM, Right-sizing, Drift |

## Agent Categories

### Kubernetes (5 agents)
- **pod-doctor** - Diagnose pod issues (CrashLoopBackOff, OOMKilled)
- **hpa-tuner** - Optimize autoscaler configurations
- **netpol-debugger** - Debug NetworkPolicy connectivity
- **yaml-linter** - Validate K8s manifests
- **resource-optimizer** - Right-size resource requests/limits

### Observability (5 agents)
- **alert-manager** - Optimize alerting rules
- **slo-guardian** - Monitor SLO compliance
- **dashboard-generator** - Auto-generate dashboards
- **log-analyzer** - Analyze logs for patterns
- **trace-investigator** - Investigate distributed traces

### Incident (5 agents)
- **incident-commander** - Coordinate incident response
- **rca-agent** - Root cause analysis
- **postmortem-writer** - Generate postmortems
- **runbook-executor** - Execute runbook procedures
- **escalation-manager** - Manage escalation chains

### CI/CD (5 agents)
- **pipeline-doctor** - Diagnose pipeline failures
- **test-analyzer** - Analyze test results/coverage
- **build-optimizer** - Optimize build times
- **release-manager** - Coordinate releases
- **deploy-guardian** - Safe deployment validation

### Security (5 agents)
- **security-scanner** - Vulnerability scanning
- **compliance-auditor** - Compliance checking
- **secret-rotator** - Secret management
- **vulnerability-patcher** - Patch vulnerabilities
- **threat-hunter** - Proactive threat detection

### Cloud (5 agents)
- **cost-optimizer** - Cloud cost optimization
- **iam-auditor** - IAM permission auditing
- **resource-rightsizer** - Right-size cloud resources
- **cloud-migrator** - Migration planning
- **drift-detector** - Infrastructure drift detection

## Agent Specification

All library agents follow the AOF Agent Specification:

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: agent-name
  labels:
    category: domain
    domain: subdomain
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2
  tools:
    - tool1
    - tool2
  system_prompt: |
    Expert system prompt...
```

## Environment Variables

Agents require appropriate credentials:

```bash
# Kubernetes
export KUBECONFIG=~/.kube/config

# Observability
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_TOKEN=your-token
export PROMETHEUS_URL=https://prometheus.example.com

# CI/CD
export GITHUB_TOKEN=ghp_xxxx
export GITLAB_TOKEN=glpat-xxxx

# Security
export TRIVY_CACHE_DIR=~/.cache/trivy

# Cloud
export AWS_PROFILE=production
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/creds.json
```

## Documentation

Full documentation at [docs.aof.sh/agent-library](https://docs.aof.sh/agent-library)

## Contributing

1. Follow the [Agent Specification](https://docs.aof.sh/reference/agent-spec)
2. Include comprehensive system prompts
3. Document required tools and environment variables
4. Add examples and use cases
5. Submit PR to the [AOF repository](https://github.com/agenticdevops/aof)

## License

Apache 2.0
