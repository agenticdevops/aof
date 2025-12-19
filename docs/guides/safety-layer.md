# Safety Layer

The AOF Safety Layer provides **platform-agnostic** access control and tool classification for safe multi-platform agent operations. The same safety framework works identically across all messaging platforms - Telegram, Slack, Discord, WhatsApp, or any future platform.

## Overview

### Platform Trust Hierarchy

Different platforms have different trust levels based on authentication, audit capabilities, and typical usage context:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     PLATFORM TRUST HIERARCHY                            │
├─────────────────────────────────────────────────────────────────────────┤
│  CLI         → Highest trust (local, authenticated, auditable)          │
│  Slack       → High trust (enterprise SSO, desktop, audit logs)         │
│  Discord     → Medium trust (server-based, desktop/mobile)              │
│  Telegram    → Lower trust (mobile-first, personal device)              │
│  WhatsApp    → Lower trust (mobile-first, personal device)              │
│  SMS/Webhook → Lowest trust (no context, minimal auth)                  │
└─────────────────────────────────────────────────────────────────────────┘
```

| Platform | Trust Level | Default Policy |
|----------|-------------|----------------|
| CLI | Highest | All operations allowed |
| Slack | High | Write/delete require approval |
| Discord | Medium | Write/delete require approval |
| Telegram | Lower | Read-only by default |
| WhatsApp | Lowest | Read-only by default |

### Design Principles

1. **Platform-Agnostic Core**: Safety logic works identically across all platforms
2. **Policy-Driven**: Platform-specific rules are configuration, not code
3. **Fail Secure**: Unknown commands default to most restrictive classification (write)
4. **Extensible**: New platforms inherit sensible defaults automatically
5. **Composable**: Policies combine with Context for environment-specific rules

## Components

### Tool Classifier

Classifies tool invocations into action classes:

- **read**: Information retrieval only (e.g., `kubectl get pods`)
- **write**: Creates or modifies resources (e.g., `kubectl apply`)
- **delete**: Removes resources (e.g., `kubectl delete`)
- **dangerous**: Potentially harmful operations (e.g., `kubectl exec`, `rm -rf`)

```rust
use aof_triggers::safety::{ToolClassifier, ActionClass};

let classifier = ToolClassifier::new();
let result = classifier.classify("kubectl delete pod my-pod");

assert_eq!(result.class, ActionClass::Delete);
assert_eq!(result.tool, "kubectl");
```

### Platform Policy

Defines what operations are allowed on each platform:

```rust
use aof_triggers::safety::{PlatformPolicy, PolicyDecision};

// Read-only policy for mobile platforms
let telegram_policy = PlatformPolicy::read_only();

// Approval required for writes
let slack_policy = PlatformPolicy::require_write_approval();

// Full access for CLI
let cli_policy = PlatformPolicy::permissive();
```

### Safety Context

Combines classification and policy for complete safety evaluation:

```rust
use aof_triggers::safety::SafetyContext;

let ctx = SafetyContext::new("production");

// Evaluate a command
let eval = ctx.evaluate(
    "kubectl delete pod my-pod",
    "telegram",
    "user123",
    Some("production"),
);

if eval.is_blocked() {
    println!("Blocked: {}", eval.message);
} else if eval.needs_approval() {
    println!("Approval required: {}", eval.message);
}
```

## Configuration

### Context YAML

Define platform policies in a Context configuration:

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
  labels:
    environment: production

spec:
  namespace: production
  cluster: prod-cluster
  default_agent: k8s-status

  # Platform-specific policies
  platform_policies:
    telegram:
      blocked_classes:
        - delete
        - dangerous
      approval_classes:
        - write
      allowed_classes:
        - read
      blocked_message: |
        This operation is blocked on Telegram.
        Use Slack or kubectl directly.

    slack:
      blocked_classes:
        - dangerous
      approval_classes:
        - delete
        - write
      allowed_classes:
        - read
      approval_timeout_minutes: 30

    cli:
      blocked_classes: []
      approval_classes: []
      allowed_classes:
        - read
        - write
        - delete
        - dangerous

  # Users who can approve operations
  approval_allowed_users:
    - "@oncall"
    - "sre-team"

  # Additional safety settings
  safety:
    require_confirmation_for_namespace:
      - production
      - kube-system
    max_resources_per_operation: 10
    audit_all_operations: true
```

