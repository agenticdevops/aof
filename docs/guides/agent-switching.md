# Agent Switching Guide

Switch between different agents via the `/agent` command.

## Quick Start

```
/agent              # Show available agents (inline buttons)
/agent k8s          # Switch to Kubernetes agent
/agent info         # Show current agent details
```

## Available Agents

| Command | Agent | Tools |
|---------|-------|-------|
| `/agent k8s` | k8s-ops | kubectl, helm |
| `/agent aws` | aws-agent | aws |
| `/agent docker` | docker-ops | docker, shell |
| `/agent devops` | devops | kubectl, docker, helm, terraform, git, shell |

## User Experience

```
User: /agent

Bot: Select Agent
     Current: Kubernetes

     [Kubernetes] [AWS]
     [Docker]     [DevOps]

User: *taps AWS*

Bot: Switched to AWS

     Tools: aws

     AWS cloud operations.

User: list ec2 instances

Bot: [AWS CLI output...]
```

## Adding Custom Agents

Create agent YAML files in your agents directory:

```yaml
# agents/my-agent.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: my-agent
spec:
  model: google:gemini-2.5-flash
  tools:
    - kubectl
    - docker
  system_prompt: |
    You are a helpful DevOps assistant.
```

Then start the server with your agents:

```bash
aofctl serve --config config.yaml --agents-dir ./agents
```

## Platform Behavior

| Platform | Write Access |
|----------|--------------|
| CLI | Full access |
| Slack | Full access (with approval workflow) |
| Telegram | Read-only |
| WhatsApp | Read-only |

See [Safety Layer Guide](safety-layer.md) for details.
