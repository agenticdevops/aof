# AgentFleet: Multi-Agent Coordination

AgentFleet enables multiple AI agents to work together on complex tasks. Think of it like a Kubernetes Deployment - multiple specialized pods working in parallel toward a common goal.

## Why Use Fleets in DevOps?

### The Single Agent Problem

A single AI agent, even a powerful one like Claude or GPT-4, has limitations:

- **Single perspective**: One model, one viewpoint
- **Blind spots**: May miss domain-specific issues
- **Hallucination risk**: No cross-validation
- **Single point of failure**: If it's wrong, you're wrong

### The Fleet Solution

Fleets solve these problems through **specialization** and **consensus**:

```
┌─────────────────────────────────────────────────────────────┐
│                    SINGLE AGENT                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Generalist Agent                                   │    │
│  │  - Knows security (somewhat)                        │    │
│  │  - Knows performance (somewhat)                     │    │
│  │  - Knows style (somewhat)                           │    │
│  │  - Single point of failure                          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    AGENT FLEET                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Security    │  │ Performance │  │ Quality     │         │
│  │ Specialist  │  │ Specialist  │  │ Specialist  │         │
│  │ (focused)   │  │ (focused)   │  │ (focused)   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         └────────────────┼────────────────┘                 │
│                          ▼                                  │
│                   ┌─────────────┐                           │
│                   │  CONSENSUS  │                           │
│                   └─────────────┘                           │
│                          ▼                                  │
│              Unified, validated result                      │
└─────────────────────────────────────────────────────────────┘
```

## When to Use Fleet vs Single Agent

### Use a Single Agent When:

- Task is straightforward (answer questions, write simple code)
- You need conversational memory across turns
- Cost is a primary concern
- Latency is critical (sub-second responses)

### Use a Fleet When:

| Scenario | Why Fleet Wins |
|----------|---------------|
| **Code Review** | 3 specialists catch more than 1 generalist |
| **Incident Response** | Parallel analysis of logs, metrics, traces |
| **Critical Decisions** | Consensus reduces hallucination risk |
| **Cost Optimization** | Cheap models in parallel vs expensive single model |
| **Compliance/Audit** | "3 agents agreed on this decision" |
| **Specialization** | Focused instructions → better results |

### Cost Comparison

```
Single Claude Opus:
- 1 call × $15/1M tokens
- 1 perspective
- ~30s response time

Fleet of 3 Gemini Flash:
- 3 parallel calls × $0.075/1M tokens
- 3 perspectives + consensus
- ~5s response time (parallel execution!)
```

## Coordination Modes

AOF supports five coordination modes for different use cases:

### 1. Peer Mode (Default)

All agents work as equals, executing in parallel with results aggregated via consensus.

```yaml
coordination:
  mode: peer
  distribution: round-robin
  consensus:
    algorithm: majority
    min_votes: 2
```

**Best for**: Code review, multi-perspective analysis, voting scenarios

**How it works**:
1. Task submitted to all agents simultaneously
2. Each agent executes independently (in parallel)
3. Results collected from all agents
4. Consensus algorithm determines final result

```
     ┌──────────┐
     │   Task   │
     └────┬─────┘
          │
    ┌─────┼─────┐
    ▼     ▼     ▼
┌──────┐┌──────┐┌──────┐
│Agent1││Agent2││Agent3│  (parallel)
└──┬───┘└──┬───┘└──┬───┘
   │       │       │
   └───────┼───────┘
           ▼
     ┌──────────┐
     │Consensus │
     └──────────┘
```

### 2. Hierarchical Mode

A manager agent coordinates worker agents, delegating tasks and synthesizing results.

```yaml
coordination:
  mode: hierarchical

agents:
  - name: coordinator
    role: manager
    spec:
      instructions: |
        You coordinate the team. Analyze tasks and delegate to specialists.
        Synthesize their findings into a final report.

  - name: log-analyzer
    role: worker
    spec:
      instructions: Focus only on log analysis.

  - name: metrics-analyzer
    role: worker
    spec:
      instructions: Focus only on metrics analysis.
```

**Best for**: Complex orchestration, incident response, multi-stage workflows

**How it works**:
1. Manager agent receives task
2. Manager analyzes and decides delegation strategy
3. Workers execute their assigned portions
4. Manager synthesizes final result

```
          ┌──────────┐
          │  Task    │
          └────┬─────┘
               ▼
         ┌──────────┐
         │ Manager  │
         └────┬─────┘
              │ delegates
    ┌─────────┼─────────┐
    ▼         ▼         ▼
┌────────┐┌────────┐┌────────┐
│Worker 1││Worker 2││Worker 3│
└───┬────┘└───┬────┘└───┬────┘
    │         │         │
    └─────────┼─────────┘
              ▼ results
         ┌──────────┐
         │ Manager  │
         │synthesize│
         └──────────┘
```

