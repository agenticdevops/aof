# ADR-001: kubectl-Compatible CLI Structure for aofctl

**Status**: Proposed
**Date**: 2025-12-11
**Decision Makers**: Architecture Team
**Related**: [aofctl CLI Refactor Initiative]

## Context

The current aofctl CLI uses a noun-verb pattern (`aofctl agent run xxx`) that differs from kubectl's widely-adopted verb-noun pattern. To improve developer experience and align with Kubernetes conventions, we need to refactor the CLI to follow kubectl's command structure and resource management patterns.

## Decision

Refactor aofctl to follow kubectl's architecture with:
1. Verb-noun command structure
2. Resource-centric API design
3. Unified resource registry
4. kubectl-compatible flags and behaviors

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         aofctl CLI                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Command    │  │   Resource   │  │   Output     │        │
│  │   Parser     │──│   Registry   │──│   Formatter  │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│         │                  │                  │               │
│         ▼                  ▼                  ▼               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Verb       │  │   Resource   │  │   Serializer │        │
│  │   Handlers   │  │   Manager    │  │   (JSON/YAML)│        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│         │                  │                                  │
│         ▼                  ▼                                  │
│  ┌─────────────────────────────────────────────┐              │
│  │         Core API (Database Layer)           │              │
│  └─────────────────────────────────────────────┘              │
│                         │                                     │
└─────────────────────────┼─────────────────────────────────────┘
                          ▼
                  ┌──────────────┐
                  │   SQLite DB  │
                  └──────────────┘
```

## Command Structure

### Verb-Noun Pattern

```
aofctl [VERB] [RESOURCE_TYPE] [NAME] [flags]
```

**Examples**:
- `aofctl run agent my-agent`
- `aofctl get agents`
- `aofctl describe agent my-agent`
- `aofctl delete workflow my-workflow`
- `aofctl logs agent my-agent`

### Special Commands (No Resource Type)

```
aofctl [VERB] [NAME] [flags]
```

**Examples**:
- `aofctl logs my-agent` (infer resource type)
- `aofctl exec my-agent -- command`
- `aofctl apply -f config.yaml`

## Command Categories

### 1. **Basic Commands** (Cluster/Resource Management)
```
create      Create a resource from file or stdin
run         Run a new resource (agent, workflow, etc.)
get         Display one or many resources
describe    Show detailed information about a resource
delete      Delete resources
apply       Apply configuration from file
```

### 2. **Deployment Commands** (Resource Lifecycle)
```
rollout     Manage rollouts of resources
scale       Scale resources (agents, workflows)
autoscale   Auto-scale resources based on metrics
```

### 3. **Cluster Management Commands**
```
drain       Drain agent pool for maintenance
cordon      Mark agent as unschedulable
uncordon    Mark agent as schedulable
top         Display resource usage (CPU, memory, tokens)
```

### 4. **Troubleshooting and Debugging**
```
logs        Print logs from a resource
exec        Execute command in a running agent
attach      Attach to a running agent process
port-forward Forward ports to a resource
debug       Debug resources (interactive mode)
events      List events for resources
```

### 5. **Advanced Commands**
```
diff        Diff live version against applied version
patch       Update fields of a resource
replace     Replace a resource
wait        Wait for a condition on resources
```

### 6. **Settings Commands**
```
config      Modify aofctl configuration
version     Print version information
api-resources Print available API resources
api-versions  Print available API versions
```

## Resource Types

### Core Resources

| Resource Type | Short Name | Namespaced | Description |
|--------------|------------|------------|-------------|
| agents | ag | Yes | AI agents (OpenAI, Anthropic, Gemini, etc.) |
| workflows | wf | Yes | Multi-step agent workflows |
| tools | tl | Yes | MCP tools and integrations |
| models | mdl | Yes | LLM model configurations |
| prompts | pt | Yes | Prompt templates |
| memories | mem | Yes | Agent memory stores |
| sessions | sess | Yes | Agent execution sessions |

### Configuration Resources

| Resource Type | Short Name | Namespaced | Description |
|--------------|------------|------------|-------------|
| configs | cfg | Yes | Configuration objects |
| secrets | sec | Yes | Sensitive configuration data |
| configmaps | cm | Yes | Non-sensitive configuration |
| providers | pv | No | LLM provider configurations |

### Runtime Resources

| Resource Type | Short Name | Namespaced | Description |
|--------------|------------|------------|-------------|
| deployments | deploy | Yes | Managed agent deployments |
| jobs | job | Yes | One-time agent executions |
| cronjobs | cj | Yes | Scheduled agent executions |
| tasks | task | Yes | Individual agent tasks |

### Storage Resources

| Resource Type | Short Name | Namespaced | Description |
|--------------|------------|------------|-------------|
| artifacts | art | Yes | Agent output artifacts |
| datasets | ds | Yes | Training/fine-tuning datasets |
| embeddings | emb | Yes | Vector embeddings |

### Observability Resources

| Resource Type | Short Name | Namespaced | Description |
|--------------|------------|------------|-------------|
| events | ev | Yes | System events |
| metrics | met | No | Performance metrics |
| traces | tr | Yes | Execution traces |
| logs | log | Yes | Structured logs |

## API Resources Registry

```rust
pub struct ApiResource {
    pub name: String,
    pub short_names: Vec<String>,
    pub api_version: String,
    pub namespaced: bool,
    pub kind: String,
    pub verbs: Vec<String>,
    pub categories: Vec<String>,
}