### Tool Classifications

Define verb classifications for tools:

```yaml
apiVersion: aof.dev/v1
kind: ToolClassification
metadata:
  name: default-classifications

spec:
  tools:
    kubectl:
      read:
        - get
        - list
        - describe
        - logs
      write:
        - apply
        - create
        - scale
      delete:
        - delete
      dangerous:
        - exec
        - port-forward

    docker:
      read:
        - ps
        - images
        - logs
      write:
        - run
        - build
      delete:
        - rm
        - rmi
      dangerous:
        - exec
        - --privileged

  # Generic patterns for unknown tools
  generic_patterns:
    read:
      - "^(get|list|show|describe|status)\\b"
    write:
      - "^(create|apply|update|deploy)\\b"
    delete:
      - "^(delete|destroy|remove|rm)\\b"
    dangerous:
      - "\\b(--force|-f)\\b"
      - "\\brm\\s+-rf\\b"
```

## Approval Workflow

When an operation requires approval:

1. Agent outputs `requires_approval: true` and `command: "..."` in response
2. Bot sends approval message with ✅ and ❌ reactions
3. Authorized user reacts to approve/deny
4. On approval, command is executed
5. Results are posted back

Example agent response triggering approval:

```
I'll scale the deployment to 5 replicas.

requires_approval: true
command: kubectl scale deployment/my-app --replicas=5
```

## Best Practices

1. **Start restrictive**: Use read-only policies for mobile platforms
2. **Require approval for writes**: Even in trusted environments
3. **Block dangerous ops on untrusted platforms**: Never allow `exec` from Telegram
4. **Audit production operations**: Enable `audit_all_operations`
5. **Use namespace confirmation**: Protect critical namespaces
6. **Limit resource scope**: Set `max_resources_per_operation`

## Testing the Safety Layer

### Run Unit Tests

```bash
# Test safety module (18 tests)
cargo test --package aof-triggers -- safety

# Test visualization crate (27 tests)
cargo test --package aof-viz

# Run all tests
cargo test --all
```

### Test with Example Configurations

```bash
# Start server with mobile agents and platform policies
aofctl serve \
  --platform telegram \
  --agents examples/agents/mobile-read-only \
  --context examples/contexts/telegram-prod.yaml

# Test from Telegram:
# 1. Send /agents - should show inline keyboard
# 2. Tap an agent - should switch and confirm
# 3. Send "what pods are running" - should respond with read data
# 4. Send "delete the pod" - should be blocked
```

### Example Files

| File | Purpose |
|------|---------|
| `examples/agents/mobile-read-only/*.yaml` | 5 read-only agents |
| `examples/tool-classifications/default.yaml` | Verb classifications for 10+ tools |
| `examples/contexts/telegram-prod.yaml` | Production policy (read-only) |
| `examples/contexts/telegram-dev.yaml` | Development policy (approval for writes) |

## Extending to New Platforms

The safety layer automatically works with new platforms. To add a new platform:

1. **Define a default policy** (in `PolicyEngine::default_policies()`):
```rust
policies.insert("matrix".to_string(), PlatformPolicy {
    blocked_classes: vec![ActionClass::Dangerous],
    approval_classes: vec![ActionClass::Delete, ActionClass::Write],
    allowed_classes: vec![ActionClass::Read],
    blocked_message: Some("Use CLI for dangerous operations".into()),
    approval_timeout_minutes: 30,
});
```

2. **Or override in Context YAML**:
```yaml
platform_policies:
  matrix:
    blocked_classes: [dangerous]
    approval_classes: [delete, write]
    allowed_classes: [read]
```

No code changes required - just configuration!

## Related

- [Context Configuration](/docs/reference/context.md)
- [Tool Classification](/docs/reference/tool-classification.md)
- [Telegram Mobile Companion](/docs/guides/telegram-mobile.md)
- [Approval Workflows](/docs/guides/approvals.md)
- [Platform Policies Reference](/docs/reference/platform-policies.md)
