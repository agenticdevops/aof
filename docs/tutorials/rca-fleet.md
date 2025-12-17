# Tutorial: Building an RCA (Root Cause Analysis) Fleet

Learn how to build a multi-agent fleet that performs comprehensive root cause analysis for production incidents.

## What You'll Build

A fleet of specialized agents that work together to diagnose incidents:

```
┌─────────────────────────────────────────────────────────────────┐
│                    RCA Fleet Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Error     │  │ Dependency  │  │   Config    │             │
│  │  Analyzer   │  │Investigator │  │   Auditor   │             │
│  │             │  │             │  │             │             │
│  │ • Logs      │  │ • DB health │  │ • Git diff  │             │
│  │ • Traces    │  │ • API calls │  │ • Env vars  │             │
│  │ • Patterns  │  │ • Network   │  │ • Configs   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         └────────────────┼────────────────┘                     │
│                          ▼                                      │
│                   ┌─────────────┐                               │
│                   │     RCA     │                               │
│                   │ Coordinator │                               │
│                   │             │                               │
│                   │ Synthesizes │                               │
│                   │   Report    │                               │
│                   └─────────────┘                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Prerequisites

- AOF installed (`aofctl version`)
- Google API key (`export GOOGLE_API_KEY=...`)
- Basic understanding of YAML

## Step 1: Understand the RCA Pattern

Root Cause Analysis benefits from multiple perspectives:

| Specialist | Focus Area | Why Separate Agent? |
|------------|------------|---------------------|
| Error Analyzer | Logs, traces, exceptions | Deep pattern matching |
| Dependency Investigator | External services, DBs | Network/connectivity focus |
| Config Auditor | Recent changes, settings | Change correlation |
| RCA Coordinator | Synthesis, prioritization | Big picture view |

**Why not one agent?**
- Each specialist has focused, specific instructions → better results
- Parallel execution → faster analysis
- Consensus → more reliable conclusions
- Cheaper models work well with focused tasks

## Step 2: Create the Fleet YAML

Create `rca-fleet.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: rca-team
  labels:
    purpose: incident-response