pub struct ApiResourceList {
    pub groups: Vec<ApiGroup>,
}

pub struct ApiGroup {
    pub name: String,
    pub versions: Vec<String>,
    pub preferred_version: String,
    pub resources: Vec<ApiResource>,
}
```

### Example Output of `aofctl api-resources`

```
NAME          SHORTNAMES   APIVERSION         NAMESPACED   KIND
agents        ag           aof.io/v1alpha1    true         Agent
workflows     wf           aof.io/v1alpha1    true         Workflow
tools         tl           aof.io/v1alpha1    true         Tool
models        mdl          aof.io/v1alpha1    true         Model
prompts       pt           aof.io/v1alpha1    true         Prompt
memories      mem          aof.io/v1alpha1    true         Memory
configs       cfg          aof.io/v1          true         Config
secrets       sec          aof.io/v1          true         Secret
providers     pv           aof.io/v1          false        Provider
deployments   deploy       apps.aof.io/v1     true         Deployment
jobs          job          batch.aof.io/v1    true         Job
cronjobs      cj           batch.aof.io/v1    true         CronJob
```

## Flag Conventions

### Global Flags (All Commands)

```bash
--namespace, -n      # Specify namespace (default: current context)
--context            # Use specific context from config
--output, -o         # Output format: json|yaml|wide|name|custom-columns
--verbose, -v        # Verbose output (levels 0-9)
--dry-run            # Preview changes without applying
--help, -h           # Show help
```

### Resource Selection Flags

```bash
--selector, -l       # Label selector (e.g., -l app=agent,tier=backend)
--field-selector     # Field selector (e.g., status.phase=Running)
--all-namespaces, -A # List resources across all namespaces
--all                # Select all resources
```

### Output Control Flags

```bash
--output-watch, -w   # Watch for changes
--no-headers         # Don't print headers
--show-labels        # Show all labels
--show-kind          # Show resource kind in output
--sort-by            # Sort by field (e.g., .metadata.createdAt)
```

### File/Input Flags

```bash
--filename, -f       # File or directory containing resources
--recursive, -R      # Process directory recursively
--stdin              # Read from stdin
--kustomize, -k      # Process kustomize directory
```

## Command Tree

```
aofctl
├── Basic Commands
│   ├── create [resource] [name] [flags]
│   ├── run [resource] [name] [flags]
│   ├── get [resource] [name] [flags]
│   ├── describe [resource] [name] [flags]
│   ├── delete [resource] [name] [flags]
│   └── apply -f [file] [flags]
│
├── Deployment Commands
│   ├── rollout
│   │   ├── status [resource] [name]
│   │   ├── history [resource] [name]
│   │   ├── undo [resource] [name]
│   │   └── restart [resource] [name]
│   ├── scale [resource] [name] --replicas=[n]
│   └── autoscale [resource] [name] --min=[n] --max=[n]
│
├── Cluster Management
│   ├── drain [agent-pool] [name]
│   ├── cordon [agent] [name]
│   ├── uncordon [agent] [name]
│   └── top
│       ├── agents [name]
│       ├── workflows [name]
│       └── pools
│
├── Troubleshooting
│   ├── logs [resource] [name] [flags]
│   ├── exec [resource] [name] -- [command]
│   ├── attach [resource] [name]
│   ├── port-forward [resource] [name] [ports]
│   ├── debug [resource] [name]
│   └── events [resource] [name]
│
├── Advanced Commands
│   ├── diff -f [file]
│   ├── patch [resource] [name] [flags]
│   ├── replace -f [file]
│   └── wait [resource] [name] --for=[condition]
│
└── Settings Commands
    ├── config
    │   ├── view
    │   ├── set-context
    │   ├── use-context
    │   ├── set-credentials
    │   └── get-contexts
    ├── version
    ├── api-resources
    └── api-versions
