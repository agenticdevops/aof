---
sidebar_position: 1
title: Agentic Skills Overview
description: Codify tribal knowledge as executable agent capabilities
---

# Agentic Skills

**Skills** are the secret weapon that transforms your AI agents from generic assistants into domain experts. They codify tribal knowledge—the hard-won experience of your senior engineers—into executable, shareable, versioned capabilities that any agent can discover and invoke.

## What Are Skills?

Skills are `SKILL.md` files with YAML frontmatter containing metadata and markdown content with instructions. They provide:

- **Domain Expertise**: Specific knowledge about tools, systems, and procedures
- **Executable Instructions**: Step-by-step guidance agents can follow
- **Requirements Gating**: Automatic detection of prerequisites (CLIs, configs, env vars)
- **Hot-Reload**: Changes take effect immediately without restart

## Why Skills Matter

### Without Skills
Your agent knows how to use `kubectl`, but doesn't know your team's specific debugging workflow for CrashLoopBackOff issues.

### With Skills
Your agent has the same debugging expertise as your most senior SRE—knowing exactly which commands to run, what to check first, and how to interpret the results.

## Quick Example

```markdown
---
name: k8s-debug
description: "Kubernetes pod debugging and troubleshooting"
metadata:
  emoji: "🐳"
  requires:
    bins: ["kubectl"]
    config: ["~/.kube/config"]
  tags: ["kubernetes", "debugging"]
---

# Kubernetes Debug Skill

## When to Use
- Pod in CrashLoopBackOff
- Application logs show errors
- Services not reachable

## Quick Diagnostics
```bash
kubectl get pods -o wide
kubectl describe pod <pod-name>
kubectl logs <pod-name> --previous
```

## Common Issues

### CrashLoopBackOff
1. Check logs: `kubectl logs <pod> --previous`
2. Check events: `kubectl describe pod <pod>`
3. Verify image exists
...
```

## Skill Sources (Precedence)

Skills are loaded from multiple sources, with higher precedence sources overriding lower ones:

| Source | Precedence | Description |
|--------|------------|-------------|
| **Workspace** | Highest | `.claude/skills/` in your project |
| **Enterprise** | High | Organization-specific registry |
| **Public** | Medium | OpsSkillsHub community registry |
| **Bundled** | Lowest | Ships with AOF |

## Using Skills with aofctl

```bash
# List all skills
aofctl skills list

# List only eligible skills (requirements met)
aofctl skills list --eligible

# Check skill requirements
aofctl skills check k8s-debug

# View skill content
aofctl skills show k8s-debug

# Search skills
aofctl skills search "kubernetes debugging"

# Generate prompt for agents
aofctl skills prompt k8s-debug,prometheus-query
```

## Bundled Skills

AOF ships with essential ops skills:

| Skill | Description |
|-------|-------------|
| `k8s-debug` | Kubernetes pod debugging and troubleshooting |
| `prometheus-query` | PromQL queries and alerting patterns |
| `argocd-sync` | ArgoCD application management |
| `loki-search` | LogQL queries and log analysis |
| `incident-diagnose` | Systematic incident triage workflow |

## Next Steps

- [Writing Skills](./writing-skills) - Create your own skills
- [Skill Reference](./skill-reference) - Complete specification
- [Bundled Skills](./bundled-skills) - Documentation for included skills
