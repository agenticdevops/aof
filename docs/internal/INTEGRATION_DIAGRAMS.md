# AOF Integration Architecture - Visual Diagrams

## 1. Crate Dependency Graph

```mermaid
graph TB
    subgraph "Workspace Root"
        A[aof-core<br/>Traits & Types]
        B[aof-tools<br/>Tool Implementations]
        C[aof-llm<br/>LLM Providers]
        D[aof-mcp<br/>MCP Client]
        E[aof-memory<br/>State Management]
        F[aof-runtime<br/>Execution Engine]
        G[aofctl<br/>CLI Binary]
    end

    B --> A
    C --> A
    D --> A
    E --> A
    F --> A
    F --> B
    F --> C
    F --> D
    F --> E
    G --> F

    style A fill:#e1f5ff
    style B fill:#fff4e1
    style F fill:#ffe1f5
    style G fill:#e1ffe1
```

## 2. Tool Implementation Layer

```mermaid
graph LR
    subgraph "aof-tools/src/tools"
        A[observability.rs<br/>Prometheus, Loki, ES]
        B[grafana.rs<br/>Grafana API]
        C[datadog.rs<br/>Reference Impl]
        D[newrelic.rs<br/>NEW]
        E[splunk.rs<br/>NEW]
        F[servicenow.rs<br/>NEW]
    end

    subgraph "Feature Flags"
        O[observability]
        S[siem]
        I[itsm]
    end

    subgraph "Common Utilities"
        U[common.rs<br/>Helpers]
    end

    O -.-> A
    O -.-> B
    O -.-> C
    O -.-> D
    S -.-> E
    I -.-> F

    A --> U
    B --> U
    C --> U
    D --> U
    E --> U
    F --> U

    style D fill:#90EE90
    style E fill:#90EE90
    style F fill:#90EE90
```

## 3. Tool Trait Implementation Flow

```mermaid
sequenceDiagram
    participant LLM as LLM Model
    participant Agent as Agent Executor
    participant Registry as Tool Registry
    participant Tool as NewRelicQueryTool
    participant API as New Relic API

    LLM->>Agent: tool_call("newrelic_nrql_query", {...})
    Agent->>Registry: execute_tool("newrelic_nrql_query", input)
    Registry->>Tool: execute(ToolInput)

    Tool->>Tool: Extract arguments
    Tool->>Tool: Validate inputs
    Tool->>Tool: Create HTTP client
    Tool->>API: POST /v1/accounts/{id}/query
    API-->>Tool: JSON response
    Tool->>Tool: Parse response
    Tool-->>Registry: ToolResult
    Registry-->>Agent: ToolResult
    Agent-->>LLM: Tool result in message
```

## 4. Integration Architecture Layers

```mermaid
graph TB
    subgraph "User Layer"
        YAML[Agent YAML Config]
        CLI[aofctl CLI]
    end

    subgraph "Runtime Layer"
        Runtime[AgentExecutor]
        Fleet[FleetOrchestrator]
    end

    subgraph "Tool Execution Layer"
        Registry[ToolRegistry]
        Builtin[BuiltinToolExecutor]
        MCP[McpToolExecutor]
    end

    subgraph "Tool Implementation Layer"
        NewRelic[NewRelicTools]
        Splunk[SplunkTools]
        ServiceNow[ServiceNowTools]
        Datadog[DatadogTools]
    end

    subgraph "External APIs"
        NRAPI[New Relic API]
        SplunkAPI[Splunk REST API]
        SNOWAPI[ServiceNow Table API]
    end

    YAML --> CLI
    CLI --> Runtime
    CLI --> Fleet
    Runtime --> Registry
    Fleet --> Registry
    Registry --> Builtin
    Registry --> MCP
    Builtin --> NewRelic
    Builtin --> Splunk
    Builtin --> ServiceNow
    Builtin --> Datadog

    NewRelic --> NRAPI
    Splunk --> SplunkAPI
    ServiceNow --> SNOWAPI

    style NewRelic fill:#90EE90
    style Splunk fill:#90EE90
    style ServiceNow fill:#90EE90
```

## 5. Tiered Fleet Execution (RCA Example)

```mermaid
graph TB
    subgraph "Tier 1: Data Collectors"
        NR[NewRelic Agent<br/>gemini-2.0-flash<br/>NRQL queries]
        SP[Splunk Agent<br/>gemini-2.0-flash<br/>SPL searches]
        K8[K8s Agent<br/>gemini-2.0-flash<br/>kubectl logs]
    end

    subgraph "Tier 2: Reasoners"
        C1[Claude Analyzer<br/>claude-sonnet-4<br/>weight: 2.0]
        G1[Gemini Analyzer<br/>gemini-2.5-pro<br/>weight: 1.0]
    end

    subgraph "Tier 3: Coordinator"
        M[Manager Agent<br/>gemini-2.5-flash<br/>Synthesize + Create Ticket]
    end

    subgraph "External Systems"
        NRAPI[New Relic API]
        SplunkAPI[Splunk API]
        K8sAPI[Kubernetes API]
        SNOWAPI[ServiceNow API]
    end

    NR -->|metrics data| NRAPI
    SP -->|log events| SplunkAPI
    K8 -->|pod logs| K8sAPI

    NR -->|Tier 1 Results| C1
    NR -->|Tier 1 Results| G1
    SP -->|Tier 1 Results| C1
    SP -->|Tier 1 Results| G1
    K8 -->|Tier 1 Results| C1
    K8 -->|Tier 1 Results| G1

    C1 -->|Weighted Consensus| M
    G1 -->|Weighted Consensus| M

    M -->|Create Incident| SNOWAPI

    style NR fill:#FFE4B5
    style SP fill:#FFE4B5
    style C1 fill:#ADD8E6
    style G1 fill:#ADD8E6
    style M fill:#90EE90
```

