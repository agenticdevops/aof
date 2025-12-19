# AgentFleet YAML Reference

Complete specification for AOF AgentFleet configuration files.

## Schema Overview

```yaml
apiVersion: aof.dev/v1      # Required: API version
kind: AgentFleet            # Required: Resource type
metadata:                   # Required: Fleet metadata
  name: string              # Required: Fleet identifier
  labels: {}                # Optional: Key-value labels
  annotations: {}           # Optional: Additional metadata
spec:                       # Required: Fleet specification
  agents: []                # Required: List of agents
  coordination: {}          # Required: Coordination config
  shared: {}                # Optional: Shared resources
  communication: {}         # Optional: Communication config
```

## Full Specification

### Metadata

```yaml
metadata:
  name: my-fleet                    # Required: Unique identifier
  labels:                           # Optional: Filterable labels
    purpose: incident-response
    team: platform
    environment: production
  annotations:                      # Optional: Non-filterable metadata
    description: "Fleet for RCA analysis"
    author: "Platform Team"
    version: "1.0.0"
```

### Agent Definition

Each agent in the `spec.agents` array can be defined inline or reference an external config file.

#### Inline Agent Definition

```yaml
spec:
  agents:
    - name: security-reviewer       # Required: Agent identifier
      role: specialist              # Optional: worker|manager|specialist|validator
      tier: 2                       # Optional: Tier for tiered mode (1, 2, 3, ...)
      weight: 1.5                   # Optional: Weight for weighted consensus
      replicas: 1                   # Optional: Number of instances (default: 1)
      labels:                       # Optional: Agent-specific labels
        model-provider: anthropic
      spec:                         # Required if no 'config': Inline agent spec
        model: anthropic:claude-sonnet-4-20250514
        instructions: |
          You are a security specialist...
        tools:
          - shell
          - read_file
        max_iterations: 5
        temperature: 0.4
```

#### External Agent Reference

```yaml
spec:
  agents:
    - name: security-reviewer
      config: ./agents/security-reviewer.yaml   # Path to agent YAML file
      tier: 2
      weight: 1.5
      role: specialist
```

#### Agent Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `name` | string | Yes | - | Unique identifier for the agent |
| `config` | string | No* | - | Path to external agent YAML file |
| `spec` | object | No* | - | Inline agent specification |
| `role` | string | No | `worker` | Agent role: `worker`, `manager`, `specialist`, `validator` |
| `tier` | integer | No | - | Execution tier for tiered mode |
| `weight` | float | No | `1.0` | Vote weight for weighted consensus |
| `replicas` | integer | No | `1` | Number of agent instances |
| `labels` | object | No | - | Key-value labels for filtering |

*Either `config` or `spec` must be provided, but not both.

### Coordination Configuration

```yaml
spec:
  coordination:
    mode: peer                      # Required: Coordination mode
    distribution: round-robin       # Optional: Task distribution strategy
    consensus: {}                   # Optional: Consensus configuration
    tiered: {}                      # Optional: Tiered mode configuration
    deep: {}                        # Optional: Deep mode configuration
```

#### Coordination Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `peer` | All agents work as equals in parallel | Code review, multi-perspective analysis |
| `hierarchical` | Manager delegates to workers | Complex orchestration, incident response |
| `pipeline` | Sequential processing | Data transformation, ETL |
| `swarm` | Dynamic load-balanced coordination | High-volume task processing |
| `tiered` | Tier-based parallel execution | Multi-model RCA, staged workflows |
| `deep` | Iterative planning + execution loop | Complex investigations, RCA |

#### Peer Mode

```yaml
coordination:
  mode: peer
  distribution: round-robin
  consensus:
    algorithm: majority
    min_votes: 2
    timeout_ms: 60000
    allow_partial: true
```

#### Hierarchical Mode

```yaml
coordination:
  mode: hierarchical
  distribution: skill-based

agents:
  - name: coordinator
    role: manager      # One agent must be manager
    spec: ...

  - name: worker-1
    role: worker
    spec: ...
```

#### Pipeline Mode

```yaml
coordination:
  mode: pipeline

agents:
  # Order matters - first agent receives input, last produces output
  - name: collector
    spec: ...
  - name: analyzer
    spec: ...
  - name: reporter
    spec: ...
```

#### Swarm Mode

```yaml
coordination:
  mode: swarm
  distribution: least-loaded
```

