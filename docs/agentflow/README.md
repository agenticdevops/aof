# AgentFlow Documentation

AgentFlow is a declarative YAML-based DAG workflow system for orchestrating AI agents and automated operations in DevOps/SRE environments. Think of it as "n8n for AI agents" - a visual, declarative way to build complex automation workflows.

## Overview

AgentFlow enables DevOps and SRE teams to:
- Build declarative workflows with AI agents as first-class citizens
- Trigger workflows from multiple sources (Slack, GitHub, PagerDuty, Cron, etc.)
- Orchestrate complex multi-agent analysis and decision-making
- Automate incident response and remediation
- Generate reports and insights with parallel agent execution
- Implement human-in-the-loop approvals and escalations

## Architecture

### Core Concepts

1. **Triggers** - Entry points that start workflow execution
   - Webhook, Slack, GitHub, PagerDuty, Cron, Kafka, SQS, Manual

2. **Nodes** - Vertices in the DAG workflow
   - **Agent Nodes**: Execute AI agents with LLM models
   - **Action Nodes**: HTTP requests, Slack messages, GitHub operations, shell commands
   - **Control Flow Nodes**: Conditions, splits, merges, loops, waits
   - **Transform Nodes**: Data manipulation with JSONPath, JQ, templates, CEL

3. **Connections** - Edges connecting nodes with optional conditions

4. **Context** - Shared state passed between nodes
   - Trigger payload
   - Node outputs
   - Variables and secrets
   - Execution metadata

## Files in This Directory

### Schema Definition
- **`schema.yaml`** - Complete AgentFlow YAML schema specification
  - All trigger types with configurations
  - All node types with properties
  - Connection definitions
  - Error handling and retry policies
  - Context and variable passing
  - Observability configuration

### Example Workflows

1. **`example-slack-agent-slack.yaml`** - Slack → Agent → Slack
   - User asks question via Slack mention
   - AI agent analyzes and answers
   - Response posted in thread
   - Demonstrates: Slack integration, agent execution, templating

2. **`example-github-pr-review.yaml`** - GitHub PR Multi-Agent Review
   - PR opened triggers workflow
   - Security agent + Code quality agent run in parallel
   - Results merged and formatted
   - Review comment posted to PR
   - Jira ticket created for critical issues
   - Demonstrates: Parallel execution, GitHub integration, conditional logic

3. **`example-incident-remediation.yaml`** - PagerDuty Incident Auto-Remediation
   - PagerDuty alert triggers diagnostic agent
   - Agent determines if auto-fixable
   - Conditional branching: auto-fix, human approval, or escalate
   - Verification agent confirms resolution
   - PagerDuty incident updated automatically
   - Demonstrates: Conditional branching, human-in-the-loop, complex decision logic

4. **`example-scheduled-report.yaml`** - Scheduled Multi-Agent Daily Report
   - Cron trigger runs daily at 9 AM
   - Five agents run in parallel:
     - Performance analysis
     - Cost analysis
     - Security posture
     - Deployment activity
     - Incident summary
   - Results aggregated and formatted
   - Executive summary generated
   - Comprehensive report posted to Slack
   - Demonstrates: Cron scheduling, parallel agents, data aggregation

## Key Features

### 1. Multi-Trigger Support

AgentFlow supports 8 trigger types:

```yaml
triggers:
  - type: Webhook       # HTTP webhooks
  - type: Slack         # Slack events
  - type: GitHub        # GitHub webhooks
  - type: PagerDuty     # PagerDuty incidents
  - type: Cron          # Time-based scheduling
  - type: Kafka         # Message queue
  - type: SQS           # AWS queue
  - type: Manual        # CLI/API invocation
```

### 2. Agent Execution

AI agents are first-class citizens in AgentFlow:

```yaml
- id: analysis-agent
  type: Agent
  config:
    agentInline:
      model: "claude-3-5-sonnet-20241022"
      systemPrompt: "You are an expert..."
      tools: [...]
    prompt:
      template: "Analyze this: {{.input}}"
    outputFormat: json
```

### 3. Parallel Execution

Split nodes enable parallel agent execution:

```yaml
- id: split-analysis
  type: Split
  config:
    strategy: all
    continueOnError: true

# Multiple agents run in parallel
- id: security-agent
  type: Agent
  # ...

- id: quality-agent
  type: Agent
  # ...
```

### 4. Conditional Logic

Condition nodes enable branching:

```yaml
- id: decision
  type: Condition
  config:
    conditions:
      - label: auto-fix
        expression: "severity == 'low' && autoFixable"
      - label: escalate
        expression: "severity == 'critical'"
```

### 5. Human-in-the-Loop

Human nodes enable manual approvals:

```yaml
- id: approval
  type: Human
  config:
    task: "Review and approve remediation"
    notification:
      type: slack
    form:
      fields:
        - name: action
          type: select
          options: ["approve", "reject", "escalate"]
    timeout: "30m"
```

### 6. Error Handling

Comprehensive error handling with retries and fallbacks:

```yaml
spec:
  config:
    retryPolicy:
      maxAttempts: 3
      backoff: exponential
      retryOn: ["timeout", "agent-error"]

  errorHandling:
    strategy: fallback
    onError:
      - nodeId: error-handler
    onFailure:
      - nodeId: escalate
```

### 7. Observability

Built-in logging, metrics, and tracing:

