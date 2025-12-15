---
sidebar_position: 2
---

# AOF API Resources Reference

Comprehensive reference for all resource types available in the Agent Orchestration Framework.

## Overview

This document describes all resource types (API objects) that can be created and managed with aofctl.

View all available resources:
```bash
aofctl api-resources
```

---

## API Groups

Resources are organized into logical API groups:

- **v1** - Core AOF resources (Agents, Workflows, Tools, Config, Secret)
- **apps/v1** - Application management (Deployment)
- **batch/v1** - Job management (Job, CronJob)
- **storage/v1** - Storage resources (PersistentVolume, PersistentVolumeClaim)
- **mcp/v1** - MCP tool integration (MCPTool)

---

## Core Resources (v1)

### Agent

Represents an AI agent that can execute tasks autonomously.

**Kind:** Agent
**API Version:** v1
**Namespaced:** Yes
**Short Names:** ag

**Description:**
An Agent is a containerized AI entity with a specific model, instructions, and tools. Agents can process input and generate output based on their configuration.

**Common Verbs:**
- create, get, list, delete, describe, run, logs, exec

**YAML Example:**
```yaml
apiVersion: v1
kind: Agent
metadata:
  name: data-analyzer
  namespace: default
  labels:
    app: analytics
    version: "1.0"
  annotations:
    description: "Analyzes data and provides insights"
spec:
  # Model configuration
  model: claude-sonnet-4-5-20250929
  provider: anthropic

  # Agent instructions
  instructions: |
    You are a data analysis expert. Your job is to:
    1. Analyze provided data
    2. Identify patterns and anomalies
    3. Provide actionable insights

  # Available tools
  tools:
    - name: search
      source: google-search
    - name: sql
      source: database

  # Memory configuration
  memory:
    type: short-term
    size: 5000
    ttl: 3600

  # Resource limits
  resources:
    requests:
      cpu: "100m"
      memory: "256Mi"
    limits:
      cpu: "1"
      memory: "1Gi"

  # Auto-scaling configuration
  autoscaling:
    enabled: true
    minReplicas: 1
    maxReplicas: 5
    targetCPU: 80

status:
  phase: Running
  replicas: 1
  updatedReplicas: 1
  readyReplicas: 1
  conditions:
    - type: Ready
      status: "True"
      reason: "AgentReady"
      message: "Agent is running"
    - type: Available
      status: "True"
      message: "Agent is available"
  lastUpdateTime: "2025-12-11T14:49:02Z"
```

**Fields:**
- `spec.model` (string, required) - LLM model to use
- `spec.provider` (string) - Model provider (anthropic, openai, etc.)
- `spec.instructions` (string) - System instructions for the agent
- `spec.tools` (array) - Available tools
- `spec.memory` (object) - Memory configuration
- `spec.resources` (object) - Resource limits and requests
- `spec.autoscaling` (object) - Auto-scaling configuration

**Examples:**
```bash
# Create agent from file
aofctl apply -f agent.yaml

# Get all agents
aofctl get agents

# Get specific agent
aofctl get agent data-analyzer

# Run agent
aofctl run agent data-analyzer --input "analyze sales.csv"

# Show agent details
aofctl describe agent data-analyzer

# View agent logs
aofctl logs agent data-analyzer --follow

# Delete agent
aofctl delete agent data-analyzer
```

---

### Workflow

Represents a multi-step orchestration of agents and tools.

**Kind:** Workflow
**API Version:** v1
**Namespaced:** Yes
**Short Names:** wf, workflow

**Description:**
A Workflow is a DAG (Directed Acyclic Graph) of steps that execute sequentially or in parallel. Each step can be an Agent, Tool, or another Workflow.

**Common Verbs:**
- create, get, list, delete, describe, run

**YAML Example:**
```yaml
apiVersion: v1
kind: Workflow
metadata:
  name: data-pipeline
  namespace: default
spec:
  description: "Multi-step data processing workflow"

  steps:
    # Step 1: Gather data
    - name: gather
      type: agent
      resource:
        name: data-collector
      input:
        source: "database"
        query: "SELECT * FROM events"

    # Step 2: Analyze data (depends on gather)
    - name: analyze
      type: agent
      resource:
        name: data-analyzer
      dependsOn:
        - gather
      input:
        data: "{{ steps.gather.output }}"

    # Step 3: Generate report (depends on analyze)
    - name: report
      type: agent
      resource:
        name: report-generator
      dependsOn:
        - analyze
      input:
        analysis: "{{ steps.analyze.output }}"
      output:
        format: "pdf"

  # Timeout for entire workflow
  timeout: 3600

  # Failure handling
  onFailure:
    action: "notify"
    channels:
      - slack
      - email

status:
  phase: Running
  startTime: "2025-12-11T14:49:02Z"
  completionTime: null
  stepStatus:
    - name: gather
      phase: Succeeded
      endTime: "2025-12-11T14:52:00Z"
    - name: analyze
      phase: Running
      startTime: "2025-12-11T14:52:00Z"
    - name: report
      phase: Pending
  conditions:
    - type: Ready
      status: "True"
```

**Fields:**
- `spec.steps` (array, required) - Workflow steps
- `spec.timeout` (integer) - Timeout in seconds
- `spec.onFailure` (object) - Failure handling strategy

**Examples:**
```bash
# List workflows
aofctl get workflows

# Get specific workflow
aofctl get workflow data-pipeline

# Run workflow
aofctl run workflow data-pipeline --input "config.json"

# Describe workflow
aofctl describe workflow data-pipeline

# Delete workflow
aofctl delete workflow data-pipeline
```