### 3. Pipeline Mode

Agents execute sequentially, with each agent's output becoming the next agent's input.

```yaml
coordination:
  mode: pipeline

agents:
  - name: data-collector
    spec:
      instructions: Collect and format raw data.

  - name: analyzer
    spec:
      instructions: Analyze the collected data.

  - name: reporter
    spec:
      instructions: Generate a human-readable report.
```

**Best for**: Data transformation, multi-stage processing, ETL workflows

**How it works**:
1. First agent processes original input
2. Output passed to next agent as input
3. Continues through all agents sequentially
4. Final agent produces the result

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ Agent 1  │───▶│ Agent 2  │───▶│ Agent 3  │
│(collect) │    │(analyze) │    │(report)  │
└──────────┘    └──────────┘    └──────────┘
     input ──▶ intermediate ──▶ output
```

### 4. Swarm Mode

Self-organizing, dynamic coordination with intelligent load balancing.

```yaml
coordination:
  mode: swarm
  distribution: least-loaded
```

**Best for**: High-volume task processing, load-balanced workloads

**How it works**:
1. Task arrives
2. System selects least-loaded idle agent
3. Agent processes task
4. Metrics tracked for future balancing

### 5. Tiered Mode (Multi-Model RCA)

Tier-based parallel execution with consensus at each tier. Designed for multi-model scenarios like Root Cause Analysis where cheap data collectors feed reasoning models.

```yaml
coordination:
  mode: tiered
  consensus:
    algorithm: weighted
    min_confidence: 0.6
  tiered:
    pass_all_results: true
    final_aggregation: manager_synthesis

agents:
  # Tier 1: Data Collectors (cheap models, parallel)
  - name: loki-collector
    tier: 1
    spec:
      model: google:gemini-2.0-flash  # ~$0.075/1M tokens

  - name: prometheus-collector
    tier: 1
    spec:
      model: google:gemini-2.0-flash

  # Tier 2: Reasoning Models (multi-model consensus)
  - name: claude-analyzer
    tier: 2
    weight: 1.5  # Higher weight for Claude
    spec:
      model: anthropic:claude-sonnet-4-20250514

  - name: gemini-analyzer
    tier: 2
    weight: 1.0
    spec:
      model: google:gemini-2.5-pro

  # Tier 3: Coordinator (synthesis)
  - name: rca-coordinator
    tier: 3
    role: manager
    spec:
      model: anthropic:claude-sonnet-4-20250514
```

**Best for**: Multi-model RCA, complex analysis workflows, cost-optimized multi-perspective analysis

**How it works**:
1. **Tier 1** agents execute in parallel (cheap data collection)
2. Results from Tier 1 are passed to **Tier 2** agents
3. **Tier 2** agents analyze with different LLMs (multi-model consensus)
4. **Tier 3** manager synthesizes final result

```
        ┌─────────────────────────────────────────────────┐
        │              TIERED EXECUTION                    │
        │                                                  │
        │  TIER 1: Data Collectors (parallel)             │
        │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    │
        │  │ Loki   │ │ Prom   │ │  K8s   │ │  Git   │    │
        │  │ Logs   │ │Metrics │ │ State  │ │Changes │    │
        │  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘    │
        │      └──────────┼──────────┼──────────┘         │
        │                 ▼ (collected data)              │
        │                                                  │
        │  TIER 2: Reasoning Models (multi-model)         │
        │  ┌────────────┐ ┌────────────┐ ┌────────────┐   │
        │  │  Claude    │ │  Gemini    │ │   GPT-4    │   │
        │  │ (wt: 1.5)  │ │ (wt: 1.0)  │ │ (wt: 1.0)  │   │
        │  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘   │
        │        └──────────────┼──────────────┘          │
        │                       ▼ (weighted consensus)    │
        │                                                  │
        │  TIER 3: Coordinator                            │
        │  ┌────────────────────────────────────────┐     │
        │  │           RCA Coordinator              │     │
        │  │    (synthesize final report)           │     │
        │  └────────────────────────────────────────┘     │
        │                       ▼                          │
        │               Final RCA Report                   │
        └─────────────────────────────────────────────────┘
```

**Tiered Configuration Options**:

```yaml
tiered:
  # Pass all results to next tier (vs just consensus winner)
  pass_all_results: true

  # Final tier aggregation strategy
  final_aggregation: manager_synthesis  # or: consensus, merge

  # Per-tier consensus configuration
  tier_consensus:
    "1":
      algorithm: first_wins  # Fast data collection
    "2":
      algorithm: weighted    # Multi-model consensus
      min_votes: 2
