# AOF Documentation

Welcome to the AOF (Agentic Ops Framework) documentation.

## Quick Links

| Document | Description |
|----------|-------------|
| [FEATURES.md](user/FEATURES.md) | Complete feature overview |
| [CLI_REFERENCE.md](user/CLI_REFERENCE.md) | Command-line interface guide |
| [tools/index.md](user/tools/index.md) | Built-in tools reference |
| [MCP_CONFIGURATION.md](user/MCP_CONFIGURATION.md) | MCP server configuration |

## Getting Started

### Installation

```bash
cargo install aofctl
```

### Quick Start

1. **Create an agent configuration**:

```yaml
# my-agent.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-helper
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a DevOps assistant.
  tools:
    - kubectl    # Unified kubectl tool
    - git        # Unified git tool
    - docker     # Unified docker tool
    - shell
```

2. **Set your API key**:

```bash
export GOOGLE_API_KEY=your-api-key
# or for OpenAI:
export OPENAI_API_KEY=your-api-key
```

3. **Run the agent**:

```bash
aofctl run agent my-agent.yaml --input "Show git status"
```

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                    AOF Runtime                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │  Agent   │  │ AgentFleet│  │ Workflow │             │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       │             │             │                     │
│       └─────────────┼─────────────┘                     │
│                     ▼                                   │
│              ┌──────────────┐                           │
│              │ Tool Executor │                          │
│              └──────┬───────┘                           │
│         ┌───────────┼───────────┐                       │
│         ▼           ▼           ▼                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│  │ Built-in │ │   MCP    │ │  Custom  │               │
│  │  Tools   │ │  Tools   │ │  Tools   │               │
│  └──────────┘ └──────────┘ └──────────┘               │
└────────────────────────────────────────────────────────┘
```

## Key Concepts

### Agents

Agents are LLM-powered assistants that can use tools to accomplish tasks.

### Tools

AOF provides **unified CLI tools** that let the LLM construct commands:

| Tool | Description |
|------|-------------|
| `kubectl` | Run any kubectl command |
| `git` | Run any git command |
| `docker` | Run any docker command |
| `terraform` | Run any terraform command |
| `aws` | Run any AWS CLI command |
| `helm` | Run any helm command |
| `shell` | General shell commands |

Plus file operations: `read_file`, `write_file`, `list_directory`, `search_files`

### AgentFleet

Multi-agent coordination with different modes:
- **Hierarchical**: Manager coordinates workers
- **Peer**: Agents as equals
- **Swarm**: Self-organizing
- **Pipeline**: Sequential processing

### Workflows

Step-based execution with checkpointing and retry logic.

### Triggers

Event-driven agent execution from Slack, Discord, Telegram, etc.

## Examples

See the `examples/` directory:

```
examples/
├── agents/           # Single agent examples
│   ├── devops-agent.yaml
│   ├── sre-agent.yaml
│   ├── terraform-agent.yaml
│   ├── aws-agent.yaml
│   └── hybrid-agent.yaml
├── fleets/           # Multi-agent examples
│   ├── k8s-rca-team.yaml
│   ├── dockerizer-team.yaml
│   ├── incident-response-team.yaml
│   ├── code-review-team.yaml
│   └── data-pipeline-team.yaml
└── workflows/        # Workflow examples
    ├── ci-cd-pipeline.yaml
    ├── incident-response.yaml
    └── simple-workflow.yaml
```

### Running Examples

```bash
# Run a single agent
aofctl run agent examples/agents/devops-agent.yaml --input "Show git status"

# Run an agent fleet
aofctl run fleet examples/fleets/k8s-rca-team.yaml --task "Analyze failing pods in production namespace"

# Run a workflow
aofctl run workflow examples/workflows/ci-cd-pipeline.yaml
```

## Support

- GitHub Issues: https://github.com/agenticdevops/aof/issues
- Documentation: https://aof.sh/docs
