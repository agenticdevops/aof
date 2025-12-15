# aofctl Command Structure - Visual Architecture

## High-Level Command Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Input                                  │
│                  aofctl [VERB] [RESOURCE] [NAME] [FLAGS]            │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Command Parser & Validator                     │
├─────────────────────────────────────────────────────────────────────┤
│  • Parse verb, resource type, name, flags                          │
│  • Validate command structure                                      │
│  • Apply defaults from context                                     │
│  • Infer resource type if needed                                   │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                 ┌───────────────┼───────────────┐
                 │               │               │
                 ▼               ▼               ▼
     ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
     │  Verb        │  │  Resource    │  │  Flag        │
     │  Handler     │  │  Registry    │  │  Processor   │
     └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
            │                 │                  │
            └────────┬────────┴────────┬─────────┘
                     │                 │
                     ▼                 ▼
          ┌─────────────────────────────────┐
          │     Resource Manager            │
          │  • CRUD operations              │
          │  • Lifecycle management         │
          │  • Status tracking              │
          │  • Event generation             │
          └────────────┬────────────────────┘
                       │
                       ▼
          ┌─────────────────────────────────┐
          │     Database Layer (SQLite)     │
          │  • Persistence                  │
          │  • Transactions                 │
          │  • Queries                      │
          └────────────┬────────────────────┘
                       │
                       ▼
          ┌─────────────────────────────────┐
          │     Output Formatter            │
          │  • JSON / YAML / Table          │
          │  • Custom columns               │
          │  • Watch / streaming            │
          └────────────┬────────────────────┘
                       │
                       ▼
          ┌─────────────────────────────────┐
          │     User Output                 │
          └─────────────────────────────────┘
```

## Command Verb Routing

```
aofctl [VERB] ...
    │
    ├── create ──────────► CreateHandler
    │                       │
    │                       ├─ Parse resource definition (YAML/JSON)
    │                       ├─ Validate schema
    │                       ├─ Check for existing resource
    │                       ├─ Create in database
    │                       └─ Emit creation event
    │
    ├── run ─────────────► RunHandler
    │                       │
    │                       ├─ Parse inline spec
    │                       ├─ Create resource
    │                       ├─ Initialize runtime
    │                       └─ Start execution
    │
    ├── get ─────────────► GetHandler
    │                       │
    │                       ├─ Parse selectors (label/field)
    │                       ├─ Query database
    │                       ├─ Filter results
    │                       └─ Format output
    │
    ├── describe ────────► DescribeHandler
    │                       │
    │                       ├─ Query resource
    │                       ├─ Fetch related resources
    │                       ├─ Get events
    │                       └─ Format detailed view
    │
    ├── delete ──────────► DeleteHandler
    │                       │
    │                       ├─ Parse resource selectors
    │                       ├─ Validate deletion
    │                       ├─ Execute deletion
    │                       └─ Emit deletion event
    │
    ├── apply ───────────► ApplyHandler
    │                       │
    │                       ├─ Parse resource definition
    │                       ├─ Check if exists
    │                       ├─ Create or update
    │                       └─ Emit event
    │
    ├── logs ────────────► LogsHandler
    │                       │
    │                       ├─ Infer resource type
    │                       ├─ Find resource
    │                       ├─ Stream logs
    │                       └─ Format output
    │
    ├── exec ────────────► ExecHandler
    │                       │
    │                       ├─ Find running resource
    │                       ├─ Establish connection
    │                       ├─ Execute command
    │                       └─ Stream I/O
    │
    └── [other verbs] ───► [respective handlers]
```

## Resource Type Resolution

```
Input: aofctl get [RESOURCE_TYPE] [NAME]
            │
            ▼
┌─────────────────────────────────┐
│   Resource Registry Lookup      │
├─────────────────────────────────┤
│  Check:                         │
│  1. Full name (agents)          │
│  2. Short name (ag)             │
│  3. Singular (agent)            │
│  4. Case-insensitive match      │
└────────────┬────────────────────┘
             │
             ├─ Found ──────────────► Continue with resource type
             │
             └─ Not Found ──────────► Try resource inference
                                       │
                                       ├─ Check name prefix
                                       ├─ Check common patterns
                                       └─ Error: unknown resource
```

## API Resources Organization

```
API Groups
│
├── aof.io/v1alpha1 (Core Resources)
│   ├── agents
│   ├── workflows
│   ├── tools
│   ├── models
│   ├── prompts
│   ├── memories
│   └── sessions
│
├── aof.io/v1 (Stable Core)
│   ├── configs
│   ├── secrets
│   ├── configmaps
│   └── namespaces
│
├── apps.aof.io/v1 (Application Resources)
│   ├── deployments
│   ├── replicasets
│   └── daemonsets
│
├── batch.aof.io/v1 (Batch Processing)
│   ├── jobs
│   ├── cronjobs
│   └── tasks
│
├── storage.aof.io/v1 (Storage Resources)
│   ├── artifacts
│   ├── datasets
│   └── embeddings
│
└── observability.aof.io/v1 (Observability)
    ├── events
    ├── metrics
    ├── traces
    └── logs
