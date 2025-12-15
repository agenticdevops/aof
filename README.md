# AOF - Agentic Ops Framework

> **n8n for Agentic Ops** - Build AI agents with Kubernetes-style YAML. No Python required.

AOF is a Rust-based framework that lets DevOps, SRE, and Platform engineers build and orchestrate AI agents using familiar YAML specifications and kubectl-style CLI commands.

## Why AOF?

**If you know Kubernetes, you already know how to use AOF.**

| Traditional AI Frameworks | AOF |
|--------------------------|-----|
| Write Python code (LangChain, CrewAI) | Write YAML specs |
| Learn new programming paradigms | Use kubectl-style CLI |
| Complex dependency management | Single binary, no dependencies |
| Limited tooling integration | Native MCP support + Shell/HTTP/GitHub |

## Key Features

- **🎯 YAML-First**: Define agents like K8s resources - no code required
- **🛠️ MCP Tooling**: Native Model Context Protocol support for extensible tools
- **🔀 Multi-Provider**: OpenAI, Anthropic, Ollama, Groq - switch with one line
- **📊 AgentFlow**: n8n-style visual DAG workflows for complex automation
- **🚀 Production Ready**: Built in Rust for performance and reliability
- **🔧 Ops-Native**: kubectl-style CLI that feels familiar

## Quick Install

### Option 1: Cargo (Rust users)
```bash
cargo install aofctl
```

### Option 2: Binary Download
```bash
# macOS (Apple Silicon)
curl -L https://github.com/yourusername/aof/releases/latest/download/aofctl-aarch64-apple-darwin -o aofctl
chmod +x aofctl
sudo mv aofctl /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/yourusername/aof/releases/latest/download/aofctl-x86_64-apple-darwin -o aofctl
chmod +x aofctl
sudo mv aofctl /usr/local/bin/

# Linux
curl -L https://github.com/yourusername/aof/releases/latest/download/aofctl-x86_64-unknown-linux-gnu -o aofctl
chmod +x aofctl
sudo mv aofctl /usr/local/bin/
```

## 30-Second Example

Create your first agent:

```bash
# Set your API key
export OPENAI_API_KEY=sk-...

# Create a simple agent
cat > my-agent.yaml <<EOF
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: openai:gpt-4
  instructions: |
    You are a helpful Kubernetes expert. Help users with kubectl commands,
    troubleshoot pod issues, and explain K8s concepts clearly.
  tools:
    - type: Shell
      config:
        allowed_commands: ["kubectl"]
EOF

# Run it interactively
aofctl agent run my-agent.yaml

# Chat with your agent
> How do I check if my pods are running?
```

## What Can You Build?

- **Slack Bots**: Auto-respond to incidents, answer questions
- **Incident Response**: Auto-remediation workflows with human-in-the-loop
- **PR Reviewers**: Automated code review and feedback
- **Daily Reports**: Scheduled cluster health checks and summaries
- **On-Call Assistants**: Diagnose and fix issues automatically

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         aofctl CLI                          │
│              (kubectl-style user interface)                 │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
┌───────▼────────┐  ┌────────▼────────┐  ┌────────▼────────┐
│     Agent      │  │   AgentFleet    │  │   AgentFlow     │
│  (Single AI)   │  │  (Team of AIs)  │  │  (Workflow DAG) │
└───────┬────────┘  └────────┬────────┘  └────────┬────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
┌───────▼────────┐  ┌────────▼────────┐  ┌────────▼────────┐
│  LLM Providers │  │   MCP Servers   │  │  Integrations   │
│ (OpenAI/Claude)│  │  (kubectl/git)  │  │ (Slack/PagerDuty)│
└────────────────┘  └─────────────────┘  └─────────────────┘
```

## Documentation

- **[Getting Started](docs/getting-started.md)** - 5-minute quickstart guide
- **[Core Concepts](docs/concepts.md)** - Understand Agents, Fleets, and Flows
- **[Tutorials](docs/tutorials/)** - Step-by-step guides
  - [Your First Agent](docs/tutorials/first-agent.md)
  - [Building a Slack Bot](docs/tutorials/slack-bot.md)
  - [Incident Response Flow](docs/tutorials/incident-response.md)
- **[Reference](docs/reference/)** - Complete YAML and CLI specs
  - [Agent Spec](docs/reference/agent-spec.md)
  - [AgentFlow Spec](docs/reference/agentflow-spec.md)
  - [aofctl CLI](docs/reference/aofctl.md)
- **[Examples](docs/examples/)** - Copy-paste ready YAML files

## Example: Incident Response Flow

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-response
spec:
  trigger:
    type: Webhook
    config:
      path: /pagerduty

  nodes:
    - id: diagnose
      type: Agent
      config:
        agent: k8s-diagnostic-agent

    - id: auto-fix
      type: Agent
      config:
        agent: remediation-agent
      conditions:
        - severity != "critical"

    - id: human-approval
      type: Slack
      config:
        channel: "#sre-alerts"
        message: "Critical issue detected. Approve fix?"
      conditions:
        - severity == "critical"

  connections:
    - from: diagnose
      to: auto-fix
    - from: diagnose
      to: human-approval
```

## Community & Support

- **Documentation**: [https://aof.dev/docs](https://aof.dev/docs)
- **GitHub**: [https://github.com/yourusername/aof](https://github.com/yourusername/aof)
- **Issues**: [Report bugs or request features](https://github.com/yourusername/aof/issues)
- **Discussions**: [Join the community](https://github.com/yourusername/aof/discussions)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache 2.0 - See [LICENSE](LICENSE) for details.

---

**Built by ops engineers, for ops engineers.** 🚀