## 6. Tool Registration Flow

```mermaid
graph LR
    subgraph "Tool Definition"
        T1[NewRelicNRQLQueryTool]
        T2[NewRelicAlertTool]
        T3[NewRelicMetricTool]
    end

    subgraph "Collection"
        Coll[NewRelicTools::all]
    end

    subgraph "Registration"
        Reg[ToolRegistry]
    end

    subgraph "Executor"
        Exec[BuiltinToolExecutor]
    end

    T1 --> Coll
    T2 --> Coll
    T3 --> Coll
    Coll --> Reg
    Reg --> Exec

    style Coll fill:#FFE4B5
    style Reg fill:#ADD8E6
    style Exec fill:#90EE90
```

## 7. Authentication Flow

```mermaid
sequenceDiagram
    participant User as User YAML Config
    participant Agent as Agent Runtime
    participant Context as Context Manager
    participant Tool as NewRelicQueryTool
    participant Client as HTTP Client
    participant API as New Relic API

    User->>Agent: Load agent config
    Agent->>Context: Resolve secrets
    Context->>Context: Load from env/vault/etc
    Agent->>Tool: execute(input)
    Tool->>Tool: Extract api_key from input
    Tool->>Client: create_newrelic_client(api_key)
    Client->>Client: Set X-Api-Key header
    Client->>API: POST with auth header
    API-->>Client: 200 OK + data
    Client-->>Tool: Response
    Tool-->>Agent: ToolResult
```

## 8. Splunk Async Search Pattern

```mermaid
stateDiagram-v2
    [*] --> CreateJob: POST /search/jobs
    CreateJob --> PollStatus: GET /search/jobs/{sid}

    PollStatus --> PollStatus: isDone=false (retry)
    PollStatus --> GetResults: isDone=true
    PollStatus --> Timeout: max_attempts reached

    GetResults --> [*]: Return results
    Timeout --> [*]: Error
```

## 9. Data Flow: Incident Response Pipeline

```mermaid
graph TB
    subgraph "Detection"
        A[New Relic Alert<br/>CPU > 90%]
    end

    subgraph "Investigation (Tier 1)"
        B[New Relic Agent<br/>Query metrics]
        C[Splunk Agent<br/>Search error logs]
        D[K8s Agent<br/>Get pod status]
    end

    subgraph "Analysis (Tier 2)"
        E[Claude Analyzer<br/>Correlate signals]
        F[Gemini Analyzer<br/>Find patterns]
    end

    subgraph "Consensus"
        G[Weighted Voting<br/>Claude: 2.0, Gemini: 1.0]
    end

    subgraph "Response (Tier 3)"
        H[Manager Agent<br/>Synthesize RCA]
        I[ServiceNow Agent<br/>Create incident]
    end

    subgraph "Output"
        J[ServiceNow Ticket<br/>INC0012345]
    end

    A --> B
    A --> C
    A --> D

    B --> E
    B --> F
    C --> E
    C --> F
    D --> E
    D --> F

    E --> G
    F --> G

    G --> H
    H --> I
    I --> J

    style A fill:#FF6B6B
    style G fill:#4ECDC4
    style J fill:#95E1D3
```

## 10. Tool Configuration Schema

```mermaid
classDiagram
    class ToolConfig {
        +String name
        +String description
        +JsonValue parameters
        +ToolType tool_type
        +u64 timeout_secs
        +HashMap extra
    }

    class ToolInput {
        +JsonValue arguments
        +Option~HashMap~ context
        +get_arg~T~(key) T
    }

    class ToolResult {
        +bool success
        +JsonValue data
        +Option~String~ error
        +u64 execution_time_ms
        +with_execution_time(ms) Self
    }

    class Tool {
        <<trait>>
        +execute(input) ToolResult
        +config() ToolConfig
        +validate_input(input) Result
        +definition() ToolDefinition
    }

    class NewRelicQueryTool {
        -ToolConfig config
        +new() Self
        +execute(input) ToolResult
        +config() ToolConfig
    }

    Tool <|.. NewRelicQueryTool
    NewRelicQueryTool --> ToolConfig
    NewRelicQueryTool --> ToolInput
    NewRelicQueryTool --> ToolResult
```

## 11. Feature Flag Dependencies

