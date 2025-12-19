# AOF Documentation Index

Complete documentation for the Agentic Ops Framework (AOF).

## 📚 Documentation Structure

### Main README
- **[README.md](../README.md)** - Project overview, quick install, 30-second example

### Getting Started
- **[Getting Started](getting-started.md)** - 5-minute quickstart guide
  - Installation options (cargo, binary, source)
  - API key configuration
  - First agent creation and execution
  - Common troubleshooting

### Introduction (New!)
- **[Core Concepts](introduction/concepts.md)** ⭐ Complete composable architecture guide
  - The 6 resource types: Agent, Fleet, Flow, Context, Trigger, FlowBinding
  - Resource composition and references (ref: syntax)
  - When to use each resource type
  - Current (v1) vs Future (v1alpha2) architecture
  - Multi-tenant routing and patterns
- **[Quickstart Guide](introduction/quickstart.md)** - Your first agent in 5 minutes
  - Installation with curl script
  - Create a Telegram bot trigger
  - Connect agent + flow + context
  - Test end-to-end workflow
- **[Enterprise Setup](guides/enterprise-setup.md)** - Multi-environment production deployment
  - Multi-environment (prod/staging/dev)
  - Multi-cluster Kubernetes
  - Multi-organization (different Slack workspaces)
  - Security, audit logging, rate limiting

### Core Concepts (Legacy)
- **[Core Concepts](concepts.md)** - Quick reference (redirects to introduction/concepts.md)
  - Agents - Single AI assistants
  - AgentFleets - Teams of agents
  - AgentFlows - Workflow automation
  - Tools - MCP, Shell, HTTP, integrations
  - Models - Multi-provider support
  - Memory - Context persistence

### Advanced Concepts
- **[AgentFleet: Multi-Agent Coordination](concepts/fleets.md)** - Deep dive into fleets
  - Why use fleets in DevOps (consensus, parallel execution, specialization)
  - 5 coordination modes (peer, hierarchical, pipeline, swarm, **tiered**)
  - 5 consensus algorithms (majority, unanimous, weighted, first_wins, **human_review**)
  - Fleet vs single agent comparison
  - Real-world examples (code review, incident response, data pipeline)

### Architecture
- **[Multi-Model Consensus Architecture](architecture/multi-model-consensus.md)** - Deep dive into multi-model AI
  - The problem with single-model analysis
  - Tiered execution model (data collection → reasoning → synthesis)
  - Consensus engine implementation (5 algorithms)
  - Data flow architecture
  - Cost optimization strategies
  - Use cases (RCA, code review, performance analysis)

### Tutorials (Step-by-Step)
1. **[Build Your First Agent](tutorials/first-agent.md)** (15 min)
   - Agent definition and configuration
   - Adding tools (Shell, MCP)
   - Memory management
   - Deployment and testing

2. **[Create a Slack Bot](tutorials/slack-bot.md)** (20 min)
   - Slack app setup
   - Event handling
   - Human-in-the-loop approvals
   - Interactive features

3. **[Incident Response Flow](tutorials/incident-response.md)** (30 min)
   - PagerDuty integration
   - Auto-diagnostics
   - Conditional remediation
   - Post-incident analysis

4. **[Build an RCA Fleet](tutorials/rca-fleet.md)** (25 min)
   - Multi-agent root cause analysis
   - Specialist agents (error analyzer, dependency checker, config auditor)
   - Consensus-based incident diagnosis
   - Customization for different tech stacks (K8s, database, AWS)

5. **[Multi-Model RCA with Tiered Execution](tutorials/multi-model-rca.md)** (35 min) ⭐ NEW
   - Tiered architecture (data collectors → reasoning models → coordinator)
   - Multi-model consensus (Claude + Gemini + GPT-4)
   - Cost-optimized data collection (cheap models for Tier 1)
   - Weighted consensus with confidence scoring
   - Production-ready RCA report generation

### Reference Documentation
- **[Agent YAML Spec](reference/agent-spec.md)** - Complete Agent specification
  - Metadata fields
  - Model configuration
  - Instructions best practices
  - All tool types (Shell, HTTP, MCP, Slack, GitHub, etc.)
  - Memory types and configuration
  - Complete examples

- **[AgentFlow YAML Spec](reference/agentflow-spec.md)** - Complete AgentFlow specification
  - 8 trigger types (Webhook, Schedule, Slack, GitHub, etc.)
  - 9 node types (Agent, Fleet, HTTP, Shell, Conditional, etc.)
  - Connections and conditions
  - Variable interpolation
  - Error handling