```

## Data Structures

### Resource Metadata (Common)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub name: String,
    pub namespace: Option<String>,
    pub uid: Uuid,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub generation: i64,
    pub resource_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMeta {
    pub api_version: String,
    pub kind: String,
}
```

### Resource Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource<T> {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: ResourceMetadata,
    pub spec: T,
    pub status: Option<ResourceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub phase: String,
    pub conditions: Vec<Condition>,
    pub message: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub type_: String,
    pub status: String,
    pub last_transition_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub message: Option<String>,
}
```

### Agent Resource Example

```yaml
apiVersion: aof.io/v1alpha1
kind: Agent
metadata:
  name: code-reviewer
  namespace: default
  labels:
    app: reviewer
    tier: backend
spec:
  provider: anthropic
  model: claude-3-sonnet-20240229
  systemPrompt: "You are a code reviewer..."
  tools:
    - mcp__github__pr_review
    - mcp__code__analyze
  memory:
    enabled: true
    type: vector
  resources:
    limits:
      tokens: 100000
      cost: 1.00
status:
  phase: Running
  conditions:
    - type: Ready
      status: "True"
      lastTransitionTime: "2025-12-11T10:00:00Z"
```

## Command Behavior Patterns

### Resource Inference

When resource type is omitted, infer from name or context:

```bash
aofctl logs my-agent          # Infer: agent
aofctl exec workflow-123      # Infer: workflow
aofctl delete cfg-production  # Infer: config (from prefix)
```

### Multiple Resources

Support kubectl-style multiple resource operations:

```bash
aofctl get agents,workflows                    # Multiple types
aofctl delete agent/agent-1 workflow/wf-1      # Multiple resources
aofctl get all -n production                   # All resource types
```

### Label Selectors

```bash
aofctl get agents -l app=reviewer,env=prod
aofctl delete agents -l tier=backend --dry-run
```

### Field Selectors

```bash
aofctl get agents --field-selector status.phase=Running
aofctl get workflows --field-selector spec.provider=anthropic
```

## Implementation Phases

### Phase 1: Core Command Structure
- Implement verb-noun parser
- Build resource registry
- Create base resource types
- Implement get/describe/delete commands

### Phase 2: Resource Management
- Implement create/run/apply commands
- Add file-based resource definitions (YAML/JSON)
- Build resource validation
- Add namespace support

### Phase 3: Advanced Features
- Implement logs/exec/debug commands
- Add label and field selectors
- Build output formatters
- Add watch functionality

### Phase 4: Lifecycle Management
- Implement rollout commands
- Add scaling capabilities
- Build autoscaling
- Add deployment strategies

### Phase 5: Documentation & Polish
- Generate API reference docs
- Add bash/zsh completion
- Build kubectl converter tool
- Create migration guide

## Trade-offs

### Pros
- **Familiar**: Developers already know kubectl patterns
- **Extensible**: Easy to add new resource types and verbs
- **Composable**: Commands can be chained and scripted
- **Discoverable**: `api-resources` shows all capabilities
- **Consistent**: Same patterns across all resources

### Cons
- **Breaking Change**: Existing scripts need updates
- **Migration Effort**: Users must learn new commands
- **Complexity**: More sophisticated CLI architecture
- **Implementation Time**: Requires significant refactoring

### Mitigation
- Provide kubectl-to-aofctl command translator
- Create comprehensive migration guide
- Support both old and new commands temporarily (with deprecation warnings)
- Build interactive command builder/helper

## Migration Strategy

1. **Dual Support Period** (3 months)
   - Support both old and new command structures
   - Add deprecation warnings to old commands
   - Log usage metrics to track adoption

2. **Documentation & Education**
   - Create side-by-side command comparison
   - Build interactive tutorials
   - Host migration workshops

3. **Tooling Support**
   - `aofctl convert-command` - translate old to new
   - Shell alias helper script
   - CI/CD pipeline updater tool

4. **Gradual Deprecation**
   - Month 1-2: Warnings only
   - Month 3: Require `--use-legacy` flag
   - Month 4: Remove old commands

## Success Metrics

- Command execution speed < 100ms (cold start)
- 100% kubectl flag compatibility for equivalent operations
- < 5% increase in binary size
- 90%+ user satisfaction in migration survey
- Zero breaking changes to API/database layer

## References

- [Kubernetes CLI Conventions](https://kubernetes.io/docs/reference/kubectl/conventions/)
- [kubectl Command Reference](https://kubernetes.io/docs/reference/kubectl/kubectl/)
- [Kubernetes API Concepts](https://kubernetes.io/docs/reference/using-api/api-concepts/)

## Decision

**Approved**: Proceed with kubectl-compatible CLI refactor following this architectural design.

---

*Architecture by: System Architecture Designer*
*Review Status: Awaiting Team Review*