spec:
  agents:
    # Agent 1: Error Pattern Analyzer
    - name: error-analyzer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are an Error Pattern Analyzer.

          Your job:
          1. Search logs for errors and exceptions
          2. Identify the FIRST occurrence (patient zero)
          3. Find patterns in error frequency
          4. Extract key stack trace information

          Focus ONLY on error analysis. Other agents handle dependencies and config.

          Output format:
          ## Error Analysis
          - **First Error**: [timestamp and message]
          - **Error Count**: [number in timeframe]
          - **Pattern**: [what's repeating]
          - **Key Stack Frames**: [relevant code paths]
          - **Likely Component**: [where the bug is]
        tools:
          - shell
          - read_file

    # Agent 2: Dependency Investigator
    - name: dependency-checker
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a Dependency Investigator.

          Your job:
          1. Check if external services are healthy
          2. Test database connectivity
          3. Verify API endpoints respond
          4. Check network connectivity

          Use these commands:
          - curl -I <url> (HTTP health checks)
          - nc -zv <host> <port> (port connectivity)
          - ping <host> (basic connectivity)

          Focus ONLY on dependencies. Other agents handle logs and config.

          Output format:
          ## Dependency Health
          | Service | Status | Response Time | Notes |
          |---------|--------|---------------|-------|
        tools:
          - shell

    # Agent 3: Configuration Auditor
    - name: config-auditor
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a Configuration Auditor.

          Your job:
          1. Check git history for recent changes
          2. Look for config file modifications
          3. Verify environment variables
          4. Identify any suspicious settings

          Commands to use:
          - git log --oneline -20 (recent commits)
          - git diff HEAD~5 (recent changes)
          - env | grep -i <app> (environment)

          Focus ONLY on configuration. Other agents handle logs and dependencies.

          Output format:
          ## Configuration Audit
          - **Recent Changes**: [list with dates]
          - **Suspicious Settings**: [any red flags]
          - **Rollback Candidate**: [if applicable]
        tools:
          - shell
          - read_file
          - git

    # Agent 4: RCA Coordinator (Manager)
    - name: rca-coordinator
      role: manager
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are the RCA Coordinator.

          Your job:
          1. Review findings from all specialists
          2. Determine the most likely root cause
          3. Create prioritized remediation steps
          4. Write a clear incident report

          Output this exact format:

          # Incident RCA Report

          ## Summary
          [One paragraph describing what happened]

          ## Root Cause
          [The primary cause with evidence]

          ## Contributing Factors
          1. [Factor 1]
          2. [Factor 2]

          ## Immediate Actions
          - [ ] [Action 1 - do now]
          - [ ] [Action 2 - do now]

          ## Follow-up Actions
          - [ ] [Action for this week]

          ## Prevention
          - [How to prevent recurrence]
        tools:
          - shell

  # All agents run in parallel, then reach consensus
  coordination:
    mode: peer
    distribution: round-robin
    consensus:
      algorithm: majority
      min_votes: 3
      timeout_ms: 120000
      allow_partial: true
```

## Step 3: Run the Fleet

```bash
# Set your API key
export GOOGLE_API_KEY=AIza...

# Run the RCA fleet
aofctl run fleet rca-fleet.yaml \
  --input "Investigate: Users reporting 500 errors on the checkout API since 2pm"
```

## Step 4: Understanding the Output

The fleet will:

1. **Parallel Execution**: All 4 agents start simultaneously
2. **Independent Analysis**: Each focuses on their specialty
3. **Consensus**: Results are combined with majority voting
4. **Unified Report**: Coordinator synthesizes the final RCA

Example output:
```
[AGENT] Started: error-analyzer
[AGENT] Started: dependency-checker
[AGENT] Started: config-auditor
[AGENT] Started: rca-coordinator
[FLEET] Started: rca-team with 4 agents
[TASK] Submitted: abc123

... agents execute in parallel ...

[CONSENSUS] Reached for task abc123 with 4 votes
[FLEET] Stopped: rca-team

Result: {
  "response": "# Incident RCA Report\n\n## Summary\nThe checkout API began..."
}
```

## Step 5: Customize for Your Stack

### Kubernetes-Specific RCA

```yaml
# Add kubectl tool for K8s analysis
- name: pod-analyzer
  spec:
    instructions: |
      Analyze pod status using:
      - kubectl get pods -A | grep -v Running
      - kubectl describe pod <name>
      - kubectl logs <pod> --tail=100
    tools:
      - kubectl
```

### Database-Specific RCA

```yaml
# Add database analysis
- name: db-analyzer
  spec:
    instructions: |
      Analyze database health:
      - Check slow query logs
      - Verify connection pool status
      - Look for lock contention
    tools:
      - shell
```

### Cloud-Specific RCA (AWS)

```yaml
# Add AWS CLI for cloud analysis
- name: aws-analyzer
  spec:
    instructions: |
      Check AWS service health:
      - aws cloudwatch get-metric-statistics
      - aws logs filter-log-events
      - aws ecs describe-services
    tools:
      - shell  # Requires AWS CLI configured
```

## Step 6: Advanced Patterns

### Hierarchical Mode (Manager-Led)

For complex incidents where you want the coordinator to delegate:

```yaml
coordination:
  mode: hierarchical  # Manager delegates instead of parallel

agents:
  - name: incident-commander
    role: manager
    spec:
      instructions: |
        You lead the investigation.
        1. Assess the incident severity
        2. Decide which specialists to engage
        3. Coordinate the investigation
        4. Deliver the final report
```

### Weighted Consensus (Senior Reviewers)

Give more weight to experienced agents:

```yaml
consensus:
  algorithm: weighted
  weights:
    senior-analyst: 2.0    # Counts as 2 votes
    junior-analyst: 1.0    # Counts as 1 vote
```

### Pipeline Mode (Sequential Analysis)

When each step depends on the previous:

```yaml
coordination:
  mode: pipeline

agents:
  - name: data-collector      # Step 1: Gather data
  - name: pattern-analyzer    # Step 2: Find patterns
  - name: root-cause-finder   # Step 3: Identify cause
  - name: report-writer       # Step 4: Write report
```

## Complete Example: Production-Ready RCA Fleet

See the full examples in the repository:

- **Kubernetes RCA**: `examples/fleets/k8s-rca-team.yaml`
- **Application RCA**: `examples/fleets/application-rca-team.yaml`
- **Database RCA**: `examples/fleets/database-rca-team.yaml`

## Best Practices

### 1. Keep Agent Instructions Focused

```yaml
# ❌ Bad: Too broad
instructions: Investigate the incident and find the root cause.

# ✅ Good: Specific and focused
instructions: |
  You are an ERROR ANALYZER. Focus ONLY on:
  1. Log patterns and error messages
  2. Stack traces and exceptions
  3. Error frequency and timing

  DO NOT analyze configs or dependencies - other agents do that.
```

### 2. Use Appropriate Timeouts

```yaml
consensus:
  timeout_ms: 120000    # 2 minutes for standard RCA
  allow_partial: true   # Don't fail if one agent is slow
```

### 3. Add Observability

```yaml
# Future: Send RCA reports to Slack/PagerDuty
communication:
  pattern: broadcast
  broadcast:
    channel: incident-response
```

### 4. Test Incrementally

```bash
# Test individual agents first
aofctl run agent error-analyzer.yaml --input "Check /var/log/app.log for errors"

# Then test the full fleet
aofctl run fleet rca-fleet.yaml --input "Full incident analysis"
```

## Troubleshooting

### Agents Timing Out

Increase timeout or reduce scope:
```yaml
consensus:
  timeout_ms: 300000  # 5 minutes
```

### Inconsistent Results

Increase minimum votes:
```yaml
consensus:
  algorithm: unanimous  # All must agree
  # OR
  min_votes: 4         # Need all 4 agents
```

### Too Verbose Output

Add output constraints in instructions:
```yaml
instructions: |
  ...
  Keep your analysis under 500 words.
  Focus on actionable findings only.
```

## Summary

You've learned how to:

1. ✅ Design an RCA fleet with specialized agents
2. ✅ Configure peer mode with consensus
3. ✅ Run parallel investigations
4. ✅ Synthesize findings into actionable reports
5. ✅ Customize for different tech stacks

## Next Steps

- **[Fleet Concepts](../concepts/fleets.md)** - Deep dive into fleet architecture
- **[Fleet Examples](../examples/index.md)** - More fleet configurations
- **[Fleet YAML Reference](../reference/fleet-spec.md)** - Complete spec documentation

---

**Have an incident?** Run your RCA fleet and let the agents investigate while you focus on mitigation!
