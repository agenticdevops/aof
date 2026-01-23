---
sidebar_position: 15
sidebar_label: Structured I/O
---

# Structured I/O (Output Schemas)

Structured I/O allows you to define expected output formats for agents, enabling type-safe workflows and better composability.

## Overview

By default, agents return free-form text responses. With Structured I/O, you can:

- Define expected output structure using JSON Schema
- Get validated, parseable responses
- Chain agents with type-safe data flow
- Auto-generate documentation from schemas

## Basic Usage

### Defining Output Schema

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: pod-analyzer
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Analyze Kubernetes pods and report their status.
    Always respond in the specified JSON format.
  output_schema:
    type: object
    properties:
      status:
        type: string
        enum: [healthy, degraded, critical, unknown]
        description: Overall pod health status
      pods_checked:
        type: integer
        description: Number of pods analyzed
      issues:
        type: array
        items:
          type: object
          properties:
            pod_name:
              type: string
            namespace:
              type: string
            severity:
              type: string
              enum: [low, medium, high, critical]
            message:
              type: string
            recommendation:
              type: string
          required: [pod_name, severity, message]
      summary:
        type: string
        description: Human-readable summary
    required: [status, pods_checked, issues, summary]
```

### Agent Response

When an agent has an `output_schema`, its response will be validated JSON:

```json
{
  "status": "degraded",
  "pods_checked": 12,
  "issues": [
    {
      "pod_name": "api-server-abc123",
      "namespace": "production",
      "severity": "high",
      "message": "Container restarted 5 times in the last hour",
      "recommendation": "Check application logs for OOM or crash errors"
    }
  ],
  "summary": "12 pods checked, 1 high-severity issue found in production namespace"
}
```

## Schema Types

### Simple Types

```yaml
output_schema:
  type: string
  description: A simple text response
```

```yaml
output_schema:
  type: number
  minimum: 0
  maximum: 100
  description: A percentage value
```

```yaml
output_schema:
  type: boolean
  description: Success indicator
```

### Object Types

```yaml
output_schema:
  type: object
  properties:
    name:
      type: string
    count:
      type: integer
    enabled:
      type: boolean
  required: [name, count]
  additionalProperties: false
```

### Array Types

```yaml
output_schema:
  type: array
  items:
    type: object
    properties:
      id: { type: string }
      value: { type: number }
  minItems: 1
  maxItems: 100
```

### Enum Types

```yaml
output_schema:
  type: string
  enum: [approved, rejected, pending, needs_review]
```

### Union Types (oneOf)

```yaml
output_schema:
  oneOf:
    - type: object
      properties:
        success: { type: boolean, const: true }
        data: { type: object }
      required: [success, data]
    - type: object
      properties:
        success: { type: boolean, const: false }
        error: { type: string }
      required: [success, error]
```

## Use Cases

### Incident Classification

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: incident-classifier
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Classify incidents by severity and category.
  output_schema:
    type: object
    properties:
      severity:
        type: string
        enum: [P1, P2, P3, P4]
      category:
        type: string
        enum: [infrastructure, application, security, network, database]
      affected_services:
        type: array
        items: { type: string }
      estimated_impact:
        type: string
      recommended_runbook:
        type: string
    required: [severity, category, affected_services]
```

### Code Review

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: code-reviewer
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Review code changes and provide structured feedback.
  output_schema:
    type: object
    properties:
      verdict:
        type: string
        enum: [approve, request_changes, comment]
      score:
        type: integer
        minimum: 1
        maximum: 10
      findings:
        type: array
        items:
          type: object
          properties:
            file:
              type: string
            line:
              type: integer
            type:
              type: string
              enum: [bug, style, performance, security, suggestion]
            message:
              type: string
          required: [file, type, message]
      summary:
        type: string
    required: [verdict, score, findings, summary]
