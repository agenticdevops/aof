---
sidebar_position: 1
title: Documentation Index
---

# AOF Documentation Index

Complete documentation for the Agentic Ops Framework (AOF).

## 📚 Documentation Structure

### Main README
- **[Project README](https://github.com/agenticopsorg/aof)** - Project overview, quick install, 30-second example

### Getting Started
- **[Getting Started](./getting-started)** - 5-minute quickstart guide
  - Installation options (cargo, binary, source)
  - API key configuration
  - First agent creation and execution
  - Common troubleshooting

### Core Concepts
- **[Core Concepts](./concepts)** - Understanding AOF fundamentals
  - Agents - Single AI assistants
  - AgentFleets - Teams of agents
  - AgentFlows - Workflow automation
  - Tools - MCP, Shell, HTTP, integrations
  - Models - Multi-provider support
  - Memory - Context persistence

### Tutorials (Step-by-Step)
1. **[Build Your First Agent](./tutorials/first-agent)** (15 min)
   - Agent definition and configuration
   - Adding tools (Shell, MCP)
   - Memory management
   - Deployment and testing

2. **[Create a Slack Bot](./tutorials/slack-bot)** (20 min)
   - Slack app setup
   - Event handling
   - Human-in-the-loop approvals
   - Interactive features

3. **[Incident Response Flow](./tutorials/incident-response)** (30 min)
   - PagerDuty integration
   - Auto-diagnostics
   - Conditional remediation
   - Post-incident analysis

### Reference Documentation
- **[Agent YAML Spec](./reference/agent-spec)** - Complete Agent specification
  - Metadata fields
  - Model configuration
  - Instructions best practices
  - All tool types (Shell, HTTP, MCP, Slack, GitHub, etc.)
  - Memory types and configuration
  - Complete examples

- **[AgentFlow YAML Spec](./reference/agentflow-spec)** - Complete AgentFlow specification
  - 8 trigger types (Webhook, Schedule, Slack, GitHub, etc.)
  - 9 node types (Agent, Fleet, HTTP, Shell, Conditional, etc.)
  - Connections and conditions
  - Variable interpolation
  - Error handling

- **[aofctl CLI Reference](./reference/aofctl)** - Complete CLI command reference
  - Agent commands (apply, get, run, chat, exec, logs, etc.)
  - Fleet commands (create, scale, exec, status)
  - Flow commands (apply, run, status, visualize)
  - Config management
  - Examples and troubleshooting

### Examples (Copy-Paste Ready)
- **[Examples Overview](./examples/)** - Overview of all examples

#### Production-Ready Examples:
1. **[kubernetes-agent.yaml](./examples/#1-kubernetes-operations-agent)**
   - Interactive K8s cluster management
   - Safe kubectl execution
   - Pod/deployment troubleshooting

2. **[github-pr-reviewer.yaml](./examples/#2-github-pr-review-agent)**
   - Automated code review
   - Security scanning
   - Best practices enforcement
   - Automated PR comments

3. **[incident-responder.yaml](./examples/#3-incident-response-system)**
   - PagerDuty webhook integration
   - Intelligent diagnostics
   - Auto-remediation with approvals
   - Incident tracking

4. **[slack-bot-flow.yaml](./examples/#4-slack-bot-with-interactive-features)**
   - Conversational K8s assistant
   - Interactive approvals
   - Daily reports
   - Slash commands

5. **[daily-report-flow.yaml](./examples/#5-dailyweekly-reports)**
   - Scheduled cluster health reports
   - Weekly summaries
   - Custom on-demand reports

## 📖 Recommended Reading Path

### For First-Time Users:
1. Start with **[Project README](https://github.com/agenticopsorg/aof)** - Understand what AOF is
2. Follow **[Getting Started](./getting-started)** - Get up and running
3. Read **[Core Concepts](./concepts)** - Understand the building blocks
4. Try **[First Agent Tutorial](./tutorials/first-agent)** - Hands-on practice

### For Production Deployment:
1. Review **[Agent Spec](./reference/agent-spec)** - Understand all options
2. Study **[Examples](./examples/)** - See production patterns
3. Read **[AgentFlow Spec](./reference/agentflow-spec)** - Learn workflow automation
4. Check **[CLI Reference](./reference/aofctl)** - Master the tools

### For Specific Use Cases:
- **Slack Bot**: [Slack Bot Tutorial](./tutorials/slack-bot) + [slack-bot-flow.yaml](./examples/#4-slack-bot-with-interactive-features)
- **Incident Response**: [Incident Response Tutorial](./tutorials/incident-response) + [incident-responder.yaml](./examples/#3-incident-response-system)
- **Code Review**: [github-pr-reviewer.yaml](./examples/#2-github-pr-review-agent)
- **K8s Operations**: [kubernetes-agent.yaml](./examples/#1-kubernetes-operations-agent)

## 🎯 Documentation by Role

### DevOps Engineers
Essential reading:
- [Getting Started](./getting-started)
- [kubernetes-agent.yaml](./examples/#1-kubernetes-operations-agent)
- [incident-responder.yaml](./examples/#3-incident-response-system)
- [Agent Spec](./reference/agent-spec) (Tools section)

### SRE Teams
Essential reading:
- [Core Concepts](./concepts)
- [Incident Response Tutorial](./tutorials/incident-response)
- [incident-responder.yaml](./examples/#3-incident-response-system)
- [daily-report-flow.yaml](./examples/#5-dailyweekly-reports)

### Platform Engineers
Essential reading:
- [AgentFlow Spec](./reference/agentflow-spec)
- [All Examples](./examples/)
- [CLI Reference](./reference/aofctl)
- All tutorials

## 🔍 Quick Reference

### Common Tasks

| Task | Documentation |
|------|---------------|
| Install AOF | [Getting Started](./getting-started) |
| Create first agent | [First Agent Tutorial](./tutorials/first-agent) |
| Add kubectl tools | [Agent Spec - Tools](./reference/agent-spec#tool-shell) |
| Build Slack bot | [Slack Bot Tutorial](./tutorials/slack-bot) |
| Setup auto-remediation | [Incident Response Tutorial](./tutorials/incident-response) |
| Schedule workflows | [AgentFlow Spec - Schedule Trigger](./reference/agentflow-spec#schedule) |
| CLI commands | [aofctl Reference](./reference/aofctl) |

### YAML Quick Reference

| Resource | Spec Doc | Example |
|----------|----------|---------|
| Agent | [agent-spec.md](./reference/agent-spec) | [kubernetes-agent.yaml](./examples/#1-kubernetes-operations-agent) |
| AgentFleet | [agent-spec.md](./reference/agent-spec) | Coming soon |
| AgentFlow | [agentflow-spec.md](./reference/agentflow-spec) | [slack-bot-flow.yaml](./examples/#4-slack-bot-with-interactive-features) |

### Model Providers

| Provider | Format | Env Variable | Docs |
|----------|--------|--------------|------|
| OpenAI | `openai:gpt-4` | `OPENAI_API_KEY` | [Agent Spec](./reference/agent-spec#specmodel) |
| Anthropic | `anthropic:claude-3-5-sonnet-20241022` | `ANTHROPIC_API_KEY` | [Agent Spec](./reference/agent-spec#specmodel) |
| Ollama | `ollama:llama3` | None | [Agent Spec](./reference/agent-spec#specmodel) |
| Groq | `groq:llama-3.1-70b-versatile` | `GROQ_API_KEY` | [Agent Spec](./reference/agent-spec#specmodel) |

## 🛠️ Tool Documentation

| Tool Type | Description | Docs |
|-----------|-------------|------|
| Shell | Execute terminal commands | [Agent Spec - Shell](./reference/agent-spec#tool-shell) |
| HTTP | REST API requests | [Agent Spec - HTTP](./reference/agent-spec#tool-http) |
| MCP | Model Context Protocol servers | [Agent Spec - MCP](./reference/agent-spec#tool-mcp-model-context-protocol) |
| Slack | Slack integration | [Agent Spec - Slack](./reference/agent-spec#tool-slack) |
| GitHub | GitHub API | [Agent Spec - GitHub](./reference/agent-spec#tool-github) |
| PagerDuty | Incident management | [Agent Spec - PagerDuty](./reference/agent-spec#tool-pagerduty) |

## 📝 Contributing

### Documentation Contributions
- Fix typos or improve clarity
- Add missing examples
- Update outdated information
- Translate to other languages

### Example Contributions
See [Examples README](./examples/) for guidelines.

## 🆘 Getting Help

1. **Check documentation** - Search this index
2. **Review examples** - See [examples](./examples/)
3. **Troubleshooting** - Check each tutorial's troubleshooting section
4. **GitHub Issues** - [Report bugs or request features](https://github.com/agenticopsorg/aof/issues)
5. **Discussions** - [Ask questions](https://github.com/agenticopsorg/aof/discussions)

## 📊 Documentation Coverage

### ✅ Complete
- [x] Main README
- [x] Getting Started guide
- [x] Core Concepts
- [x] 3 comprehensive tutorials
- [x] Complete Agent YAML reference
- [x] Complete AgentFlow YAML reference
- [x] Complete CLI reference
- [x] 5 production-ready examples

### 🚧 Coming Soon
- [ ] AgentFleet tutorial
- [ ] Advanced patterns guide
- [ ] Performance tuning guide
- [ ] Security best practices
- [ ] Migration from other frameworks
- [ ] API documentation (if REST API is added)

## 🔄 Documentation Updates

Last updated: 2024-01-20

### Recent Changes
- Added complete reference documentation
- Added 5 production examples
- Added 3 step-by-step tutorials
- Added quickstart guide

---

**Questions?** Start with [Getting Started](./getting-started) or jump to a [Tutorial](./tutorials/first-agent).

**Building something?** Check the [Examples](./examples/) for copy-paste templates.

**Need details?** See the [Reference Documentation](./reference/agent-spec).