- **[AgentFleet YAML Spec](reference/fleet-spec.md)** - Complete Fleet specification
  - Coordination modes (peer, hierarchical, pipeline, swarm, tiered)
  - Consensus algorithms (majority, unanimous, weighted, first_wins, human_review)
  - Tiered configuration (per-tier consensus, final aggregation)
  - Shared resources and communication patterns
  - Complete examples

- **[aofctl CLI Reference](reference/aofctl.md)** - Complete CLI command reference
  - Agent commands (apply, get, run, chat, exec, logs, etc.)
  - Fleet commands (create, scale, exec, status)
  - Flow commands (apply, run, status, visualize)
  - Config management
  - Examples and troubleshooting

- **[Platform Policies Reference](reference/platform-policies.md)** ⭐ NEW - Complete platform policy specification
  - Default policies for all platforms (CLI, Slack, Telegram, WhatsApp, Discord)
  - Action classes (read, write, delete, dangerous)
  - Custom policy configuration
  - Environment-specific policies
  - Approval workflow integration

- **[Kubernetes CRDs](reference/kubernetes-crds.md)** ⭐ NEW - Complete CRD definitions for Kubernetes Operator
  - All 6 resource CRDs (Agent, Fleet, Flow, Context, Trigger, FlowBinding)
  - Operator architecture overview
  - Status fields and conditions
  - Cross-namespace references
  - RBAC requirements
  - Multi-tenancy patterns
  - Installation and deployment
  - GitOps integration

### Examples (Copy-Paste Ready)
- **[Examples README](examples/README.md)** - Overview of all examples

#### Production-Ready Examples:
1. **[kubernetes-agent.yaml](examples/kubernetes-agent.yaml)**
   - Interactive K8s cluster management
   - Safe kubectl execution
   - Pod/deployment troubleshooting

2. **[github-pr-reviewer.yaml](examples/github-pr-reviewer.yaml)**
   - Automated code review
   - Security scanning
   - Best practices enforcement
   - Automated PR comments

3. **[incident-responder.yaml](examples/incident-responder.yaml)**
   - PagerDuty webhook integration
   - Intelligent diagnostics
   - Auto-remediation with approvals
   - Incident tracking

4. **[slack-bot-flow.yaml](examples/slack-bot-flow.yaml)**
   - Conversational K8s assistant
   - Interactive approvals
   - Daily reports
   - Slash commands

5. **[daily-report-flow.yaml](examples/daily-report-flow.yaml)**
   - Scheduled cluster health reports
   - Weekly summaries
   - Custom on-demand reports

## 📖 Recommended Reading Path

### For First-Time Users:
1. Start with **[README.md](../README.md)** - Understand what AOF is
2. Follow **[Getting Started](getting-started.md)** - Get up and running
3. Read **[Core Concepts](concepts.md)** - Understand the building blocks
4. Try **[First Agent Tutorial](tutorials/first-agent.md)** - Hands-on practice

### For Production Deployment:
1. Review **[Agent Spec](reference/agent-spec.md)** - Understand all options
2. Study **[Examples](examples/)** - See production patterns
3. Read **[AgentFlow Spec](reference/agentflow-spec.md)** - Learn workflow automation
4. Check **[CLI Reference](reference/aofctl.md)** - Master the tools

### For Specific Use Cases:
- **Slack Bot**: [Slack Bot Tutorial](tutorials/slack-bot.md) + [slack-bot-flow.yaml](examples/slack-bot-flow.yaml)
- **Human-in-the-Loop Approval**: [Approval Workflow Guide](guides/approval-workflow.md) - Reaction-based command approval
- **Conversation Memory**: [Conversation Memory Guide](guides/conversation-memory.md) - Context persistence across messages
- **Context Switching**: [Context Switching Guide](guides/context-switching.md) - Switch between projects/clusters (replaces /env and /agents)
- **Telegram Mobile Companion**: [Telegram Mobile Guide](guides/telegram-mobile.md) - Safe mobile access with read-first UX
- **Safety Layer**: [Safety Layer Guide](guides/safety-layer.md) - Platform-agnostic tool classification and policies
- **Platform Policies**: [Platform Policies Reference](reference/platform-policies.md) - Complete policy specification
- **WhatsApp Integration**: Uses same safety layer as Telegram (see [Safety Layer Guide](guides/safety-layer.md))
- **Discord Integration**: Uses same safety layer as Slack (see [Safety Layer Guide](guides/safety-layer.md))
- **Incident Response**: [Incident Response Tutorial](tutorials/incident-response.md) + [incident-responder.yaml](examples/incident-responder.yaml)
- **Code Review**: [github-pr-reviewer.yaml](examples/github-pr-reviewer.yaml)
- **K8s Operations**: [kubernetes-agent.yaml](examples/kubernetes-agent.yaml)

