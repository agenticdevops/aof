# Internal Design: Platform-Aware Mobile Companion

**Status**: ✅ Implemented (Phase 1 & 2)
**Author**: AOF Team
**Created**: December 19, 2024
**Epic**: Multi-Platform Safety & UX (GitHub #38)

---

## Executive Summary

This document describes the design for a **platform-agnostic safety layer** and **mobile-first UX** for AOF agents. While initially focused on Telegram, the architecture is designed to work across all messaging platforms.

### Implemented Components

| Component | Status | Location |
|-----------|--------|----------|
| Tool Classification | ✅ Complete | `crates/aof-triggers/src/safety/classifier.rs` |
| Platform Policies | ✅ Complete | `crates/aof-triggers/src/safety/policy.rs` |
| Safety Context | ✅ Complete | `crates/aof-triggers/src/safety/context.rs` |
| /agents Command | ✅ Complete | `crates/aof-triggers/src/handler/mod.rs` |
| /flows Command | ✅ Complete | `crates/aof-triggers/src/handler/mod.rs` |
| ASCII Visualization | ✅ Complete | `crates/aof-viz/` |
| Mobile Agents | ✅ Complete | `examples/agents/mobile-read-only/` |

---

## Strategic Context

### Multi-Platform Safety Model

The safety layer is **platform-agnostic** - the same classification and policy system works across all platforms:

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

### Design Principles

1. **Platform-Agnostic Core**: Safety logic works identically across all platforms
2. **Policy-Driven**: Platform-specific rules are configuration, not code
3. **Fail Secure**: Unknown commands default to most restrictive classification
4. **Extensible**: New platforms inherit sensible defaults automatically
5. **Composable**: Policies combine with Context for environment-specific rules

---

## Architecture Overview

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          SAFETY LAYER ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐                                                   │
│  │ Incoming Message │  (Telegram, Slack, WhatsApp, Discord, etc.)       │
│  └────────┬─────────┘                                                   │
│           │                                                              │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ TriggerHandler (crates/aof-triggers/src/handler/mod.rs)          │   │
│  │ ├── Parse commands (/agents, /flows, /help)                      │   │
│  │ ├── Handle callbacks (callback:agent:*, callback:flow:*)         │   │
│  │ ├── Manage session state (user → selected agent)                 │   │
│  │ └── Route natural language to agents                             │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│           │                                                              │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Agent Execution (with safety check)                               │   │
│  │ ├── LLM generates tool call                                      │   │
│  │ ├── ToolClassifier.classify(command) → ActionClass               │   │
│  │ ├── PolicyEngine.evaluate(platform, class) → Decision            │   │
│  │ │   ├── Allow → Execute                                          │   │
│  │ │   ├── RequireApproval → Trigger approval flow                  │   │
│  │ │   └── Block → Return error with suggestion                     │   │
│  │ └── Return result                                                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│           │                                                              │
│           ▼                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ Response Rendering (aof-viz)                                      │   │
│  │ ├── StatusRenderer (execution status, thinking indicator)        │   │
│  │ ├── FlowRenderer (workflow progress, node status)                │   │
│  │ ├── ToolRenderer (command output, tables)                        │   │
│  │ ├── SafetyRenderer (blocked/approval messages)                   │   │
│  │ └── Platform-specific RenderConfig (width, colors, compact)      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### 1. Safety Module (`crates/aof-triggers/src/safety/`)

#### Action Classes

```rust
// classifier.rs
pub enum ActionClass {
    Read,      // get, list, describe, logs - information retrieval
    Write,     // apply, create, scale - modifications
    Delete,    // delete, destroy, rm - resource removal
    Dangerous, // exec, --force, sudo - potentially harmful
}
```

#### Tool Classification

The classifier uses a priority system:

1. **Dangerous patterns** (highest priority) - `--force`, `rm -rf`, `sudo`
2. **Tool-specific verbs** - `kubectl get` → Read, `kubectl delete` → Delete
3. **Generic patterns** - regex matching for unknown tools
4. **Default: Write** (fail secure)

#### Platform Policies

```rust
// policy.rs
pub enum PolicyDecision {
    Allow,
    RequireApproval { reason: String, timeout_minutes: u32 },
    Block { reason: String, suggestion: Option<String> },
}

// Default policies (can be overridden in Context YAML)
pub fn default_policies() -> HashMap<String, PlatformPolicy> {
    // CLI: All allowed
    // Slack: Write/Delete require approval, Dangerous blocked
    // Telegram/WhatsApp: Read only, everything else blocked
    // Discord: Same as Slack
}
```

### 2. Command Handling (`crates/aof-triggers/src/handler/`)

#### Session State

```rust
// User's selected agent persists across messages
user_agent_sessions: Arc<DashMap<String, String>>

// Methods
fn set_user_agent(&self, user_id: &str, agent_name: &str)
fn get_user_agent(&self, user_id: &str) -> Option<String>
```

#### Callback Handling

```rust
// Telegram inline keyboard callbacks
if message.text.starts_with("callback:") {
    match parse_callback(&message.text) {
        Callback::Agent(name) => set_user_agent(user_id, name),
        Callback::Flow(name) => execute_flow(name),
        Callback::Approve(id) => approve_action(id),
        Callback::Reject(id) => reject_action(id),
    }
}
```

### 3. Visualization (`crates/aof-viz/`)

#### Platform-Specific Configs

```rust
impl RenderConfig {
    pub fn telegram() -> Self {
        Self { max_width: 35, use_unicode: true, use_colors: false, compact: true }
    }
    pub fn slack() -> Self {
        Self { max_width: 50, use_unicode: true, use_colors: false, compact: false }
    }
    pub fn terminal() -> Self {
        Self { max_width: 80, use_unicode: true, use_colors: true, compact: false }
    }
}
```

#### Renderers

| Renderer | Purpose |
|----------|---------|
| `StatusRenderer` | Execution status, thinking indicators |
| `FlowRenderer` | Workflow progress, node visualization |
| `ToolRenderer` | Command output, tables, code blocks |
| `SafetyRenderer` | Blocked/approval messages |
| `ProgressBar` | Long-running operation progress |
| `Spinner` | Loading animations |

---

## Configuration

### Context YAML with Platform Policies

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: production
spec:
  namespace: production
  cluster: prod-cluster
  default_agent: k8s-status  # Read-only agent

  platform_policies:
    telegram:
      blocked_classes: [delete, dangerous]
      approval_classes: [write]
      allowed_classes: [read]
      blocked_message: "Use Slack or CLI for write operations."

    slack:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]
      allowed_classes: [read]
      approval_timeout_minutes: 30

    whatsapp:
      blocked_classes: [delete, dangerous, write]
      approval_classes: []
      allowed_classes: [read]

    discord:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]
      allowed_classes: [read]

    cli:
      blocked_classes: []
      approval_classes: []
      allowed_classes: [read, write, delete, dangerous]

  approval_allowed_users:
    - "@oncall"
    - "sre-team"

  safety:
    require_confirmation_for_namespace: [production, kube-system]
    max_resources_per_operation: 10
    audit_all_operations: true
