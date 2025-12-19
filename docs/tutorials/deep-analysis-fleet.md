# Deep Analysis Fleet Tutorial

Learn how to build an agentic investigation system using AOF's **deep coordination mode**. This pattern enables iterative, self-directed analysis where a planner agent creates investigation plans and worker agents execute them until the root cause is found.

## What You'll Build

A multi-agent fleet that:
- Uses **deep coordination** for iterative investigation
- Integrates **MCP servers** for filesystem and GitHub access
- Combines code analysis and infrastructure analysis
- Synthesizes findings into actionable conclusions

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Google AI API key (or other supported provider)
- Node.js (for MCP servers)
- kubectl configured (for infrastructure analysis)
- Optional: GitHub token, Prometheus/Loki endpoints

## Understanding Deep Coordination

Traditional coordination modes (hierarchical, peer, pipeline) have fixed execution patterns. **Deep coordination** is an agentic pattern where:

1. **Planner** analyzes the problem and creates an investigation plan
2. **Workers** execute investigation steps and report findings
3. **Planner** reviews findings and refines the plan
4. **Loop** continues until confidence threshold is met
5. **Synthesizer** combines all findings into conclusions

```
┌─────────────────────────────────────────────────────────────────┐
│                    Deep Coordination Flow                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────┐     ┌───────────────┐     ┌──────────────────┐    │
│  │  Task    │────▶│   Planner     │────▶│  Investigation   │    │
│  │  Input   │     │  (Manager)    │     │     Plan         │    │
│  └──────────┘     └───────────────┘     └────────┬─────────┘    │
│                          ▲                        │              │
│                          │                        ▼              │
│                   ┌──────┴──────┐     ┌──────────────────┐      │
│                   │   Refine    │     │  Worker Agents   │      │
│                   │    Plan     │     │  Execute Steps   │      │
│                   └──────┬──────┘     └────────┬─────────┘      │
│                          │                      │                │
│                          │         ┌────────────┘                │
│                          │         ▼                             │
│                   ┌──────┴──────────────┐                        │
│                   │     Findings        │                        │
│                   │   Confidence < 0.8  │───────┐                │
│                   └─────────────────────┘       │                │
│                          │                      │                │
│                          │ Confidence ≥ 0.8    │ Loop           │
│                          ▼                      │                │
│                   ┌─────────────────────┐       │                │
│                   │    Synthesizer      │◀──────┘                │
│                   │  Final Conclusions  │                        │
│                   └─────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

## Step 1: Create the Fleet Configuration

Create `deep-analysis-fleet.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: deep-analysis-fleet
  labels:
    purpose: investigation
    pattern: deep-agentic

spec:
  agents:
    # Planner - creates and refines investigation plans
    - name: planner
      role: manager
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a senior investigation planner. Your role is to:

          1. Analyze the problem statement and current findings
          2. Create a structured investigation plan with specific steps
          3. Prioritize steps based on likelihood of finding root cause
          4. Refine the plan based on worker agent findings

          Output investigation plans in this format:
          ```
          INVESTIGATION PLAN:
          1. [Step description] - [Expected outcome]
          2. [Step description] - [Expected outcome]

          PRIORITY: [high|medium|low]
          HYPOTHESIS: [Current working hypothesis]
          ```
        tools:
          - shell
        max_iterations: 5
        temperature: 0.3

    # Code analyzer with MCP filesystem access
    - name: code-analyzer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a code analysis specialist. Examine source code
          for bugs, misconfigurations, and recent changes that might
          cause the reported issue.

          Always provide specific file paths and line numbers.
        tools:
          - git
          - shell
        mcp_servers:
          - name: filesystem
            transport: stdio
            command: npx
            args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
        max_iterations: 8
        temperature: 0.2

    # Infrastructure analyzer
    - name: infra-analyzer
      role: specialist
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are an infrastructure analysis specialist. Query
          Kubernetes, logs, and metrics to identify infrastructure
          issues causing the reported problem.
        tools:
          - kubectl
          - prometheus_query
          - loki_query
        max_iterations: 8
        temperature: 0.2

    # Synthesizer - combines findings
    - name: synthesizer
      role: validator
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are an analysis synthesizer. Review all findings,
          identify patterns, determine root cause with confidence
          level, and recommend remediation steps.
        tools:
          - shell
        max_iterations: 5
        temperature: 0.3

  coordination:
    mode: deep
    manager: planner
    deep:
      max_iterations: 5
      confidence_threshold: 0.8
      context_strategy: cumulative
      investigation_agents:
        - code-analyzer
        - infra-analyzer
      synthesis_agent: synthesizer
    distribution: capability_based