---

### Tool

Represents an available MCP tool that can be used by agents.

**Kind:** Tool
**API Version:** mcp/v1
**Namespaced:** No
**Short Names:** tl

**Description:**
A Tool is a callable MCP server endpoint. Tools provide specific functionality that agents can invoke.

**Examples:**
```bash
# List all tools
aofctl get tools

# Get specific tool
aofctl get tool google-search

# View tool details
aofctl describe tool google-search
```

---

### Config

Configuration objects for application settings.

**Kind:** Config
**API Version:** v1
**Namespaced:** Yes
**Short Names:** cfg

**Description:**
Configuration data that can be referenced by agents and workflows.

**YAML Example:**
```yaml
apiVersion: v1
kind: Config
metadata:
  name: app-config
  namespace: default
data:
  database_url: "postgresql://localhost:5432/mydb"
  api_timeout: "30"
  max_retries: "3"
  log_level: "info"
```

**Examples:**
```bash
# Apply config
aofctl apply -f config.yaml

# List configs
aofctl get configs

# Get config data
aofctl get config app-config -o yaml

# Delete config
aofctl delete config app-config
```

---

### Secret

Sensitive data storage for credentials and secrets.

**Kind:** Secret
**API Version:** v1
**Namespaced:** Yes
**Short Names:** sec

**Description:**
Secrets store sensitive information like API keys, passwords, and tokens.

**YAML Example:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-credentials
  namespace: default
type: Opaque
data:
  api_key: <base64-encoded-key>
  api_secret: <base64-encoded-secret>
```

**Examples:**
```bash
# Apply secret (values should be base64 encoded)
aofctl apply -f secret.yaml

# List secrets
aofctl get secrets

# Delete secret
aofctl delete secret api-credentials
```

---

## Application Resources (apps/v1)

### Deployment

Managed deployment of agents with replicas and updates.

**Kind:** Deployment
**API Version:** apps/v1
**Namespaced:** Yes
**Short Names:** deploy

**Description:**
A Deployment describes the desired state of agents including replica count, update strategy, and scaling.

**YAML Example:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: analyzer-deployment
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: analyzer
  template:
    metadata:
      labels:
        app: analyzer
    spec:
      agent: data-analyzer
      model: claude-sonnet-4-5-20250929

  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0

status:
  replicas: 3
  updatedReplicas: 3
  readyReplicas: 3
  availableReplicas: 3
```

---

## Job Resources (batch/v1)

### Job

One-time execution of an agent or workflow.

**Kind:** Job
**API Version:** batch/v1
**Namespaced:** Yes

**Description:**
A Job runs an agent or workflow to completion, then stops.

**Examples:**
```bash
# List jobs
aofctl get jobs

# Get job details
aofctl get job my-job -o yaml

# View job logs
aofctl logs job my-job

# Delete job
aofctl delete job my-job
```

---

### CronJob

Scheduled execution at specified times.

**Kind:** CronJob
**API Version:** batch/v1
**Namespaced:** Yes
**Short Names:** cj

**Description:**
A CronJob schedules agents or workflows to run at specified intervals.

**YAML Example:**
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-report
spec:
  schedule: "0 9 * * *"  # 9 AM every day
  jobTemplate:
    spec:
      workflow: report-generator
      input:
        date: "{{ now | date 'YYYY-MM-DD' }}"
```

---

## Storage Resources (storage/v1)

### PersistentVolume

Storage volume for persistent data.

**Kind:** PersistentVolume
**API Version:** storage/v1
**Namespaced:** No

### PersistentVolumeClaim

Claim for storage from agents.

**Kind:** PersistentVolumeClaim
**API Version:** storage/v1
**Namespaced:** Yes

---

## Resource Discovery

### List All Resources

```bash
# Show all available resource types
aofctl api-resources

# Show with additional columns
aofctl api-resources -o wide

# Filter by namespace capability
aofctl api-resources --namespaced=true

# Filter by API group
aofctl api-resources --api-group=v1
```

### Resource Information

```bash
# Get resource short names and API versions
aofctl api-resources

# Example output:
# NAME          SHORTNAMES    APIVERSION    NAMESPACED   KIND
# agents        ag            v1            true         Agent
# workflows     wf,workflow   v1            true         Workflow
# tools         tl            mcp/v1        false        Tool
# jobs                        batch/v1      true         Job
# cronjobs      cj            batch/v1      true         CronJob
```

---

## Common Field Reference

### metadata

Common metadata for all resources:

```yaml
metadata:
  name: resource-name              # Required: Resource name
  namespace: default               # Namespace (default: "default")
  labels:                          # Key-value labels
    key1: value1
    key2: value2
  annotations:                     # Annotations (metadata)
    key: value
  created: 2025-12-11T14:49:02Z   # Created timestamp
  uid: abc-123-def-456             # Unique identifier
```

### status

Common status fields:

```yaml
status:
  phase: Running                    # Current phase
  conditions:                       # Current conditions
    - type: Ready
      status: "True"
      reason: "AgentReady"
      message: "Agent is ready"
  lastUpdateTime: 2025-12-11T14:49:02Z
```

---

## API Versioning

Resources use semantic versioning:
- **v1** - Stable, generally available
- **v1alpha1** - Alpha, subject to change
- **v1beta1** - Beta, mostly stable

---

## Further Reading

- [Complete CLI Reference](./aofctl-complete.md)
- [Agent Specification](./agent-spec.md)
- [Workflow Tutorial](../tutorials/first-agent.md)

