# Agent Definitions

This directory contains **single source of truth** agent definitions. Each agent is defined once and referenced elsewhere using `ref:` syntax.

## Available Agents

### Core Infrastructure Agents

- **`k8s-ops.yaml`** - Kubernetes operations expert
  - Cluster management and troubleshooting
  - Pod/Deployment/Service diagnostics
  - Resource management and scaling
  - Helm chart deployments
  - **Tools**: kubectl, helm

- **`devops.yaml`** - Full-stack DevOps operations
  - Kubernetes + Docker + CI/CD
  - Infrastructure as Code (Terraform)
  - Cloud platform operations
  - Monitoring setup
  - **Tools**: kubectl, docker, helm, terraform, git, shell

### Security & Compliance

- **`security.yaml`** - Multi-layer security scanner
  - Container vulnerability scanning (trivy)
  - Code security analysis (semgrep)
  - Dependency checks
  - Infrastructure misconfigurations
  - Compliance checks (CIS, PCI DSS, SOC2)
  - **Tools**: trivy, semgrep, kubectl

### Operations & Incident Response

- **`incident.yaml`** - Incident response coordinator
  - Incident detection and classification
  - Root cause analysis (RCA)
  - Log and metric analysis
  - Runbook execution
  - Post-incident reports
  - **Tools**: kubectl, prometheus, loki, shell

### General Purpose

- **`general-assistant.yaml`** - Versatile helper
  - Answer general questions
  - Provide documentation
  - Explain concepts
  - Route to specialized agents
  - **Tools**: None (knowledge-based)

## Usage Patterns

### 1. Direct Execution
\`\`\`bash
# Run agent directly with aofctl
aofctl run agent k8s-ops "check pod status in default namespace"
aofctl run agent security "scan docker image nginx:latest"
aofctl run agent incident "investigate high latency in prod"
\`\`\`

### 2. Reference in Fleets
\`\`\`yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: my-fleet
spec:
  agents:
    - ref: agents/k8s-ops.yaml
    - ref: agents/security.yaml
\`\`\`

### 3. Reference in Flows
\`\`\`yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: my-flow
spec:
  nodes:
    - id: k8s-check
      type: Agent
      config:
        agent: k8s-ops  # References metadata.name
\`\`\`

## Best Practices

1. **Never duplicate agents** - Always reference the canonical version
2. **Use descriptive metadata.name** - Short, kebab-case identifiers
3. **Document tools** - List all tools the agent can use
4. **Implement safety** - Always warn before destructive operations
5. **Keep focused** - One agent, one primary responsibility

## Related Directories

- `/fleets` - Compose agents into teams
- `/flows` - Orchestrate agent workflows
- `/contexts` - Environment-specific configurations
- `/triggers` - Message sources (Slack, Telegram, etc.)
- `/bindings` - Tie everything together