```mermaid
graph TB
    subgraph "Cargo Features"
        ALL[all]
        OBS[observability]
        SIEM[siem]
        ITSM[itsm]
    end

    subgraph "Dependencies"
        REQ[reqwest]
        CHRONO[chrono]
        BASE64[base64]
        URL[urlencoding]
    end

    subgraph "Tool Modules"
        NR[newrelic.rs]
        SP[splunk.rs]
        SN[servicenow.rs]
        DD[datadog.rs]
    end

    ALL --> OBS
    ALL --> SIEM
    ALL --> ITSM

    OBS --> REQ
    OBS --> CHRONO
    SIEM --> REQ
    SIEM --> CHRONO
    ITSM --> REQ
    ITSM --> CHRONO
    ITSM --> BASE64
    ITSM --> URL

    OBS -.-> NR
    OBS -.-> DD
    SIEM -.-> SP
    ITSM -.-> SN

    style NR fill:#90EE90
    style SP fill:#90EE90
    style SN fill:#90EE90
```

## 12. Error Handling Flow

```mermaid
graph TB
    Start[Tool Execute] --> Extract[Extract Arguments]
    Extract --> Validate{Validation OK?}

    Validate -->|No| Err1[ToolResult::error<br/>Invalid arguments]
    Validate -->|Yes| CreateClient[Create HTTP Client]

    CreateClient --> ClientOK{Client OK?}
    ClientOK -->|No| Err2[ToolResult::error<br/>Auth failed]
    ClientOK -->|Yes| SendReq[Send HTTP Request]

    SendReq --> ReqOK{Request OK?}
    ReqOK -->|No| Err3[ToolResult::error<br/>Network error]
    ReqOK -->|Yes| ParseResp[Parse Response]

    ParseResp --> StatusOK{Status 200?}
    StatusOK -->|No| Err4[ToolResult::error<br/>API error + status]
    StatusOK -->|Yes| Success[ToolResult::success<br/>with data]

    style Success fill:#90EE90
    style Err1 fill:#FF6B6B
    style Err2 fill:#FF6B6B
    style Err3 fill:#FF6B6B
    style Err4 fill:#FF6B6B
```

## 13. Module Organization

```
aof-tools/
├── Cargo.toml
│   └── [features]
│       ├── observability = ["reqwest", "chrono"]
│       ├── siem = ["reqwest", "chrono"]
│       └── itsm = ["reqwest", "chrono", "base64"]
│
├── src/
│   ├── lib.rs
│   │   └── pub use tools::{NewRelicTools, SplunkTools, ServiceNowTools}
│   │
│   ├── registry.rs
│   │   ├── ToolRegistry
│   │   ├── BuiltinToolExecutor
│   │   └── CompositeToolExecutor
│   │
│   └── tools/
│       ├── mod.rs
│       │   ├── #[cfg(feature = "observability")]
│       │   ├── #[cfg(feature = "siem")]
│       │   └── #[cfg(feature = "itsm")]
│       │
│       ├── common.rs
│       │   ├── create_schema()
│       │   ├── tool_config()
│       │   └── execute_command()
│       │
│       ├── observability.rs
│       ├── datadog.rs (REFERENCE)
│       ├── newrelic.rs (NEW)
│       ├── splunk.rs (NEW)
│       └── servicenow.rs (NEW)
│
└── tests/
    ├── newrelic_tests.rs
    ├── splunk_tests.rs
    └── servicenow_tests.rs
```

## 14. Deployment Architecture

```mermaid
graph TB
    subgraph "Development"
        Dev[Developer]
        Code[Write Tool Code]
        Test[Unit Tests]
    end

    subgraph "Build"
        Cargo[cargo build --release]
        Features[--features observability,siem,itsm]
    end

    subgraph "Binary"
        Binary[aofctl]
        Tools[Built-in Tools<br/>NewRelic, Splunk, ServiceNow]
    end

    subgraph "Runtime"
        Agent[Agent Runtime]
        Registry[Tool Registry]
    end

    subgraph "External APIs"
        NR[New Relic]
        SP[Splunk]
        SN[ServiceNow]
    end

    Dev --> Code
    Code --> Test
    Test --> Cargo
    Cargo --> Features
    Features --> Binary
    Binary --> Tools
    Tools --> Agent
    Agent --> Registry
    Registry --> NR
    Registry --> SP
    Registry --> SN

    style Binary fill:#90EE90
    style Tools fill:#FFE4B5
```

---

## Diagram Usage Guide

- **Diagram 1-2:** Understand overall crate structure and module organization
- **Diagram 3:** Learn how tools are invoked at runtime
- **Diagram 4:** See the full stack from YAML to API calls
- **Diagram 5:** Understand multi-tier fleet execution pattern
- **Diagram 6-7:** Tool registration and authentication flows
- **Diagram 8:** Splunk's unique async search pattern
- **Diagram 9:** Complete incident response data flow
- **Diagram 10:** Core data structures and relationships
- **Diagram 11:** Feature flag dependencies for compilation
- **Diagram 12:** Error handling patterns to implement
- **Diagram 13:** File organization reference
- **Diagram 14:** Build and deployment architecture

**These diagrams complement the written architecture analysis in `INTEGRATION_ARCHITECTURE.md`.**
