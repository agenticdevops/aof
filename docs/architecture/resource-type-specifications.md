# Resource Type Specifications

Complete specification of all resource types in aofctl with their schemas, behaviors, and API contracts.

## Core Resources (aof.io/v1alpha1)

### 1. Agent

**Description**: AI agent execution unit supporting multiple LLM providers.

**Resource Names**:
- Plural: `agents`
- Singular: `agent`
- Short: `ag`
- Kind: `Agent`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`, `watch`, `run`, `exec`, `logs`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Agent
metadata:
  name: string              # Required, RFC 1123 subdomain
  namespace: string         # Optional, defaults to current context
  labels:
    key: value              # Optional, key-value pairs
  annotations:
    key: value              # Optional
spec:
  provider: string          # Required: openai|anthropic|google|ollama|custom
  model: string             # Required: model identifier
  systemPrompt: string      # Optional: system prompt
  temperature: float        # Optional: 0.0-2.0, default 0.7
  maxTokens: int            # Optional: max output tokens
  topP: float               # Optional: 0.0-1.0
  frequencyPenalty: float   # Optional: -2.0 to 2.0
  presencePenalty: float    # Optional: -2.0 to 2.0
  tools:                    # Optional: list of MCP tools
    - string
  memory:
    enabled: bool           # Optional: default false
    type: string            # Optional: vector|conversational|summary
    maxMessages: int        # Optional: default 100
  resources:
    limits:
      tokens: int           # Optional: token budget limit
      cost: float           # Optional: dollar cost limit
      time: duration        # Optional: execution time limit
  parallelism: int          # Optional: concurrent execution limit
status:
  phase: string             # Pending|Running|Succeeded|Failed|Unknown
  conditions:
    - type: string
      status: string        # True|False|Unknown
      lastTransitionTime: timestamp
      reason: string
      message: string
  startTime: timestamp
  completionTime: timestamp
  tokensUsed: int
  cost: float
  lastExecution:
    timestamp: timestamp
    duration: duration
    result: string
```

**Status Phases**:
- `Pending`: Agent created, awaiting initialization
- `Running`: Agent actively processing
- `Succeeded`: Execution completed successfully
- `Failed`: Execution failed
- `Unknown`: Status cannot be determined

**Condition Types**:
- `Ready`: Agent is ready to accept requests
- `Initialized`: Agent initialization complete
- `ToolsAvailable`: All required tools are accessible
- `MemoryReady`: Memory system initialized

**Example**:
```bash
aofctl run agent code-reviewer \
  --provider=anthropic \
  --model=claude-3-sonnet-20240229 \
  --system-prompt="Review code for quality and security" \
  --tools=mcp__github__pr_review,mcp__code__analyze
```

---

### 2. Workflow

**Description**: Multi-step orchestrated agent execution pipeline.

**Resource Names**:
- Plural: `workflows`
- Singular: `workflow`
- Short: `wf`
- Kind: `Workflow`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`, `watch`, `run`, `logs`, `rollout`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Workflow
metadata:
  name: string
  namespace: string
spec:
  steps:
    - name: string          # Required: step name
      agentRef:
        name: string        # Required: reference to agent
        namespace: string   # Optional
      inputs:               # Optional: step inputs
        key: value
      outputs:              # Optional: step outputs to pass
        - key: string
          value: string
      retryPolicy:
        maxAttempts: int    # Optional: default 1
        backoff: duration   # Optional: retry backoff
      timeout: duration     # Optional: step timeout
      when:                 # Optional: conditional execution
        condition: string   # CEL expression
  dependencies:             # Optional: step dependencies
    - from: string          # Step name
      to: string            # Step name
      type: string          # Optional: sequential|parallel|conditional
  parallelism: int          # Optional: max parallel steps
  timeout: duration         # Optional: workflow timeout
  retryPolicy:
    maxAttempts: int
    backoff: duration
  failurePolicy: string     # Optional: stop|continue|rollback
status:
  phase: string             # Pending|Running|Succeeded|Failed|Paused
  startTime: timestamp
  completionTime: timestamp
  currentStep: string
  stepStatuses:
    - stepName: string
      phase: string
      startTime: timestamp
      completionTime: timestamp
      attempts: int
      message: string
  conditions:
    - type: string
      status: string
      lastTransitionTime: timestamp
```

