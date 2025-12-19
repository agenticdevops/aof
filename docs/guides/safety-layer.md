# Safety Layer

The AOF Safety Layer provides simple, platform-based access control for safe multi-platform agent operations.

## Overview

### Platform Access Levels

| Platform | Write Access | Description |
|----------|--------------|-------------|
| **CLI** | ✅ Full | Local terminal, full trust |
| **Slack** | ✅ Full | Enterprise platform with approval workflow |
| **Telegram** | ❌ Read-only | Mobile platform, writes blocked |
| **WhatsApp** | ❌ Read-only | Mobile platform, writes blocked |

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     PLATFORM ACCESS HIERARCHY                           │
├─────────────────────────────────────────────────────────────────────────┤
│  CLI         → Full access (local, authenticated)                       │
│  Slack       → Full access + approval workflow for destructive ops      │
│  Telegram    → Read-only (mobile safety)                                │
│  WhatsApp    → Read-only (mobile safety)                                │
└─────────────────────────────────────────────────────────────────────────┘
```

## How It Works

### Write Operation Detection

The safety layer detects write operations using pattern matching:

**kubectl**
- Blocked: `apply`, `create`, `delete`, `patch`, `edit`, `replace`, `set`, `scale`, `rollout`, `drain`, `cordon`, `taint`, `label`, `annotate`, `expose`
- Allowed: `get`, `describe`, `logs`, `top`, `explain`

**docker**
- Blocked: `rm`, `rmi`, `stop`, `kill`, `prune`, `push`, `build`, `run`, `exec`
- Allowed: `ps`, `images`, `inspect`, `logs`

**helm**
- Blocked: `install`, `upgrade`, `delete`, `uninstall`, `rollback`
- Allowed: `list`, `status`, `get`, `search`, `show`

**terraform**
- Blocked: `apply`, `destroy`, `import`
- Allowed: `plan`, `show`, `state list`, `output`

**aws**
- Blocked: `terminate`, `stop`, `start`, `run`, `rm`, `cp`, `mv`, `sync`, `update`, `delete`
- Allowed: `describe`, `list`, `get`

**git**
- Blocked: `push`, `commit`, `reset`, `revert`, `merge`, `rebase`, `checkout`, `branch -d`
- Allowed: `status`, `log`, `diff`, `show`, `branch` (list)

**Natural Language**
- Blocked intents: "create", "deploy", "delete", "remove", "scale", "restart", "update", "apply", "install", "uninstall", "rollback", "push", "commit", "terminate", "stop", "kill"

### Slack Approval Workflow

On Slack, destructive operations trigger an approval workflow:

1. Agent detects destructive command
2. Posts approval request with ✅ ❌ reactions
3. Authorized user reacts to approve/deny
4. Command executes only after approval

Configure allowed approvers in your flow config:
```yaml
spec:
  platforms:
    slack:
      approval_allowed_users:
        - U015VBH1GTZ  # Slack user ID
        - U087ABC1234
```

### Telegram Read-Only Mode

On Telegram, write operations are blocked immediately:

```
🚫 Write operation blocked

telegram is read-only. Write, delete, and dangerous operations
are not allowed from mobile.

What you can do:
• Use read-only commands (get, list, describe, logs)
• Use Slack or CLI for write operations

Detected write intent: `create deployment nginx`
```

## Testing

### Test Read Operations (Telegram)
```
User: list pods
Bot: [shows pod list]

User: describe pod nginx-abc123
Bot: [shows pod details]

User: kubectl get deployments
Bot: [shows deployments]
```

### Test Blocked Operations (Telegram)
```
User: create deployment nginx
Bot: 🚫 Write operation blocked...

User: kubectl delete pod nginx-abc123
Bot: 🚫 Write operation blocked...

User: scale deployment nginx to 5 replicas
Bot: 🚫 Write operation blocked...
```

### Test Approval Flow (Slack)
```
User: kubectl delete pod nginx-abc123
Bot: ⚠️ This action requires approval
     `kubectl delete pod nginx-abc123`
     React with ✅ to approve or ❌ to deny.

[User reacts with ✅]
Bot: [executes command]
```

## Design Principles

1. **Simple**: Platform-based check, no complex policy engine
2. **Safe by Default**: Mobile platforms are read-only
3. **Fast**: Pattern matching, no external calls
4. **Clear Feedback**: Users know why operations are blocked
5. **Extensible**: Easy to add new platforms or patterns

## Future Enhancements

For enterprise deployments, these features can be added later:
- Per-context read/write policies
- Time-based access windows
- User/group-based permissions
- Audit logging
- Custom approval workflows per operation type