#### Tiered Mode

```yaml
coordination:
  mode: tiered
  consensus:
    algorithm: weighted
    min_confidence: 0.6
  tiered:
    pass_all_results: true
    final_aggregation: manager_synthesis
    tier_consensus:
      "1":
        algorithm: first_wins
      "2":
        algorithm: weighted
        min_votes: 2
      "3":
        algorithm: first_wins
```

#### Deep Mode

Deep mode adds an agentic loop: planning → execution → re-planning until goal achieved.

```yaml
coordination:
  mode: deep
  deep:
    max_iterations: 10          # Safety limit
    planning: true              # Enable planning phase
    memory: true                # Persist findings across iterations
    planner_model: anthropic:claude-sonnet-4-20250514  # Optional: model for planning
    planner_prompt: |           # Optional: custom planning prompt
      Generate investigation steps to find root cause.
    synthesizer_prompt: |       # Optional: custom synthesis prompt
      Synthesize findings into a report with evidence.
```

### Distribution Strategies

```yaml
coordination:
  distribution: round-robin   # Distribution strategy
```

| Strategy | Description | Best For |
|----------|-------------|----------|
| `round-robin` | Cycle through agents sequentially | Even distribution |
| `least-loaded` | Assign to agent with fewest tasks | Load balancing |
| `random` | Random agent selection | Simple scenarios |
| `skill-based` | Match agent skills to task requirements | Specialized work |
| `sticky` | Same task type → same agent | Caching benefits |

### Consensus Configuration

```yaml
coordination:
  consensus:
    algorithm: majority         # Consensus algorithm
    min_votes: 2                # Minimum responses required
    timeout_ms: 60000           # Max wait time in milliseconds
    allow_partial: true         # Accept result if some agents fail
    min_confidence: 0.7         # Minimum confidence threshold
    weights:                    # Per-agent weights (for weighted algorithm)
      senior-reviewer: 2.0
      junior-reviewer: 1.0
```

#### Consensus Algorithms

| Algorithm | Description | Configuration |
|-----------|-------------|---------------|
| `majority` | >50% of agents must agree | `min_votes` |
| `unanimous` | 100% of agents must agree | - |
| `weighted` | Votes weighted by agent weight | `weights` or agent `weight` field |
| `first_wins` | First response wins | - |
| `human_review` | Always flags for human decision | `min_confidence` |

#### Consensus Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `algorithm` | string | No | `majority` | Consensus algorithm |
| `min_votes` | integer | No | `1` | Minimum votes required |
| `timeout_ms` | integer | No | `60000` | Maximum wait time |
| `allow_partial` | boolean | No | `true` | Accept partial consensus |
| `min_confidence` | float | No | - | Minimum confidence (0.0-1.0) |
| `weights` | object | No | - | Per-agent weight overrides |

### Tiered Configuration

For `mode: tiered`, additional configuration is available:

```yaml
coordination:
  tiered:
    pass_all_results: true            # Pass all results vs consensus winner
    final_aggregation: manager_synthesis   # How to aggregate final tier
    tier_consensus:                   # Per-tier consensus override
      "1":
        algorithm: first_wins
      "2":
        algorithm: weighted
        min_votes: 2
        min_confidence: 0.5
```

#### Tiered Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `pass_all_results` | boolean | No | `false` | Pass all results to next tier |
| `final_aggregation` | string | No | `consensus` | `consensus`, `merge`, `manager_synthesis` |
| `tier_consensus` | object | No | - | Per-tier consensus configuration |

#### Final Aggregation Strategies

| Strategy | Description |
|----------|-------------|
| `consensus` | Apply global consensus algorithm |
| `merge` | Merge all results into single output |
| `manager_synthesis` | Manager agent synthesizes final output |

### Deep Configuration

For `mode: deep`, additional configuration is available:

```yaml
coordination:
  deep:
    max_iterations: 10              # Safety limit - stop after N iterations
    planning: true                  # Enable planning phase
    memory: true                    # Persist findings across iterations
    planner_model: anthropic:claude-sonnet-4-20250514  # Override model for planning
    planner_prompt: |               # Custom planning system prompt
      Generate investigation steps to find root cause.
    synthesizer_prompt: |           # Custom synthesis system prompt
      Synthesize findings into a report with evidence.
```