**Example**:
```bash
aofctl apply -f - <<EOF
apiVersion: aof.io/v1alpha1
kind: Workflow
metadata:
  name: code-review-pipeline
spec:
  steps:
    - name: analyze
      agentRef:
        name: code-analyzer
      timeout: 5m
    - name: review
      agentRef:
        name: code-reviewer
      timeout: 10m
    - name: suggest
      agentRef:
        name: improvement-suggester
  dependencies:
    - from: analyze
      to: review
    - from: review
      to: suggest
EOF
```

---

### 3. Tool

**Description**: MCP tool integration and configuration.

**Resource Names**:
- Plural: `tools`
- Singular: `tool`
- Short: `tl`
- Kind: `Tool`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`, `watch`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Tool
metadata:
  name: string
  namespace: string
spec:
  type: string              # Required: mcp|function|custom
  mcpServer:                # Required if type=mcp
    name: string            # MCP server name
    command: string         # MCP server command
    args: [string]          # Optional: command arguments
    env:                    # Optional: environment variables
      key: value
  function:                 # Required if type=function
    name: string            # Function identifier
    description: string     # Function description
    parameters:             # JSON Schema for parameters
      type: object
      properties: {}
  enabled: bool             # Optional: default true
  rateLimit:                # Optional: rate limiting
    requestsPerMinute: int
    requestsPerDay: int
  timeout: duration         # Optional: execution timeout
status:
  available: bool
  lastChecked: timestamp
  version: string
  conditions:
    - type: string
      status: string
```

**Example**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Tool
metadata:
  name: github-pr-review