## 🎯 Documentation by Role

### DevOps Engineers
Essential reading:
- [Getting Started](getting-started.md)
- [kubernetes-agent.yaml](examples/kubernetes-agent.yaml)
- [incident-responder.yaml](examples/incident-responder.yaml)
- [Agent Spec](reference/agent-spec.md) (Tools section)

### SRE Teams
Essential reading:
- [Core Concepts](concepts.md)
- [Incident Response Tutorial](tutorials/incident-response.md)
- [incident-responder.yaml](examples/incident-responder.yaml)
- [daily-report-flow.yaml](examples/daily-report-flow.yaml)

### Platform Engineers
Essential reading:
- [AgentFlow Spec](reference/agentflow-spec.md)
- [All Examples](examples/)
- [CLI Reference](reference/aofctl.md)
- All tutorials

## 🔍 Quick Reference

### Common Tasks

| Task | Documentation |
|------|---------------|
| Install AOF | [Getting Started](getting-started.md) |
| Create first agent | [First Agent Tutorial](tutorials/first-agent.md) |
| Add kubectl tools | [Agent Spec - Tools](reference/agent-spec.md#tool-shell) |
| Build Slack bot | [Slack Bot Tutorial](tutorials/slack-bot.md) |
| Add approval workflow | [Approval Workflow Guide](guides/approval-workflow.md) |
| Setup auto-remediation | [Incident Response Tutorial](tutorials/incident-response.md) |
| Schedule workflows | [AgentFlow Spec - Schedule Trigger](reference/agentflow-spec.md#schedule) |
| CLI commands | [aofctl Reference](reference/aofctl.md) |

### YAML Quick Reference

| Resource | Spec Doc | Example |
|----------|----------|---------|
| Agent | [agent-spec.md](reference/agent-spec.md) | [kubernetes-agent.yaml](examples/kubernetes-agent.yaml) |
| AgentFleet | [fleet-spec.md](reference/fleet-spec.md) | [multi-model-rca-fleet.yaml](../examples/fleets/multi-model-rca-fleet.yaml) |
| AgentFlow | [agentflow-spec.md](reference/agentflow-spec.md) | [slack-bot-flow.yaml](examples/slack-bot-flow.yaml) |

### Model Providers

| Provider | Format | Env Variable | Docs |
|----------|--------|--------------|------|
| OpenAI | `openai:gpt-4` | `OPENAI_API_KEY` | [Agent Spec](reference/agent-spec.md#specmodel) |
| Anthropic | `anthropic:claude-3-5-sonnet-20241022` | `ANTHROPIC_API_KEY` | [Agent Spec](reference/agent-spec.md#specmodel) |
| Ollama | `ollama:llama3` | None | [Agent Spec](reference/agent-spec.md#specmodel) |
| Groq | `groq:llama-3.1-70b-versatile` | `GROQ_API_KEY` | [Agent Spec](reference/agent-spec.md#specmodel) |

## 🛠️ Tool Documentation

| Tool Type | Description | Docs |
|-----------|-------------|------|
| Shell | Execute terminal commands | [Agent Spec - Shell](reference/agent-spec.md#tool-shell) |
| HTTP | REST API requests | [Agent Spec - HTTP](reference/agent-spec.md#tool-http) |
| MCP | Model Context Protocol servers | [Agent Spec - MCP](reference/agent-spec.md#tool-mcp-model-context-protocol) |
| Slack | Slack integration | [Agent Spec - Slack](reference/agent-spec.md#tool-slack) |
| GitHub | GitHub API | [Agent Spec - GitHub](reference/agent-spec.md#tool-github) |
| PagerDuty | Incident management | [Agent Spec - PagerDuty](reference/agent-spec.md#tool-pagerduty) |

## 📝 Contributing

### Documentation Contributions
- Fix typos or improve clarity
- Add missing examples
- Update outdated information
- Translate to other languages

### Example Contributions
See [Examples README](examples/README.md#contributing-examples) for guidelines.

## 🆘 Getting Help

1. **Check documentation** - Search this index
2. **Review examples** - See [examples/](examples/)
3. **Troubleshooting** - Check each tutorial's troubleshooting section
4. **GitHub Issues** - [Report bugs or request features](https://github.com/agenticdevops/aof/issues)
5. **Discussions** - [Ask questions](https://github.com/agenticdevops/aof/discussions)

## 📊 Documentation Coverage

### ✅ Complete
- [x] Main README
- [x] Getting Started guide
- [x] Core Concepts
- [x] AgentFleet multi-agent coordination guide (5 modes, 5 consensus algorithms)
- [x] Multi-Model Consensus Architecture guide
- [x] 5 comprehensive tutorials (including Multi-Model RCA)
- [x] Complete Agent YAML reference
- [x] Complete AgentFlow YAML reference
- [x] Complete AgentFleet YAML reference
- [x] Complete CLI reference
- [x] 12+ example agents (observability + reasoning)
- [x] Multi-model RCA fleet example

### 🚧 Coming Soon
- [ ] Advanced patterns guide
- [ ] Performance tuning guide
- [ ] Security best practices
- [ ] Migration from other frameworks
- [ ] API documentation (if REST API is added)

## 🔄 Documentation Updates

Last updated: 2025-12-19

### Recent Changes
- **NEW**: Context Switching Guide - Unified `/context` command for project/cluster switching
  - `/context` command with inline keyboard selection
  - Context = Agent + Connection Parameters (replaces `/env` and `/agents`)
  - Per-user context session tracking
  - Example context files (k8s-cluster-a.yaml, aws-dev.yaml, database.yaml, prometheus.yaml)
- **NEW**: Comprehensive DevOps Read-Only Agent
  - Single agent with kubectl, docker, helm, git, terraform, aws, prometheus
  - Designed as default mobile agent (no constant switching)
  - Located at examples/agents/mobile-read-only/devops-readonly.yaml
- **NEW**: Platform Policies Reference - Complete platform policy specification
  - Default policies for CLI, Slack, Telegram, WhatsApp, Discord
  - Action classes and custom policy configuration
  - Environment-specific policies
- **UPDATED**: Safety Layer Guide - Now emphasizes platform-agnostic architecture
  - Platform trust hierarchy documentation
  - Testing instructions with example configurations
  - Extension guide for new platforms
- **UPDATED**: Telegram Mobile Guide - Now reflects platform-agnostic foundation
  - Testing examples and end-to-end test instructions
  - Architecture diagram showing shared safety components
- **NEW**: Kubernetes CRD Documentation - Complete CRD definitions for future operator
  - All 6 resource CRDs with OpenAPI v3 schemas
  - Status fields and operator-managed conditions
  - Cross-namespace references and RBAC patterns
  - Multi-tenancy and namespace isolation
  - GitOps integration examples
  - Migration path from CLI to operator
- **UPDATED**: Context, Trigger, and FlowBinding docs with K8s CRD compatibility sections
  - Status field specifications
  - Namespace support details
  - Cross-namespace reference patterns
  - kubectl usage examples
- **NEW**: Conversation Memory System - Context persistence across Slack messages
- **NEW**: AgentFlow Multi-Tenant Routing - Route messages to different agents based on patterns
- **NEW**: Bot self-approval prevention - Auto-detects bot_user_id at startup
- **NEW**: Human-in-the-Loop Approval Workflow Guide - Reaction-based command approval for Slack
- **NEW**: Conversation Memory Guide - Contextual follow-up conversations
- **NEW**: Multi-Model Consensus Architecture guide
- **NEW**: Multi-Model RCA Tutorial with tiered execution
- **NEW**: Telegram Mobile Companion - Safe mobile access with /agents, /flows commands
- **NEW**: Safety Layer - Platform-agnostic tool classification and policies
- **NEW**: Platform Policies Reference - Complete policy specification for all platforms
- **NEW**: ASCII Visualization Crate (aof-viz) - Mobile-friendly output rendering
- **NEW**: AgentFleet YAML Reference (fleet-spec.md)
- **NEW**: 8 observability + reasoning example agents
- **NEW**: Tiered coordination mode (data collectors → reasoning → synthesis)
- **NEW**: HumanReview consensus algorithm
- Updated AgentFleet concepts with 5 coordination modes
- Updated Docusaurus sidebar with Architecture section
- Added RCA Fleet tutorial with consensus patterns
- Added complete reference documentation

---

**Questions?** Start with [Getting Started](getting-started.md) or jump to a [Tutorial](tutorials/).

**Building something?** Check the [Examples](examples/) for copy-paste templates.

**Need details?** See the [Reference Documentation](reference/).
