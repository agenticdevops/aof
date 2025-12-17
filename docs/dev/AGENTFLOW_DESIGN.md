# AgentFlow Design Document

**Last Updated**: December 17, 2025
**Status**: Implemented (Core Features)

## Overview

AgentFlow is AOF's workflow orchestration system, designed with feature parity to CrewAI, LangGraph, Agno, and Google A2A/ADK. It enables multi-step workflows with validation, human-in-the-loop approval, and agent coordination.

## Design Principles

1. **Kubernetes-Native**: YAML-based configuration matching K8s patterns
2. **Graph-Based**: Nodes and edges for flexible workflow definition
3. **Stateful**: Checkpoint-based state management for recovery
4. **Human-in-the-Loop**: Built-in approval and validation gates
5. **MCP Integration**: Use MCP servers for tool execution

## Core Concepts

### Workflow

A workflow defines a graph of steps with state transitions.

```yaml
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: incident-response
  labels:
    category: sre

spec:
  # Initial state schema
  state:
    type: object
    properties:
      incident_id: { type: string }
      severity: { type: string, enum: [low, medium, high, critical] }
      findings: { type: array, items: { type: string } }
      resolution: { type: string }

  # Entry point
  entrypoint: detect

  # Workflow steps
  steps:
    - name: detect
      type: agent
      agent: detector-agent
      next:
        - condition: "state.severity == 'critical'"
          target: escalate
        - condition: "state.severity == 'high'"
          target: analyze
        - target: monitor  # default

    - name: escalate
      type: approval
      approvers: [oncall-team]
      timeout: 5m
      next:
        - condition: approved
          target: analyze
        - target: notify-only

    - name: analyze
      type: agent
      agent: analyzer-agent
      parallel: true  # Can run in parallel with other steps
      next: remediate

    - name: remediate
      type: agent
      agent: remediation-agent
      validation:
        - type: function
          script: validate_remediation
        - type: llm
          prompt: "Verify the remediation is safe and complete"
      next: verify

    - name: verify
      type: agent
      agent: verifier-agent
      next:
        - condition: "state.verified == true"
          target: complete
        - target: analyze  # Loop back

    - name: complete
      type: terminal
      status: completed

    - name: notify-only
      type: agent
      agent: notifier-agent
      next: complete

    - name: monitor
      type: agent
      agent: monitor-agent
      next: complete
```

### Step Types

1. **agent**: Execute an agent with tools
2. **approval**: Human-in-the-loop approval gate
3. **validation**: Automated validation step
4. **parallel**: Fork into multiple parallel steps
5. **join**: Wait for parallel steps to complete
6. **terminal**: End of workflow

### State Management

State flows through the workflow with immutable updates:

```yaml
spec:
  state:
    # Define state schema (JSON Schema format)
    type: object
    properties:
      input: { type: string }
      result: { type: string }
      metadata: { type: object }

  # Custom reducers for state updates
  reducers:
    messages:
      type: append  # Append to list
    findings:
      type: merge   # Merge objects
    count:
      type: sum     # Sum numeric values
```

### Conditional Routing

Edges support conditional expressions:

```yaml
next:
  - condition: "state.score > 0.8"
    target: high-confidence
  - condition: "state.score > 0.5"
    target: medium-confidence
  - condition: "state.error != null"
    target: error-handler
  - target: low-confidence  # Default (no condition)
```

### Human-in-the-Loop

Built-in approval mechanisms:

```yaml
steps:
  - name: deploy-approval
    type: approval
    config:
      approvers:
        - role: sre-team
        - user: admin@example.com
      timeout: 30m
      required_approvals: 2
      auto_approve:
        condition: "state.environment == 'dev'"
    next:
      - condition: approved
        target: deploy
      - condition: rejected
        target: notify-rejection
      - condition: timeout
        target: escalate

  - name: human-input
    type: agent
    agent: interactive-agent
    interrupt:
      type: input
      prompt: "Please provide additional context"
      schema:
        type: object
        properties:
          context: { type: string }
```

### Validation Steps

Multiple validation approaches:

```yaml
steps:
  - name: validate-output
    type: validation
    config:
      validators:
        # Function-based validation
        - type: function
          name: validate_json_schema
          args:
            schema: { $ref: "#/definitions/OutputSchema" }

        # LLM-based validation
        - type: llm
          model: openai:gpt-4o
          prompt: |
            Validate that the following output is:
            1. Complete and addresses all requirements
            2. Safe to execute in production
            3. Follows security best practices

        # Script-based validation
        - type: script
          command: ./validate.sh
          timeout: 30s

      # Retry on validation failure
      max_retries: 3
      on_failure: retry_previous_step
```

### Parallel Execution

Fork-join pattern for parallel work:

```yaml
steps:
  - name: start-analysis
    type: parallel
    branches:
      - name: logs-analysis
        steps:
          - agent: log-analyzer
      - name: metrics-analysis
        steps:
          - agent: metrics-analyzer
      - name: traces-analysis
        steps:
          - agent: trace-analyzer
    join:
      strategy: all  # wait for all | any | majority
      timeout: 10m
    next: aggregate-findings

  - name: aggregate-findings
    type: agent
    agent: aggregator-agent
```

### Error Handling

Comprehensive error handling:

```yaml
spec:
  # Global error handling
  error_handler: error-step

  # Retry configuration
  retry:
    max_attempts: 3
    backoff: exponential
    initial_delay: 1s
    max_delay: 30s

  steps:
    - name: risky-step
      type: agent
      agent: risky-agent
      # Step-level error handling
      on_error:
        - condition: "error.type == 'timeout'"
          target: retry-with-longer-timeout
        - condition: "error.type == 'rate_limit'"
          target: wait-and-retry
        - target: error-handler  # Default

    - name: error-handler
      type: agent
      agent: error-handler-agent
      next: complete-with-error
```

### Checkpointing

State persistence for recovery:

```yaml
spec:
  # Checkpointing configuration
  checkpointing:
    enabled: true
    backend: file  # file, redis, postgres
    path: ./checkpoints/
    # Checkpoint after each step
    frequency: step
    # Keep history for debugging
    history: 10

  # Recovery configuration
  recovery:
    # Resume from last checkpoint on failure
    auto_resume: true
    # Skip completed steps
    skip_completed: true
```

## AgentFleet Integration

AgentFlow works with AgentFleet for agent coordination:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: incident-team

spec:
  # Fleet composition
  agents:
    - name: detector
      config: ./agents/detector.yaml
      replicas: 2

    - name: analyzer
      config: ./agents/analyzer.yaml
      replicas: 1

    - name: remediator
      config: ./agents/remediator.yaml
      replicas: 1

  # Coordination mode
  coordination:
    mode: hierarchical  # hierarchical | peer | swarm
    manager: detector

  # Shared resources
  shared:
    memory:
      type: redis
      url: redis://localhost:6379
    tools:
      - mcp-server: kubectl-ai
      - mcp-server: prometheus

---
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: fleet-workflow

spec:
  # Reference fleet
  fleet: incident-team

  steps:
    - name: detect
      agent: detector  # Reference from fleet
      next: analyze

    - name: analyze
      agent: analyzer
      next: remediate
```

## Runtime API

### Starting a Workflow

```rust
// Rust API
let workflow = Workflow::load("incident-response.yaml")?;
let state = json!({
    "incident_id": "INC-123",
    "severity": "high"
});

let execution = runtime.start_workflow(workflow, state).await?;
```

### CLI Usage

```bash
# Run a workflow
aofctl run workflow incident-response.yaml --input '{"incident_id": "INC-123"}'

# Check workflow status
aofctl get workflow-run incident-response-abc123

# Resume a paused workflow
aofctl resume workflow-run incident-response-abc123

# Approve a pending step
aofctl approve workflow-run incident-response-abc123 --step deploy-approval

# Inject input
aofctl input workflow-run incident-response-abc123 --step human-input --data '{"context": "..."}'