spec:
  type: mcp
  mcpServer:
    name: github
    command: npx
    args: ["@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
  rateLimit:
    requestsPerMinute: 10
```

---

### 4. Model

**Description**: LLM model configuration and provider settings.

**Resource Names**:
- Plural: `models`
- Singular: `model`
- Short: `mdl`
- Kind: `Model`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Model
metadata:
  name: string
spec:
  provider: string          # Required: openai|anthropic|google|ollama
  modelId: string           # Required: provider-specific model ID
  apiVersion: string        # Optional: API version
  baseUrl: string           # Optional: custom base URL
  credentials:
    secretRef:
      name: string          # Reference to secret
      key: string           # Key in secret
  defaults:                 # Optional: default parameters
    temperature: float
    maxTokens: int
    topP: float
  pricing:                  # Optional: cost tracking
    inputTokens: float      # Cost per 1K input tokens
    outputTokens: float     # Cost per 1K output tokens
  capabilities:             # Optional: model capabilities
    - vision
    - functionCalling
    - streaming
status:
  available: bool
  lastChecked: timestamp
  latency: duration         # Average latency
  tokensProcessed: int
  totalCost: float
```

---

### 5. Prompt

**Description**: Reusable prompt template with variable substitution.

**Resource Names**:
- Plural: `prompts`
- Singular: `prompt`
- Short: `pt`
- Kind: `Prompt`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Prompt
metadata:
  name: string
spec:
  template: string          # Required: prompt template with {{vars}}
  variables:                # Optional: variable definitions
    - name: string
      type: string          # string|number|boolean|json
      required: bool
      default: any
      description: string
  examples:                 # Optional: few-shot examples
    - input: string
      output: string
  metadata:
    category: string        # Optional: categorization
    tags: [string]          # Optional: tags
    version: string         # Optional: semantic version
status:
  usageCount: int
  lastUsed: timestamp
```

**Example**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Prompt
metadata:
  name: code-review-prompt
spec:
  template: |
    Review the following {{language}} code for:
    - Code quality
    - Security issues
    - Performance concerns

    Code:
    ```{{language}}
    {{code}}
    ```
  variables:
    - name: language
      type: string
      required: true
    - name: code
      type: string
      required: true
```

---

### 6. Memory

**Description**: Agent memory store for context and learning.

**Resource Names**:
- Plural: `memories`
- Singular: `memory`
- Short: `mem`
- Kind: `Memory`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Memory
metadata:
  name: string
spec:
  type: string              # Required: vector|conversational|summary
  provider: string          # Optional: for vector stores
  maxSize: int              # Optional: max entries
  ttl: duration             # Optional: time-to-live for entries
  vectorConfig:             # Required if type=vector
    dimensions: int
    indexType: string       # hnsw|flat|ivf
    similarityMetric: string # cosine|euclidean|dotProduct
  compression: bool         # Optional: compress stored data
status:
  entryCount: int
  sizeBytes: int
  lastAccessed: timestamp
```

---

### 7. Session

**Description**: Agent execution session with history and context.

**Resource Names**:
- Plural: `sessions`
- Singular: `session`
- Short: `sess`
- Kind: `Session`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `delete`, `logs`, `exec`

**Schema**:
```yaml
apiVersion: aof.io/v1alpha1
kind: Session
metadata:
  name: string
spec:
  agentRef:
    name: string
    namespace: string
  interactive: bool         # Optional: interactive session
  persistent: bool          # Optional: persist after completion
  timeout: duration         # Optional: session timeout
status:
  phase: string             # Active|Completed|Failed|Timeout
  startTime: timestamp
  lastActivity: timestamp
  messageCount: int
  tokensUsed: int
  cost: float
```

---

## Configuration Resources (aof.io/v1)

### 8. Config

**Description**: Configuration object for agents and workflows.

**Resource Names**:
- Plural: `configs`
- Singular: `config`
- Short: `cfg`
- Kind: `Config`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1
kind: Config
metadata:
  name: string
data:
  key: value                # String key-value pairs
immutable: bool             # Optional: prevent updates
```

---

### 9. Secret

**Description**: Sensitive configuration data (API keys, tokens).

**Resource Names**:
- Plural: `secrets`
- Singular: `secret`
- Short: `sec`
- Kind: `Secret`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1
kind: Secret
metadata:
  name: string
type: string                # Opaque|api-key|tls
data:
  key: base64-encoded-value # Base64 encoded values
stringData:                 # Optional: plain-text for input
  key: value
```

---

### 10. Provider

**Description**: LLM provider configuration (cluster-scoped).

**Resource Names**:
- Plural: `providers`
- Singular: `provider`
- Short: `pv`
- Kind: `Provider`

**Namespaced**: No

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: aof.io/v1
kind: Provider
metadata:
  name: string              # openai|anthropic|google|ollama
spec:
  type: string              # Provider type
  baseUrl: string           # API base URL
  apiVersion: string        # API version
  defaultModel: string      # Default model to use
  credentials:
    secretRef:
      name: string
      namespace: string
  rateLimit:
    requestsPerMinute: int
    requestsPerDay: int
  timeout: duration
status:
  available: bool
  modelsAvailable: [string]
  lastChecked: timestamp
```

---

## Runtime Resources (apps.aof.io/v1)

### 11. Deployment

**Description**: Managed agent deployment with scaling and rolling updates.

**Resource Names**:
- Plural: `deployments`
- Singular: `deployment`
- Short: `deploy`
- Kind: `Deployment`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`, `rollout`, `scale`

**Schema**:
```yaml
apiVersion: apps.aof.io/v1
kind: Deployment
metadata:
  name: string
spec:
  replicas: int             # Desired number of agent instances
  selector:
    matchLabels:
      key: value
  template:
    metadata:
      labels:
        key: value
    spec:
      # Agent spec
  strategy:
    type: string            # RollingUpdate|Recreate
    rollingUpdate:
      maxUnavailable: int
      maxSurge: int
  revisionHistoryLimit: int
status:
  replicas: int
  readyReplicas: int
  availableReplicas: int
  updatedReplicas: int
  conditions:
    - type: string
      status: string
```

---

### 12. Job

**Description**: One-time agent execution job.

**Resource Names**:
- Plural: `jobs`
- Singular: `job`
- Short: `job`
- Kind: `Job`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `delete`, `logs`

**Schema**:
```yaml
apiVersion: batch.aof.io/v1
kind: Job
metadata:
  name: string
spec:
  template:
    spec:
      # Agent spec
  completions: int          # Number of successful completions
  parallelism: int          # Number of parallel executions
  backoffLimit: int         # Number of retries
  activeDeadlineSeconds: int # Timeout
status:
  active: int               # Running instances
  succeeded: int
  failed: int
  startTime: timestamp
  completionTime: timestamp
```

---

### 13. CronJob

**Description**: Scheduled agent execution.

**Resource Names**:
- Plural: `cronjobs`
- Singular: `cronjob`
- Short: `cj`
- Kind: `CronJob`

**Namespaced**: Yes

**Verbs**: `create`, `get`, `list`, `update`, `patch`, `delete`

**Schema**:
```yaml
apiVersion: batch.aof.io/v1
kind: CronJob
metadata:
  name: string
spec:
  schedule: string          # Cron expression
  timeZone: string          # Optional: IANA timezone
  jobTemplate:
    spec:
      # Job spec
  concurrencyPolicy: string # Allow|Forbid|Replace
  suspend: bool             # Suspend scheduling
  successfulJobsHistoryLimit: int
  failedJobsHistoryLimit: int
status:
  lastScheduleTime: timestamp
  lastSuccessfulTime: timestamp
  active: [jobReference]
```

---

*This specification provides the complete API contract for all aofctl resource types following kubectl conventions.*