#### Deep Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `max_iterations` | integer | No | `10` | Safety limit - maximum iterations before stopping |
| `planning` | boolean | No | `true` | Enable planning phase (LLM generates investigation steps) |
| `memory` | boolean | No | `true` | Persist findings across iterations |
| `planner_model` | string | No | - | Override model for planning phase |
| `planner_prompt` | string | No | - | Custom system prompt for planning phase |
| `synthesizer_prompt` | string | No | - | Custom system prompt for synthesis phase |

#### Deep Mode Workflow

1. **Plan** - LLM generates investigation steps based on the query
2. **Execute** - Each step is executed using available tools
3. **Iterate** - Continue until goal achieved or max_iterations reached
4. **Re-plan** - Adjust plan based on findings (if needed)
5. **Synthesize** - Produce final answer with evidence and recommendations

### Shared Resources

```yaml
spec:
  shared:
    memory:
      type: inmemory             # Memory backend type
      namespace: my-fleet        # Memory namespace
      ttl: 3600                  # Time-to-live in seconds
      url: redis://localhost:6379  # Connection URL (for redis/postgres)
```

#### Memory Types

| Type | Description | Configuration |
|------|-------------|---------------|
| `inmemory` | RAM-based storage | `namespace`, `ttl` |
| `redis` | Distributed cache | `url`, `namespace`, `ttl` |
| `sqlite` | Local database | `path`, `namespace` |
| `postgres` | Distributed database | `url`, `namespace` |

### Communication Configuration

```yaml
spec:
  communication:
    pattern: broadcast            # Communication pattern
    broadcast:
      channel: team-updates       # Channel name
      include_sender: false       # Include sender in broadcast
    direct:
      timeout_ms: 5000            # Direct message timeout
    pub_sub:
      topics:                     # Available topics
        - errors
        - metrics
        - alerts
```

#### Communication Patterns

| Pattern | Description | Configuration |
|---------|-------------|---------------|
| `direct` | Point-to-point messaging | `timeout_ms` |
| `broadcast` | All agents receive message | `channel`, `include_sender` |
| `pub_sub` | Topic-based messaging | `topics` |
| `request_reply` | Request-response pattern | `timeout_ms` |

## Complete Example

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-fleet
  labels:
    purpose: code-review
    team: engineering
  annotations:
    description: "Multi-perspective code review with consensus"

spec:
  agents:
    - name: security-reviewer
      role: specialist
      weight: 1.5
      spec:
        model: anthropic:claude-sonnet-4-20250514
        instructions: |
          Focus on security vulnerabilities:
          - SQL injection
          - XSS
          - Authentication flaws
        tools:
          - read_file
          - git

    - name: performance-reviewer
      role: specialist
      weight: 1.0
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on performance issues:
          - Algorithm complexity
          - Memory leaks
          - N+1 queries
        tools:
          - read_file
          - git

    - name: quality-reviewer
      role: specialist
      weight: 1.0
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on code quality:
          - SOLID principles
          - Error handling
          - Test coverage
        tools:
          - read_file
          - git

  coordination:
    mode: peer
    distribution: round-robin
    consensus:
      algorithm: weighted
      min_votes: 2
      timeout_ms: 120000
      allow_partial: true
      min_confidence: 0.6

  shared:
    memory:
      type: inmemory
      namespace: code-review
      ttl: 3600

  communication:
    pattern: broadcast
    broadcast:
      channel: review-findings
```

## Multi-Document YAML

Fleet files can contain multiple YAML documents separated by `---`:

```yaml
# Fleet definition
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: primary-fleet
spec:
  agents:
    - name: agent-1
      config: ./agent-1.yaml
  coordination:
    mode: peer

---
# Alternative simplified fleet
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: simple-fleet
spec:
  agents:
    - name: simple-agent
      spec:
        model: google:gemini-2.0-flash
        instructions: Simple instructions
  coordination:
    mode: peer
```

## Environment Variable Substitution

Fleet files support environment variable substitution:

```yaml
spec:
  shared:
    memory:
      type: redis
      url: ${REDIS_URL}          # Substituted at runtime
```

## Validation

Validate fleet configuration:

```bash
aofctl validate fleet my-fleet.yaml
```

## See Also

- [Fleet Concepts](../concepts/fleets.md) - Understanding fleet coordination
- [Agent Specification](./agent-spec.md) - Agent YAML reference
- [Multi-Model RCA Tutorial](../tutorials/multi-model-rca.md) - Step-by-step guide