```

## Resource Lifecycle State Machine

```
┌─────────────┐
│   Pending   │ ◄──────────────────┐
└──────┬──────┘                    │
       │                           │
       │ (validation passed)       │
       ▼                           │
┌─────────────┐                    │
│ Initializing│                    │
└──────┬──────┘                    │
       │                           │
       │ (resources allocated)     │
       ▼                           │
┌─────────────┐                    │
│   Running   │                    │
└──────┬──────┘                    │
       │                           │
       ├──────► Success ───────────┤
       │                           │
       ├──────► Failed ────────────┤
       │                           │
       └──────► Terminated ────────┘
                    ▲
                    │
              (user requested)
                    │
              ┌──────────┐
              │ Deleting │
              └──────────┘
```

## Namespace Hierarchy

```
Cluster
│
├── Namespace: default
│   ├── agents/
│   ├── workflows/
│   └── configs/
│
├── Namespace: production
│   ├── agents/
│   │   ├── code-reviewer
│   │   └── qa-assistant
│   ├── workflows/
│   │   └── review-pipeline
│   └── secrets/
│       └── anthropic-key
│
└── Namespace: development
    ├── agents/
    └── workflows/
```

## Output Formatter Pipeline

```
Resource Data
    │
    ▼
┌─────────────────────────────────┐
│   Serialization                 │
│   • Convert to common format    │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Output Format Selection       │
├─────────────────────────────────┤
│  --output flag:                 │
│  • json                         │
│  • yaml                         │
│  • wide                         │
│  • name                         │
│  • custom-columns               │
│  • jsonpath                     │
└────────────┬────────────────────┘
             │
             ├─► json ───────────► JSON Formatter
             │                      └─► Pretty print with jq-style
             │
             ├─► yaml ───────────► YAML Formatter
             │                      └─► Clean YAML with proper indent
             │
             ├─► wide ───────────► Table Formatter (extended)
             │                      └─► Show more columns
             │
             ├─► name ───────────► Name-Only Formatter
             │                      └─► resource-type/name per line
             │
             └─► custom-columns ─► Custom Column Formatter
                                    └─► User-defined column template
```

## Label & Field Selector Processing

```
Input: aofctl get agents -l app=reviewer,env=prod --field-selector status.phase=Running

         ┌────────────────────────────────┐
         │   Parse Selectors              │
         ├────────────────────────────────┤
         │  Label Selector:               │
         │    app=reviewer                │
         │    env=prod                    │
         │                                │
         │  Field Selector:               │
         │    status.phase=Running        │
         └────────────┬───────────────────┘
                      │
                      ▼
         ┌────────────────────────────────┐
         │   Build Query                  │
         ├────────────────────────────────┤
         │  WHERE labels LIKE '%app:reviewer%'│
         │    AND labels LIKE '%env:prod%'    │
         │    AND status->>'phase' = 'Running'│
         └────────────┬───────────────────┘
                      │
                      ▼
         ┌────────────────────────────────┐
         │   Execute Query                │
         │   Filter Results               │
         └────────────┬───────────────────┘
                      │
                      ▼
                   [Results]
```

## Event System

```
Resource Action
    │
    ▼
┌─────────────────────────────────┐
│   Event Generator               │
├─────────────────────────────────┤
│  • Action: Created/Updated/     │
│            Deleted/Failed        │
│  • Timestamp                    │
│  • Resource reference           │
│  • Reason & Message             │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Event Store                   │
│   (events table)                │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Event Consumers               │
├─────────────────────────────────┤
│  • aofctl events [resource]     │
│  • aofctl describe [resource]   │
│  • Audit logging                │
│  • Metrics collection           │
└─────────────────────────────────┘
```

## Configuration Context Management

```
~/.aof/config.yaml
│
├── contexts:
│   ├── default
│   │   ├── namespace: default
│   │   └── database: ~/.aof/default.db
│   │
│   ├── production
│   │   ├── namespace: production
│   │   └── database: ~/.aof/prod.db
│   │
│   └── development
│       ├── namespace: development
│       └── database: ~/.aof/dev.db
│
├── current-context: default
│
└── preferences:
    ├── default-output: table
    └── editor: vim
```

## Command Completion Flow

```
User types: aofctl get ag<TAB>

    │
    ▼
┌─────────────────────────────────┐
│  Shell Completion Script        │
│  (bash/zsh/fish)                │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│  aofctl completion helper       │
├─────────────────────────────────┤
│  • Check current word           │
│  • Check previous words         │
│  • Determine context            │
│  • Query api-resources          │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│  Return completions:            │
│  • agents                       │
│  • ag (short name)              │
│  • agent (singular)             │
└─────────────────────────────────┘
```

---

*This diagram set provides a visual overview of the aofctl CLI architecture following kubectl patterns.*