```

### Tool Classifications

```yaml
apiVersion: aof.dev/v1
kind: ToolClassification
metadata:
  name: default-classifications
spec:
  tools:
    kubectl:
      read: [get, list, describe, logs, top, explain]
      write: [apply, create, patch, scale, rollout]
      delete: [delete]
      dangerous: [exec, port-forward, cp]

    docker:
      read: [ps, images, logs, inspect, stats]
      write: [run, build, push, pull, tag]
      delete: [rm, rmi, stop, kill, prune]
      dangerous: [exec, --privileged]

    # ... more tools ...

  generic_patterns:
    read: ["^(get|list|show|describe|status)\\b"]
    write: ["^(create|apply|update|deploy)\\b"]
    delete: ["^(delete|destroy|remove|rm)\\b"]
    dangerous: ["\\b(--force|-f)\\b", "\\bsudo\\b"]
```

---

## Testing

### Unit Tests (45 total)

| Module | Tests | Status |
|--------|-------|--------|
| `safety::classifier` | 8 | ✅ Pass |
| `safety::policy` | 6 | ✅ Pass |
| `safety::context` | 4 | ✅ Pass |
| `aof-viz::status` | 4 | ✅ Pass |
| `aof-viz::flow` | 4 | ✅ Pass |
| `aof-viz::tools` | 5 | ✅ Pass |
| `aof-viz::safety` | 5 | ✅ Pass |
| `aof-viz::progress` | 6 | ✅ Pass |
| `aof-viz::lib` | 3 | ✅ Pass |

### Integration Testing

To test with a live Telegram bot:

```bash
# Set up environment
export TELEGRAM_BOT_TOKEN="your-token"