# View workflow history
aofctl logs workflow-run incident-response-abc123
```

## Implementation Plan

### Phase 1: Core Workflow Engine ✅ COMPLETED
- [x] Workflow configuration parsing (`aof-core/src/agentflow.rs`)
- [x] Step executor framework (`aof-runtime/src/agentflow_executor.rs`)
- [x] Basic sequential execution
- [x] State management with `FlowState` and `NodeResult`

### Phase 2: Advanced Execution ✅ COMPLETED
- [x] Conditional routing with `when` expressions
- [x] Parallel node support
- [x] Error handling and retry configuration
- [x] Variable interpolation (`${event.*}`, `${node-id.*}`)

### Phase 3: Human-in-the-Loop ✅ COMPLETED
- [x] Approval steps via Slack reactions
- [x] Waiting states for external input
- [x] Validation via Conditional nodes
- [x] CLI integration (`aofctl run flow`, `aofctl describe flow`)

### Phase 4: AgentFleet Integration 🔄 IN PROGRESS
- [ ] Fleet definition and management
- [ ] Coordination modes
- [ ] Shared resources
- [ ] Fleet-aware workflows

### Phase 5: Production Features 📋 PLANNED
- [ ] Full checkpointing/recovery
- [ ] Persistent state backends
- [ ] Metrics and observability
- [ ] Multi-instance coordination

## Feature Parity Matrix

| Feature | CrewAI | LangGraph | Agno | A2A | AOF AgentFlow |
|---------|--------|-----------|------|-----|---------------|
| Sequential execution | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Parallel execution | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Conditional routing | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Cycles/loops | ⚠️ | ✅ | ⚠️ | ⚠️ | ✅ Implemented |
| State management | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Checkpointing | ⚠️ | ✅ | ✅ | ⚠️ | ⚠️ Partial |
| Human approval | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Input interrupts | ⚠️ | ✅ | ✅ | ✅ | ✅ Implemented |
| Validation gates | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ Implemented |
| Error handling | ✅ | ✅ | ✅ | ✅ | ✅ Implemented |
| Retry policies | ⚠️ | ✅ | ⚠️ | ⚠️ | ✅ Implemented |
| Agent coordination | ✅ | ⚠️ | ✅ | ✅ | ⚠️ Partial |
| K8s-native config | ❌ | ❌ | ❌ | ❌ | ✅ Unique |
| MCP tool integration | ❌ | ❌ | ❌ | ❌ | ✅ Unique |

## Example Use Cases

### 1. Runbook Execution

Convert runbooks to automated workflows:

```yaml
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: database-failover

spec:
  entrypoint: check-health

  steps:
    - name: check-health
      type: agent
      agent: db-health-checker
      next:
        - condition: "state.primary_healthy == true"
          target: complete-healthy
        - target: initiate-failover

    - name: initiate-failover
      type: approval
      config:
        approvers: [dba-team]
        timeout: 5m
      next:
        - condition: approved
          target: pre-failover-checks
        - target: escalate

    - name: pre-failover-checks
      type: agent
      agent: failover-validator
      validation:
        - type: function
          name: check_replica_lag
          max_lag_seconds: 10
      next: execute-failover

    - name: execute-failover
      type: agent
      agent: failover-executor
      next: post-failover-verify

    - name: post-failover-verify
      type: agent
      agent: failover-verifier
      next:
        - condition: "state.failover_successful"
          target: complete
        - target: rollback
```

### 2. CI/CD Pipeline

```yaml
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: deploy-pipeline

spec:
  steps:
    - name: build
      type: agent
      agent: build-agent
      next: test

    - name: test
      type: parallel
      branches:
        - name: unit-tests
          steps: [{ agent: unit-test-agent }]
        - name: integration-tests
          steps: [{ agent: integration-test-agent }]
        - name: security-scan
          steps: [{ agent: security-agent }]
      join:
        strategy: all
      next:
        - condition: "state.all_passed"
          target: staging-deploy
        - target: notify-failure

    - name: staging-deploy
      type: agent
      agent: deploy-agent
      next: staging-approval

    - name: staging-approval
      type: approval
      config:
        approvers: [qa-team]
        auto_approve:
          condition: "state.environment == 'dev'"
      next:
        - condition: approved
          target: production-deploy
        - target: complete

    - name: production-deploy
      type: agent
      agent: deploy-agent
      next: complete
```

## See Also

- [CLI Reference](../user/CLI_REFERENCE.md)
- [MCP Configuration](../user/MCP_CONFIGURATION.md)
- [Architecture](./ARCHITECTURE.md)