```

## Consensus Algorithms

When using **Peer** or **Tiered** modes, consensus determines how agent results are aggregated.

### Supported Algorithms

| Algorithm | Rule | Use Case |
|-----------|------|----------|
| **majority** | >50% must agree | Code review, incident triage |
| **unanimous** | 100% must agree | Critical deployments, security |
| **weighted** | Votes weighted by role | Senior > Junior reviewers |
| **first_wins** | First response wins | Time-critical scenarios |
| **human_review** | Always flag for human | High-stakes decisions |

### Algorithm Details

#### Majority
More than 50% of agents must agree on the result. Fast and tolerant of outliers.

#### Unanimous
100% of agents must agree. Use for critical decisions where false positives are costly.

#### Weighted
Each agent has a configurable weight. Senior reviewers can count more than juniors.

```yaml
agents:
  - name: senior-reviewer
    weight: 2.0  # Counts as 2 votes

  - name: junior-reviewer
    weight: 1.0  # Counts as 1 vote

consensus:
  algorithm: weighted
  weights:
    senior-reviewer: 2.0
    junior-reviewer: 1.0
```

#### FirstWins
First agent to respond wins. Use when speed matters more than consensus.

#### HumanReview
Always flags for human operator decision. Use for high-stakes scenarios.

```yaml
consensus:
  algorithm: human_review
  min_confidence: 0.9  # If confidence below this, definitely needs review
```

### Configuration

```yaml
coordination:
  mode: peer
  consensus:
    algorithm: majority       # majority, unanimous, weighted, first_wins, human_review
    min_votes: 2              # Minimum responses required
    timeout_ms: 60000         # Max wait time (60 seconds)
    allow_partial: true       # Accept result if some agents fail
    min_confidence: 0.7       # Below this, flag for human review
    weights:                  # Per-agent weights (for weighted algorithm)
      senior-reviewer: 2.0
      junior-reviewer: 1.0
```

### Example: Code Review Consensus

```
Security Agent:   "CRITICAL: SQL injection on line 42"
Performance Agent: "No SQL issues, but N+1 query problem"
Quality Agent:    "SQL injection on line 42, missing tests"

Consensus (majority):
  - SQL injection CONFIRMED (2/3 agree)
  - N+1 query flagged (1/3, noted but not critical)
  - Missing tests flagged (1/3, noted)
```

## Task Distribution Strategies

How tasks are assigned to agents (for Hierarchical and Swarm modes):

| Strategy | Description | Best For |
|----------|-------------|----------|
| **round-robin** | Cycle through agents | Even distribution |
| **least-loaded** | Agent with fewest tasks | Load balancing |
| **random** | Random selection | Simple scenarios |
| **skill-based** | Match agent skills to task | Specialized work |
| **sticky** | Same task type → same agent | Caching benefits |

```yaml
coordination:
  mode: hierarchical
  distribution: least-loaded  # or round-robin, random, skill-based, sticky
```

## Agent Roles

Agents can have defined roles that affect their behavior in the fleet:

| Role | Description | Used In |
|------|-------------|---------|
| **worker** | Regular task executor | All modes |
| **manager** | Coordinator/orchestrator | Hierarchical mode |
| **specialist** | Domain expert | Any mode |
| **validator** | Review/validation | Quality gates |

```yaml
agents:
  - name: security-expert
    role: specialist
    spec:
      instructions: You are a security specialist...

  - name: team-lead
    role: manager
    spec:
      instructions: You coordinate the team...
```

## Communication Patterns

Agents can communicate through shared memory and messaging:

### Shared Memory Types

| Type | Description | Use Case |
|------|-------------|----------|
| **in_memory** | RAM-based | Single process, testing |
| **redis** | Distributed cache | Multi-instance, real-time |
| **sqlite** | Local database | Persistent, single node |
| **postgres** | Distributed DB | Production, multi-node |

### Message Patterns

| Pattern | Description | Use Case |
|---------|-------------|----------|
| **direct** | Point-to-point | Agent-to-agent |
| **broadcast** | All agents receive | Announcements |
| **pub_sub** | Topic-based | Event-driven |
| **request_reply** | Request-response | Queries |

```yaml
communication:
  pattern: broadcast
  broadcast:
    channel: team-updates
    include_sender: false

shared:
  memory:
    type: redis
    url: redis://localhost:6379
