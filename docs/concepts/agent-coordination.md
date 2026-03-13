# Agent Coordination

## Overview

Agent coordination in OpenAgentiX allows a coordinator agent to delegate work to specialist agents and synthesize their results. This pattern separates concerns: the coordinator handles task routing and synthesis; specialists handle domain-specific execution.

A coordinator agent does not run tools directly. Instead, it calls other agents via the `/api/v1/agents/:name/delegate` endpoint, receives their `DelegationResult`, and assembles a final response.

## Core Types

### DelegationMessage

The task envelope sent from a coordinator to a specialist:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Unique delegation ID (UUID) |
| `from_agent` | `String` | Name of the coordinator agent |
| `task` | `String` | Natural-language task description |
| `payload` | `Value` | Structured parameters for the specialist |
| `reply_to` | `Option<String>` | Which agent should receive the result |

### DelegationResult

The outcome envelope returned from a specialist to the coordinator:

| Field | Type | Description |
|-------|------|-------------|
| `delegation_id` | `String` | ID matching the original `DelegationMessage.id` |
| `from_agent` | `String` | Name of the specialist that produced this result |
| `output` | `String` | Natural-language output from the specialist |
| `status` | `DelegationStatus` | `success` or `failure` |
| `completed_at` | `DateTime<Utc>` | Timestamp of completion |

### AgentInbox

Each registered agent has a bounded inbox (capacity: 256 `DelegationMessage` items). The inbox is backed by a Tokio `mpsc` channel.

- `AgentInbox::new(capacity)` — create with custom capacity
- `AgentInbox::default_capacity()` — create with the default 256-item capacity
- `inbox.send(msg)` — push a delegation to the inbox (async, back-pressures when full)
- `inbox.receive()` — pop the next delegation (async, returns `None` when closed)
- `inbox.sender()` — clone the channel sender for use across threads

Agent memory is isolated: each agent's inbox is keyed by agent name in `AgentManager`. Agent A cannot push to Agent B's inbox without going through the `CoordinatorProtocol`.

## CoordinatorProtocol Trait

```rust
#[async_trait]
pub trait CoordinatorProtocol: Send + Sync {
    async fn delegate(
        &self,
        target: &str,
        task: &str,
        payload: serde_json::Value,
    ) -> AgentixResult<DelegationResult>;

    async fn delegate_parallel(
        &self,
        delegations: Vec<(String, String, serde_json::Value)>,
    ) -> AgentixResult<Vec<DelegationResult>>;
}
```

- `delegate()` — sends one task to one specialist, awaits the result
- `delegate_parallel()` — fans out N tasks concurrently, returns results in input order

`AgentManager` implements `CoordinatorProtocol`. When a coordinator agent calls the `/api/v1/agents/:name/delegate` endpoint, the gateway routes through this implementation.

## Audit Trail

Every delegation creates an `AuditEvent` appended to the run's audit log:

```json
{
  "event_type": "delegation",
  "actor": "ops-coordinator",
  "target": "dba-specialist",
  "delegation_id": "uuid-...",
  "timestamp": "2026-03-13T..."
}
```

This means every `agentix runs` entry for a coordinator run shows all child delegations with their parent context. Full parent/child tracing satisfies COORD-05.

## Example Agent YAML

A coordinator that delegates database and infrastructure tasks:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: ops-coordinator
spec:
  model: anthropic/claude-3-5-haiku
  system_prompt: |
    You are an ops coordinator. Delegate sub-tasks to specialists using
    the delegate tools, then synthesize their results.
  tools:
    - type: http
      name: delegate_to_dba
      url: http://localhost:7777/api/v1/agents/dba-specialist/delegate
      method: POST
    - type: http
      name: delegate_to_infra
      url: http://localhost:7777/api/v1/agents/infra-specialist/delegate
      method: POST
```

A specialist that accepts delegated tasks:

```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: dba-specialist
spec:
  model: anthropic/claude-3-5-haiku
  triggers:
    - type: agent
      id: agent-trigger-dba
```

## Limitations and Future Work

Phase 16 supports local in-process coordination: all agents run within the same `agentix serve` instance. The coordination protocol uses in-process `mpsc` channels and HTTP calls to the local gateway.

Distributed coordination (delegating across multiple gateway instances on different hosts) is planned for v2.1. The `CoordinatorProtocol` trait is designed to support remote implementations without breaking the API.

Circular delegation (A delegates to B which delegates back to A) is not detected at runtime in v2.0. Ensure coordinator agents do not create delegation cycles.