```

### Cost Analysis

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: cost-analyzer
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Analyze cloud costs and provide optimization recommendations.
  output_schema:
    type: object
    properties:
      total_cost:
        type: number
        description: Total cost in USD
      period:
        type: string
        description: Analysis period (e.g., "last 30 days")
      breakdown:
        type: array
        items:
          type: object
          properties:
            service:
              type: string
            cost:
              type: number
            percentage:
              type: number
            trend:
              type: string
              enum: [increasing, stable, decreasing]
          required: [service, cost]
      recommendations:
        type: array
        items:
          type: object
          properties:
            action:
              type: string
            estimated_savings:
              type: number
            effort:
              type: string
              enum: [low, medium, high]
            priority:
              type: integer
              minimum: 1
              maximum: 5
          required: [action, estimated_savings]
    required: [total_cost, period, breakdown, recommendations]
```

## Schema in Flows

When using agents with output schemas in flows, the structured output is available in variables:

```yaml
apiVersion: aof.sh/v1
kind: AgentFlow
spec:
  nodes:
    - id: analyze
      type: Agent
      config:
        agent: pod-analyzer
        prompt: "Analyze pods in namespace {{namespace}}"
    - id: route
      type: Conditional
      config:
        conditions:
          - condition: "{{analyze.output.status}} == 'critical'"
            target: alert
          - condition: "{{analyze.output.status}} == 'degraded'"
            target: investigate
          - condition: "true"
            target: log
    - id: alert
      type: Slack
      config:
        channel: "#incidents"
        text: |
          🚨 Critical: {{analyze.output.summary}}
          Issues: {{analyze.output.issues | length}}
```

## Validation Behavior

### Strict Mode (Default)

By default, responses that don't match the schema will fail:

```yaml
output_schema:
  type: object
  properties:
    status: { type: string }
  required: [status]
  # additionalProperties: false  # Implicit in strict mode
```

### Lenient Mode

Allow additional properties and partial matches:

```yaml
output_schema:
  type: object
  properties:
    status: { type: string }
  additionalProperties: true
  validation_mode: lenient  # Allows missing optional fields
```

### Coercion Mode

Attempt to coerce response into schema:

```yaml
output_schema:
  type: object
  properties:
    count: { type: integer }
  validation_mode: coerce  # Will parse "42" as 42
```

## Error Handling

When validation fails, the agent will:

1. Log the validation error
2. Return the raw response with `_validation_error` field
3. Optionally retry with schema instructions (if configured)

```yaml
output_schema:
  type: object
  properties:
    status: { type: string }
  on_validation_error: retry  # Options: fail, retry, passthrough
  max_retries: 2
```

## Best Practices

### 1. Provide Clear Instructions

Include schema expectations in agent instructions:

```yaml
instructions: |
  Analyze the given data and respond in JSON format with:
  - status: one of "healthy", "degraded", "critical"
  - issues: array of found problems
  - summary: brief text summary

  Always respond with valid JSON matching the output schema.
```

### 2. Use Descriptive Field Names

```yaml
# Good
properties:
  estimated_completion_time: { type: string }
  risk_assessment_score: { type: number }

# Avoid
properties:
  ect: { type: string }
  ras: { type: number }
```

### 3. Add Descriptions

```yaml
properties:
  severity:
    type: string
    enum: [P1, P2, P3, P4]
    description: |
      Incident priority level:
      - P1: Critical, immediate response required
      - P2: High, respond within 1 hour
      - P3: Medium, respond within 4 hours
      - P4: Low, respond within 24 hours
```

### 4. Start Simple

Begin with minimal schemas and expand as needed:

```yaml
# Start simple
output_schema:
  type: object
  properties:
    success: { type: boolean }
    message: { type: string }
  required: [success]

# Expand later
output_schema:
  type: object
  properties:
    success: { type: boolean }
    message: { type: string }
    data: { ... }
    metadata: { ... }
```

## Related

- [Agent Configuration](/docs/reference/agent-spec)
- [AgentFlow Variables](/docs/agentflow/variables)
- [JSON Schema Reference](https://json-schema.org/)