```

## Real-World Examples

### Example 1: Code Review Fleet

Three specialists review code in parallel, with majority consensus:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: code-review-team
spec:
  agents:
    - name: security-reviewer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on security: SQL injection, XSS, authentication,
          secrets in code, dependency vulnerabilities.
        tools:
          - read_file
          - git

    - name: performance-reviewer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on performance: Big-O complexity, memory leaks,
          N+1 queries, caching opportunities.
        tools:
          - read_file
          - git

    - name: quality-reviewer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Focus on quality: SOLID principles, error handling,
          test coverage, documentation.
        tools:
          - read_file
          - git

  coordination:
    mode: peer
    distribution: round-robin
    consensus:
      algorithm: majority
      min_votes: 2
      timeout_ms: 60000
```

**Run it**:
```bash
aofctl run fleet code-review-team.yaml \
  --input "Review this PR: https://github.com/org/repo/pull/123"
```

### Example 2: Incident Response Fleet

Hierarchical coordination with a manager orchestrating specialists:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: incident-response-team
spec:
  agents:
    - name: incident-commander
      role: manager
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are the Incident Commander. When an incident arrives:
          1. Assess severity and impact
          2. Delegate investigation to specialists
          3. Coordinate response actions
          4. Synthesize findings into an incident report
        tools:
          - shell

    - name: log-investigator
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You analyze logs. Search for errors, exceptions, and anomalies.
          Report timeline of events and root cause indicators.
        tools:
          - shell
          - read_file

    - name: metrics-analyst
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You analyze metrics and dashboards. Look for:
          - Resource exhaustion (CPU, memory, disk)
          - Traffic anomalies
          - Error rate spikes
        tools:
          - shell

  coordination:
    mode: hierarchical
    distribution: skill-based
```

### Example 3: Data Pipeline Fleet

Sequential processing with each stage building on the previous:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: data-pipeline
spec:
  agents:
    - name: collector
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Collect and normalize raw data from the input source.
          Output clean, structured JSON.

    - name: analyzer
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Analyze the structured data. Identify patterns, anomalies,
          and key insights. Output analysis summary.

    - name: reporter
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          Generate a human-readable report from the analysis.
          Include executive summary, key findings, and recommendations.

  coordination:
    mode: pipeline
```

## Fleet Metrics

Fleets automatically track execution metrics:

```yaml
metrics:
  total_tasks: 150
  completed_tasks: 145
  failed_tasks: 5
  avg_task_duration_ms: 2340
  active_agents: 3
  total_agents: 3
  consensus_rounds: 145  # For peer mode
```

Access via CLI:
```bash
aofctl describe fleet code-review-team
```

## Best Practices

### 1. Choose the Right Mode

| Situation | Recommended Mode |
|-----------|-----------------|
| Need multiple perspectives | Peer + Consensus |
| Complex orchestration | Hierarchical |
| Sequential processing | Pipeline |
| High-volume, uniform tasks | Swarm |

### 2. Optimize Agent Instructions

Each agent should have **focused, specific instructions**:

```yaml
# ❌ Bad: Too generic
instructions: Review the code for issues.

# ✅ Good: Focused and specific
instructions: |
  You are a SECURITY specialist. Focus ONLY on:
  - SQL injection vulnerabilities
  - XSS attack vectors
  - Authentication/authorization flaws
  - Secrets or credentials in code
  - Insecure dependencies

  Ignore performance and style issues - other agents handle those.
```

### 3. Use Appropriate Consensus

| Scenario | Algorithm |
|----------|-----------|
| General review | majority |
| Security-critical | unanimous |
| Mixed seniority | weighted |
| Time-critical | first_wins |

### 4. Set Reasonable Timeouts

```yaml
consensus:
  timeout_ms: 60000      # 60 seconds for most tasks
  allow_partial: true    # Don't fail if one agent is slow
```

### 5. Use Replicas for Scaling

```yaml
agents:
  - name: worker
    replicas: 3  # 3 instances for load balancing
```

## Summary

| Aspect | Single Agent | Fleet |
|--------|-------------|-------|
| **Perspectives** | 1 | Multiple |
| **Reliability** | Single point of failure | Consensus-validated |
| **Cost** | Can be expensive | Often cheaper (parallel cheap models) |
| **Latency** | Sequential | Parallel execution |
| **Specialization** | Generalist | Deep expertise per agent |
| **Audit Trail** | Limited | Full consensus history |

**Rule of thumb**: Use fleets for anything critical, multi-perspective, or high-volume. Use single agents for simple, conversational, or cost-sensitive tasks.

---

**Next Steps**:
- [Fleet Examples](../examples/index.md#agentfleet-examples)
- [AgentFleet YAML Reference](../reference/fleet-spec.md)
- [Multi-Model RCA Quickstart](../tutorials/multi-model-rca-quickstart.md)
