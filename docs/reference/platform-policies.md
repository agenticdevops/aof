# Platform Policies Reference

Platform policies define what operations are allowed on each messaging platform. This is the core of AOF's platform-agnostic safety layer.

## Overview

Platform policies are evaluated at runtime to determine whether a tool invocation should be:
- **Allowed**: Execute immediately
- **Require Approval**: Wait for authorized user approval
- **Blocked**: Reject with helpful message

## Policy Structure

```yaml
platform_policies:
  <platform_name>:
    blocked_classes: [<action_class>, ...]
    approval_classes: [<action_class>, ...]
    allowed_classes: [<action_class>, ...]
    blocked_message: "<custom message>"
    approval_timeout_minutes: <number>
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `blocked_classes` | `[ActionClass]` | Operations that are always blocked |
| `approval_classes` | `[ActionClass]` | Operations requiring approval |
| `allowed_classes` | `[ActionClass]` | Operations allowed without approval |
| `blocked_message` | `string` | Custom message shown when blocked |
| `approval_timeout_minutes` | `number` | How long approval requests remain valid |

### Action Classes

| Class | Description | Example Commands |
|-------|-------------|------------------|
| `read` | Information retrieval only | `kubectl get`, `docker ps`, `git status` |
| `write` | Creates or modifies resources | `kubectl apply`, `docker run`, `git push` |
| `delete` | Removes resources | `kubectl delete`, `docker rm`, `git reset --hard` |
| `dangerous` | Potentially harmful operations | `kubectl exec`, `--force`, `sudo`, `rm -rf` |

## Default Policies

AOF includes sensible defaults for common platforms:

### CLI (Highest Trust)

```yaml
cli:
  blocked_classes: []
  approval_classes: []
  allowed_classes: [read, write, delete, dangerous]
```

CLI has full access because:
- User is locally authenticated
- Operations are auditable
- Full context is available

### Slack (High Trust)

```yaml
slack:
  blocked_classes: [dangerous]
  approval_classes: [delete, write]
  allowed_classes: [read]
  approval_timeout_minutes: 30
```

Slack is trusted but requires approval because:
- Enterprise SSO authentication
- Desktop-first experience
- Audit logs available

### Discord (Medium Trust)

```yaml
discord:
  blocked_classes: [dangerous]
  approval_classes: [delete, write]
  allowed_classes: [read]
  approval_timeout_minutes: 30
```

### Telegram (Lower Trust)

```yaml
telegram:
  blocked_classes: [delete, dangerous]
  approval_classes: [write]
  allowed_classes: [read]
  blocked_message: "Use Slack or CLI for this operation."
```

Telegram is restricted because:
- Mobile-first (less controlled environment)
- Personal device (may be shared/unlocked)
- Limited audit capabilities

### WhatsApp (Lower Trust)

```yaml
whatsapp:
  blocked_classes: [delete, dangerous, write]
  approval_classes: []
  allowed_classes: [read]
  blocked_message: "WhatsApp is read-only. Use Slack for modifications."
```

### SMS/Webhook (Lowest Trust)

```yaml
sms:
  blocked_classes: [delete, dangerous, write]
  approval_classes: []
  allowed_classes: [read]
```

Webhooks and SMS have minimal authentication context.

## Custom Policies

Override defaults in your Context YAML:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: my-context
spec:
  platform_policies:
    # Custom Telegram policy allowing writes with approval
    telegram:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]
      allowed_classes: [read]
      approval_timeout_minutes: 15
      blocked_message: |
        Dangerous operations require CLI access.

    # Custom Slack policy blocking deletes entirely
    slack:
      blocked_classes: [delete, dangerous]
      approval_classes: [write]
      allowed_classes: [read]
```

## Adding New Platforms

New platforms automatically inherit a secure default:
- `blocked_classes: [dangerous]`
- `approval_classes: [delete, write]`
- `allowed_classes: [read]`

To customize, add the platform to your Context:

