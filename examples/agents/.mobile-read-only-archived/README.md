# Mobile Read-Only Agents

Specialized agents for mobile platforms (Telegram, WhatsApp) that provide **read-only** access to infrastructure.

## Agents

| Agent | Description | Tools |
|-------|-------------|-------|
| `k8s-status` | Kubernetes cluster status | kubectl (read) |
| `docker-status` | Container status | docker (read) |
| `git-status` | Repository status | git (read) |
| `prometheus-query` | Metrics queries | promtool, curl |
| `helm-status` | Helm release status | helm (read) |

## Safety Features

All agents in this directory have:

1. **Tool Restrictions**: Only read operations allowed
2. **Blocked Verbs**: Create, delete, modify operations blocked
3. **Mobile Labels**: `mobile-safe: "true"` label for policy enforcement
4. **Short Responses**: Optimized for small screens

## Usage

### Via Telegram
```
/agents
> Select "k8s-status"

"what pods are failing?"
```

### Via CLI
```bash
aofctl run agent k8s-status "pod status in production"
```

### In Flows
```yaml
steps:
  - agent: k8s-status
    task: "check cluster health"
```

## Tool Restrictions Example

Each agent specifies allowed and blocked verbs:

```yaml
tool_restrictions:
  kubectl:
    allowed_verbs:
      - get
      - list
      - describe
      - logs
    blocked_verbs:
      - apply
      - delete
      - exec
```

## Platform Policy Integration

These agents work with platform policies to enforce safety:

```yaml
# In context configuration
platform_policies:
  telegram:
    default_agent: k8s-status
    blocked_classes: [delete, dangerous]
    allowed_classes: [read]
```

## Adding New Read-Only Agents

1. Copy an existing agent YAML
2. Add `mobile-safe: "true"` label
3. Define `tool_restrictions` with allowed/blocked verbs
4. Optimize `system_prompt` for mobile (short responses)
5. Test on Telegram with various queries
