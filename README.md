# AOF - Agentic Ops Framework

[![GitHub stars](https://img.shields.io/github/stars/agenticdevops/aof?style=for-the-badge&logo=github)](https://github.com/agenticdevops/aof)
[![GitHub forks](https://img.shields.io/github/forks/agenticdevops/aof?style=for-the-badge&logo=github)](https://github.com/agenticdevops/aof/fork)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=for-the-badge)](https://github.com/agenticdevops/aof/blob/main/LICENSE)
[![Docs](https://img.shields.io/badge/Docs-docs.aof.sh-green?style=for-the-badge)](https://docs.aof.sh)

> **n8n for Agentic Ops** - Build AI agents with Kubernetes-style YAML. No Python required.

AOF is a Rust-based framework that lets DevOps, SRE, and Platform engineers build and orchestrate AI agents using familiar YAML specifications and kubectl-style CLI commands.

**⭐ If you find AOF useful, please star this repo! It helps us reach more developers.**

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
- **🔀 Multi-Provider**: OpenAI, Anthropic, Google Gemini, Ollama, Groq - switch with one line
- **📊 AgentFlow**: n8n-style visual DAG workflows for complex automation
- **🚀 Production Ready**: Built in Rust for performance and reliability
- **🔧 Ops-Native**: kubectl-style CLI that feels familiar

## Quick Install

### Option 1: Cargo (Rust users)
```bash
cargo install aofctl
```

### Option 2: Binary Download (Recommended)
```bash
# Auto-detect platform and install
curl -sSL https://docs.aof.sh/install.sh | bash
```

Or download manually from [GitHub Releases](https://github.com/agenticdevops/aof/releases).

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
    - shell
EOF

# Run it interactively
aofctl run agent my-agent.yaml

# Or with a query
aofctl run agent my-agent.yaml -i "How do I check if my pods are running?"
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
│(OpenAI/Claude/ │  │  (kubectl/git)  │  │(Slack/PagerDuty/ │
│ Gemini/Ollama) │  │                 │  │Discord/Telegram) │
└────────────────┘  └─────────────────┘  └─────────────────┘
```

## Documentation

📚 **Full documentation at [docs.aof.sh](https://docs.aof.sh)**

- **[Getting Started](https://docs.aof.sh/docs/getting-started)** - 5-minute quickstart guide
- **[Core Concepts](https://docs.aof.sh/docs/concepts)** - Understand Agents, Fleets, and Flows
- **[Tutorials](https://docs.aof.sh/docs/tutorials/first-agent)** - Step-by-step guides
- **[CLI Reference](https://docs.aof.sh/docs/reference/aofctl)** - Complete CLI documentation
- **[Examples](https://docs.aof.sh/docs/examples)** - Copy-paste ready YAML files

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

- **Documentation**: [docs.aof.sh](https://docs.aof.sh)
- **GitHub**: [github.com/agenticdevops/aof](https://github.com/agenticdevops/aof)
- **Issues**: [Report bugs or request features](https://github.com/agenticdevops/aof/issues)
- **Discussions**: [Join the community](https://github.com/agenticdevops/aof/discussions)

**⭐ Star us on GitHub** - It helps more DevOps engineers discover AOF!

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache 2.0 - See [LICENSE](LICENSE) for details.

---

**Built by ops engineers, for ops engineers.** 🚀