```yaml
platform_policies:
  # New platform (e.g., Matrix, Signal, Teams)
  matrix:
    blocked_classes: [dangerous]
    approval_classes: [delete]
    allowed_classes: [read, write]
```

No code changes required - the policy engine uses configuration.

## Policy Evaluation

When a command is received:

1. **Classify the command** using ToolClassifier
2. **Look up platform policy** from Context or defaults
3. **Evaluate action class** against policy:
   - If in `blocked_classes` → Block with message
   - If in `approval_classes` → Require approval
   - If in `allowed_classes` → Allow immediately
   - If not in any → Default to block (fail secure)

```rust
// Pseudocode
let class = classifier.classify("kubectl delete pod my-pod");
// class = ActionClass::Delete

let decision = policy_engine.evaluate("telegram", class);
// decision = PolicyDecision::Block { reason: "..." }
```

## Environment-Specific Policies

Combine platform policies with environment context:

### Production Environment

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: production
spec:
  namespace: production

  platform_policies:
    telegram:
      blocked_classes: [delete, dangerous, write]  # Read-only in prod
      allowed_classes: [read]
      blocked_message: "Production is read-only from Telegram."

    slack:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]  # Require approval for all writes
      allowed_classes: [read]

  safety:
    require_confirmation_for_namespace: [production, kube-system]
    audit_all_operations: true
```

### Development Environment

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: development
spec:
  namespace: development

  platform_policies:
    telegram:
      blocked_classes: [dangerous]
      approval_classes: [delete]  # More permissive
      allowed_classes: [read, write]

    slack:
      blocked_classes: []
      approval_classes: [dangerous]  # Only dangerous needs approval
      allowed_classes: [read, write, delete]
```

## Approval Workflow Integration

When a command requires approval:

1. Agent outputs `requires_approval: true` with the command
2. Bot sends approval message with ✅ and ❌ reactions
3. User from `approval_allowed_users` approves/rejects
4. On approval, command executes
5. Results posted back

Configure approvers:

```yaml
spec:
  approval_allowed_users:
    - "@oncall"           # Slack user group
    - "sre-team"          # Generic group
    - "U015VBH1GTZ"       # Specific Slack user ID
    - "email:admin@co.io" # Email format (future)
```

## Example Configurations

### Strict Production

```yaml
# examples/contexts/telegram-prod.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  platform_policies:
    telegram:
      blocked_classes: [delete, dangerous]
      approval_classes: [write]
      allowed_classes: [read]
      blocked_message: |
        This operation is blocked on Telegram for production.
        Use Slack or kubectl directly for write/delete operations.
```

### Relaxed Development

```yaml
# examples/contexts/telegram-dev.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: dev
spec:
  platform_policies:
    telegram:
      blocked_classes: [dangerous]
      approval_classes: []
      allowed_classes: [read, write, delete]
```

### Personal/Lab

```yaml
# examples/contexts/telegram-personal.yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: personal
spec:
  platform_policies:
    telegram:
      blocked_classes: []
      approval_classes: [dangerous]
      allowed_classes: [read, write, delete]
```

## Rust API

```rust
use aof_triggers::safety::{PolicyEngine, PolicyDecision, ActionClass};

// Create engine with defaults
let engine = PolicyEngine::with_defaults();

// Evaluate a command
let decision = engine.evaluate("telegram", ActionClass::Delete);

match decision {
    PolicyDecision::Allow => {
        // Execute the command
    }
    PolicyDecision::RequireApproval { reason, timeout_minutes } => {
        // Send approval request
    }
    PolicyDecision::Block { reason, suggestion } => {
        // Return error with suggestion
    }
}
```

## Related

- [Safety Layer Guide](/docs/guides/safety-layer.md)
- [Tool Classification Reference](/docs/reference/tool-classification.md)
- [Context Reference](/docs/reference/context.md)
- [Approval Workflow Guide](/docs/guides/approval-workflow.md)