# Start server with mobile agents
aofctl serve \
  --platform telegram \
  --agents examples/agents/mobile-read-only \
  --context examples/contexts/telegram-prod.yaml

# Test commands
# 1. Send /agents - should show inline keyboard
# 2. Tap an agent - should switch and confirm
# 3. Send "what pods are running" - should respond with read data
# 4. Send "delete the pod" - should be blocked
```

---

## Future Enhancements

### Phase 3: Cross-Platform Features

1. **Cross-Platform Approval**: Approve on Slack, execute on Telegram
2. **Session Persistence**: Redis/database for session state
3. **Escalation Command**: `/escalate` to hand off to another platform
4. **Chart Generation**: PNG charts via Chart.js/Vega

### Phase 4: Additional Platforms

1. **WhatsApp Business API**: Same safety layer, WhatsApp-specific formatting
2. **MS Teams**: Enterprise integration with Azure AD
3. **Discord**: Bot with slash commands
4. **Signal**: Privacy-focused option

### Phase 5: Advanced Safety

1. **ML-Based Classification**: Learn from usage patterns
2. **Anomaly Detection**: Flag unusual command patterns
3. **Role-Based Policies**: Per-user/group policy overrides
4. **Audit Dashboard**: Visualization of blocked/approved operations

---

## File Reference

### New Files Created

| File | Purpose |
|------|---------|
| `crates/aof-triggers/src/safety/mod.rs` | Safety module exports |
| `crates/aof-triggers/src/safety/classifier.rs` | Tool classification engine |
| `crates/aof-triggers/src/safety/policy.rs` | Platform policy enforcement |
| `crates/aof-triggers/src/safety/context.rs` | Safety context combining both |
| `crates/aof-viz/Cargo.toml` | Visualization crate config |
| `crates/aof-viz/src/lib.rs` | Viz crate exports |
| `crates/aof-viz/src/status.rs` | Status rendering |
| `crates/aof-viz/src/flow.rs` | Flow visualization |
| `crates/aof-viz/src/tools.rs` | Tool output formatting |
| `crates/aof-viz/src/safety.rs` | Safety decision rendering |
| `crates/aof-viz/src/progress.rs` | Progress indicators |
| `examples/agents/mobile-read-only/*.yaml` | 5 read-only agents |
| `examples/tool-classifications/default.yaml` | Default verb classifications |
| `examples/contexts/telegram-*.yaml` | Platform policy examples |

### Modified Files

| File | Changes |
|------|---------|
| `crates/aof-triggers/src/lib.rs` | Export safety module |
| `crates/aof-triggers/src/command/mod.rs` | Add Agents, Flows commands |
| `crates/aof-triggers/src/handler/mod.rs` | Session state, callbacks, new handlers |
| `crates/aof-triggers/src/flow/router.rs` | Add list_flows(), get_flow() |
| `Cargo.toml` | Add aof-viz to workspace |

---

## References

- [Telegram Bot API - Inline Keyboards](https://core.telegram.org/bots/api#inlinekeyboardmarkup)
- [GitHub Epic #38](https://github.com/agenticdevops/aof/issues/38)
- [User Guide: Safety Layer](/docs/guides/safety-layer.md)
- [User Guide: Telegram Mobile](/docs/guides/telegram-mobile.md)