```yaml
observability:
  logging:
    level: info
    includeInputs: true
    redactSecrets: true
  metrics:
    enabled: true
    customMetrics:
      - name: workflow_duration_seconds
        type: histogram
  tracing:
    enabled: true
    samplingRate: 1.0
```

## Variable and Context Passing

AgentFlow provides flexible variable passing between nodes:

### Reference Syntax

1. **JSONPath** - `$ref: "$.context.nodes.node-id.output.field"`
2. **CEL** - `$cel: "context.nodes.node_id.output.field"`
3. **Template** - `$template: "{{ .context.nodes.node_id.output.field }}"`

### Context Structure

```yaml
context:
  trigger:                    # Trigger information
    id: string
    type: string
    payload: object

  flow:                       # Flow metadata
    id: string
    name: string
    startTime: string

  nodes:                      # Node execution history
    node-id:
      status: string          # pending|running|success|failed
      input: object
      output: object
      error: object
      duration: duration

  variables: object           # Global variables
  secrets: object             # Injected secrets
```

## Getting Started

### 1. Define Your Workflow

Create a YAML file following the schema:

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: AgentFlow
metadata:
  name: my-workflow
spec:
  triggers: [...]
  nodes: [...]
  connections: [...]
```

### 2. Deploy

```bash
# Deploy to AOF cluster
kubectl apply -f my-workflow.yaml

# Or use AOF CLI
aof workflow deploy my-workflow.yaml
```

### 3. Monitor

```bash
# Watch workflow execution
aof workflow logs my-workflow

# View metrics
aof workflow metrics my-workflow
```

## Best Practices

### 1. Agent Design
- Keep agent prompts focused and specific
- Use JSON output format for structured data
- Limit tool access to only what's needed
- Set appropriate timeouts and token limits

### 2. Error Handling
- Always define retry policies
- Use `continueOnError` for non-critical parallel tasks
- Implement fallback nodes for critical paths
- Add human approval for high-risk operations

### 3. Performance
- Use parallel execution (Split nodes) when possible
- Set appropriate timeouts for each node
- Limit agent max iterations
- Use caching for repeated queries

### 4. Security
- Use Kubernetes secrets for credentials
- Enable `redactSecrets` in logging
- Limit shell command execution
- Implement approval workflows for destructive actions

### 5. Observability
- Enable tracing for complex workflows
- Add custom metrics for business KPIs
- Use structured logging
- Set up alerts for workflow failures

## Use Cases

### DevOps Automation
- Automated PR reviews and security scans
- CI/CD pipeline orchestration
- Infrastructure deployment workflows
- Compliance checking and remediation

### SRE Operations
- Incident detection and auto-remediation
- Performance analysis and optimization
- Capacity planning and scaling
- On-call escalation workflows

### Reporting and Analytics
- Scheduled health reports
- Cost analysis and optimization
- Security posture assessments
- Deployment metrics and trends

### ChatOps
- Slack command handlers
- Interactive troubleshooting
- Knowledge base queries
- Team notifications

## Integration Points

### External Systems
- **Slack**: Bi-directional communication
- **GitHub**: Webhooks, API, PR automation
- **PagerDuty**: Incident management
- **Jira**: Ticket creation and tracking
- **Kubernetes**: Resource management
- **Prometheus/Datadog**: Metrics and monitoring
- **AWS**: Cost Explorer, CloudWatch, EC2, RDS

### MCP Servers
AgentFlow can integrate with MCP servers for tool execution:

```yaml
- id: mcp-action
  type: Action
  config:
    actionType: MCPTool
    mcpTool:
      serverName: "github-mcp"
      toolName: "create-issue"
      parameters:
        title: "Security issue"
```

## Advanced Features

### 1. Loop Nodes
Iterate over collections:

```yaml
- id: process-prs
  type: Loop
  config:
    iterator:
      $ref: "$.context.nodes.fetch-prs.output.pull_requests"
    itemVariable: "pr"
    parallel: true
    parallelism: 5
```

### 2. Wait Nodes
Delays and conditional waits:

```yaml
- id: wait-deploy
  type: Wait
  config:
    waitType: until
    until:
      expression: "deployment.status == 'ready'"
      checkInterval: "30s"
      timeout: "10m"
```

### 3. Transform Nodes
Data manipulation:

```yaml
- id: extract-data
  type: Transform
  config:
    transformType: jq
    jq:
      expression: ".items | map({name: .metadata.name, status: .status.phase})"
```

### 4. Merge Strategies
Combine parallel results:

```yaml
- id: merge-results
  type: Merge
  config:
    strategy: wait-all
    aggregation: custom
    customAggregation: |
      {
        "combined": context.nodes.agent1.output + context.nodes.agent2.output,
        "best": context.nodes.agent1.output.score > context.nodes.agent2.output.score ?
                context.nodes.agent1.output : context.nodes.agent2.output
      }
```

## Roadmap

Future enhancements planned:
- Visual workflow editor (web UI)
- Workflow templates marketplace
- A/B testing for agent prompts
- Workflow versioning and rollbacks
- Cost tracking and optimization
- Multi-cluster orchestration
- Integration with more tools (DataDog, Terraform, etc.)

## Contributing

See the main AOF repository for contribution guidelines.

## License

Apache 2.0

## Support

- Documentation: https://aof.agenticops.org
- Issues: https://github.com/agenticops/aof/issues
- Slack: #agentflow channel