```

## Step 2: Add MCP Server Integration

The code-analyzer agent uses MCP servers for enhanced capabilities:

```yaml
mcp_servers:
  # Filesystem access for reading code
  - name: filesystem
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]

  # GitHub access for PR history, issues, etc.
  - name: github
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

Install the MCP servers:

```bash
# These are installed on-demand by npx, but you can pre-install:
npm install -g @modelcontextprotocol/server-filesystem
npm install -g @modelcontextprotocol/server-github
```

## Step 3: Configure Environment

Set up required environment variables:

```bash
# Required
export GOOGLE_API_KEY="your-google-api-key"

# For infrastructure analysis
export KUBECONFIG="${HOME}/.kube/config"
export PROMETHEUS_URL="http://prometheus:9090"
export LOKI_URL="http://loki:3100"

# For GitHub MCP server (optional)
export GITHUB_TOKEN="ghp_your_github_token"
```

## Step 4: Run the Fleet

Execute the fleet with an investigation task:

```bash
# Basic usage
aofctl run fleet deep-analysis-fleet.yaml \
  --task "Analyze why the payment service is returning 500 errors"

# With verbose output
aofctl run fleet deep-analysis-fleet.yaml \
  --task "Investigate high latency in the checkout API" \
  --verbose

# With custom working directory for MCP
aofctl run fleet deep-analysis-fleet.yaml \
  --task "Find the cause of memory leaks in user-service" \
  --env WORKSPACE=/path/to/your/repo
```

## Step 5: Understanding the Output

The fleet produces structured output at each iteration:

### Iteration 1: Initial Plan
```
=== DEEP ANALYSIS: Iteration 1 ===
[Planner] Creating initial investigation plan...

INVESTIGATION PLAN:
1. Check payment-service pod logs for error patterns
2. Query Prometheus for error rate and latency metrics
3. Review recent deployments and config changes
4. Examine payment-service source for error handling

PRIORITY: high
HYPOTHESIS: None yet - gathering initial data
```

### Iteration 2: First Findings
```
=== DEEP ANALYSIS: Iteration 2 ===
[code-analyzer] Analyzing recent changes...
  Finding: PR #234 modified payment validation logic 2 days ago
  Finding: New timeout configuration in config/payment.yaml

[infra-analyzer] Checking infrastructure...
  Finding: payment-service pods showing increased memory usage
  Finding: 503 errors correlate with memory pressure

[Planner] Refining plan based on findings...

INVESTIGATION PLAN:
1. Deep dive into PR #234 changes
2. Check memory limits vs actual usage
3. Trace payment validation code path

HYPOTHESIS: Memory pressure causing request failures
CONFIDENCE: 0.6
```

### Final Synthesis
```
=== DEEP ANALYSIS: Complete ===
[Synthesizer] Combining findings...

FINDINGS SUMMARY:
- PR #234 introduced unbounded cache in PaymentValidator
- Memory usage increased 300% after deployment
- Pods hitting memory limits, triggering OOMKills
- Requests failing during garbage collection pauses

ROOT CAUSE: Memory leak in PaymentValidator cache
CONFIDENCE: high (0.92)

EVIDENCE:
- Memory growth pattern matches cache accumulation
- Errors correlate with GC pause times
- Rolling restart temporarily resolves issue

REMEDIATION:
1. Immediate: Increase memory limits to 2Gi
2. Short-term: Add cache eviction policy
3. Follow-up: Review PR #234, add cache size monitoring

PREVENTION:
- Add memory usage alerts
- Include memory profiling in CI
- Require cache size limits in code review
```

## Advanced Configuration

### Custom Investigation Agents

Add domain-specific specialists:

```yaml
agents:
  # Database specialist
  - name: db-analyzer
    role: specialist
    spec:
      model: google:gemini-2.5-flash
      instructions: |
        You are a database specialist. Analyze query performance,
        connection pools, and data consistency issues.
      mcp_servers:
        - name: postgres
          transport: stdio
          command: npx
          args: ["-y", "@modelcontextprotocol/server-postgres"]
          env:
            DATABASE_URL: "${DATABASE_URL}"
```

### Confidence Tuning

Adjust when the investigation concludes:

```yaml
coordination:
  deep:
    # Higher threshold = more thorough investigation
    confidence_threshold: 0.9

    # More iterations allowed
    max_iterations: 10

    # How context is passed between iterations
    # - cumulative: All findings accumulate
    # - windowed: Only last N findings
    # - summarized: LLM summarizes findings
    context_strategy: cumulative
```

### Multi-Model Investigation

Use different models for different roles:

```yaml
agents:
  - name: planner
    spec:
      model: google:gemini-2.5-flash  # Fast for planning
      # ...

  - name: code-analyzer
    spec:
      model: anthropic:claude-3-5-sonnet  # Strong at code
      # ...

  - name: synthesizer
    spec:
      model: openai:gpt-4o  # Good at synthesis
      # ...
```

## Best Practices

### 1. Clear Investigation Scope

Provide specific, actionable tasks:

```bash
# Good - specific and actionable
aofctl run fleet deep-analysis-fleet.yaml \
  --task "Find why checkout API returns 504 timeout errors after 10pm UTC"

# Too vague
aofctl run fleet deep-analysis-fleet.yaml \
  --task "Fix the API"
```

### 2. Appropriate Tool Access

Give agents only the tools they need:

```yaml
# Code analyzer needs file access, not kubectl
- name: code-analyzer
  spec:
    tools: [git, shell]
    mcp_servers: [filesystem]

# Infra analyzer needs kubectl, not file access
- name: infra-analyzer
  spec:
    tools: [kubectl, prometheus_query]
```

### 3. Resource Management

Set appropriate limits for long investigations:

```yaml
agents:
  - name: code-analyzer
    spec:
      max_iterations: 8  # Limit per-agent iterations
      temperature: 0.2   # Lower = more focused

coordination:
  deep:
    max_iterations: 5    # Limit total planning cycles
```

### 4. Secure MCP Configuration

Use environment variables for sensitive data:

```yaml
mcp_servers:
  - name: github
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"  # Never hardcode tokens
```

## Troubleshooting

### MCP Server Won't Start

```bash
# Verify npx works
npx -y @modelcontextprotocol/server-filesystem --version

# Check Node.js version (requires 18+)
node --version
```

### Investigation Not Converging

- Lower `confidence_threshold` (e.g., 0.7)
- Increase `max_iterations`
- Check if task is too broad - narrow the scope

### Missing Findings

- Verify tool permissions (kubectl access, file paths)
- Check environment variables are set
- Review agent logs for errors

## Complete Example

See the full example at:
- Fleet config: `examples/fleets/deep-analysis-fleet.yaml`
- Agent configs: `examples/agents/library/`

## Next Steps

- [Fleet Reference](../reference/fleet-spec.md) - Complete fleet specification
- [MCP Integration](../tools/mcp-integration.md) - Advanced MCP configuration
- [Tools Reference](../tools/builtin-tools.md) - Available built-in tools

