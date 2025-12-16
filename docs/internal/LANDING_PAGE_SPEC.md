# AOF Landing Page Specification

**Target URL**: https://aof.sh
**Documentation Link**: https://docs.aof.sh/docs/intro
**Builder.io Implementation Guide**

---

## Page Overview

**Goal**: Convert DevOps/SRE/Platform Engineers to try AOF by demonstrating it's the only AI agent framework built specifically for operations teams.

**Primary CTA**: "Get Started" → https://docs.aof.sh/docs/intro
**Secondary CTA**: "View on GitHub" → https://github.com/agenticdevops/aof

---

## Section 1: Hero

### Layout
- Full-width hero with dark gradient background (zinc-950 to zinc-900)
- Centered content with animated terminal preview on right

### Content

**Tagline** (small, above headline):
```
The AI Agent Framework Built for Operations
```

**Headline** (H1, large, bold):
```
n8n for Agentic Ops
```

**Subheadline** (H2, medium):
```
Build AI agents with Kubernetes-style YAML. No Python required.
```

**Description** (paragraph):
```
AOF is the only AI agent framework designed from the ground up for DevOps, SRE,
and Platform Engineers. Define agents in YAML, run with kubectl-style commands,
deploy anywhere—no Kubernetes required. Native integrations with Slack, PagerDuty,
Jira, Discord, GitHub, and more.
```

**Primary CTA Button** (large, orange-700 background):
```
Get Started in 5 Minutes →
```
Link: https://docs.aof.sh/docs/intro

**Secondary CTA** (outline button):
```
View on GitHub
```
Link: https://github.com/agenticdevops/aof

**Terminal Preview** (animated, dark theme):
```yaml
# my-agent.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: openai:gpt-4
  instructions: |
    You are a Kubernetes expert assistant.
    Help users troubleshoot pod issues.
  tools:
    - shell
```

```bash
$ aofctl run agent my-agent.yaml
> Agent 'k8s-helper' is ready. Type your message:
> Why is my pod in CrashLoopBackOff?
```

---

## Section 2: The Problem

### Layout
- 3-column grid on dark background
- Pain point icons with descriptions

### Headline
```
AI Frameworks Weren't Built for Ops
```

### Pain Points

**Column 1: Icon - Code brackets**
```
Title: "Learn Python First"
Description: LangChain, CrewAI, and Agno require you to write Python code.
Your team manages YAML all day—why learn a new language for AI?
```

**Column 2: Icon - Kubernetes logo**
```
Title: "Kubernetes Required"
Description: Kagent ties you to Kubernetes. What about your VMs,
bare metal servers, or hybrid infrastructure?
```

**Column 3: Icon - Puzzle pieces**
```
Title: "Complex Dependencies"
Description: Python virtual environments, pip dependencies, version conflicts.
More infrastructure to manage just to run an agent.
```

---

## Section 3: The Solution - Why AOF

### Layout
- Large feature blocks with code examples
- Alternating left/right layout

### Headline
```
Built for Ops, By Ops
```

### Subheadline
```
If you know kubectl, you already know AOF.
```

### Feature Block 1: YAML-First

**Title**: Define Agents Like K8s Resources

**Description**:
```
No Python. No code. Just YAML you already understand. AOF uses the same
declarative patterns you use for Kubernetes manifests, Terraform configs,
and Ansible playbooks.
```

**Code Example**:
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: incident-responder
  labels:
    team: sre
    env: production
spec:
  model: anthropic:claude-3-5-sonnet-20241022
  instructions: |
    You are an SRE assistant. Diagnose incidents,
    suggest fixes, and execute approved remediations.
  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, helm, docker]
    - type: Slack
    - type: PagerDuty
```

### Feature Block 2: kubectl-Style CLI

**Title**: Commands You Already Know

**Description**:
```
apply, get, describe, logs, delete—AOF's CLI mirrors kubectl exactly.
Your muscle memory works here. No new workflows to learn.
```

**Code Example**:
```bash
# Apply configuration
aofctl apply -f agent.yaml

# Get resources
aofctl get agents

# Run interactively
aofctl run agent my-agent.yaml

# Check logs
aofctl logs agent my-agent

# Delete when done
aofctl delete agent my-agent
```

### Feature Block 3: Zero Dependencies

**Title**: Single Binary. Runs Anywhere.

**Description**:
```
Built in Rust for speed and reliability. One binary, zero dependencies.
No Python, no Node.js, no virtual environments. Install and run in seconds.
```

**Installation Example**:
```bash
# That's it. You're done.
cargo install aofctl

# Or download the binary
curl -sSL https://aof.sh/install.sh | bash
```

### Feature Block 4: No Kubernetes Required

**Title**: Deploy Anywhere—K8s Optional

**Description**:
```
Unlike Kagent, AOF doesn't require Kubernetes. Run agents on your laptop,
VMs, bare metal, Docker, or yes—Kubernetes too. Your infrastructure, your choice.
```

**Deployment Options** (icon grid):
- Laptop/Local
- VMs (EC2, GCE, Azure VMs)
- Bare Metal
- Docker/Containers
- Kubernetes (optional)
- Serverless (AWS Lambda, Cloud Run)

---

## Section 4: Feature Comparison Table

### Layout
- Full-width table with sticky header
- Checkmarks/X marks for feature support
- AOF column highlighted

### Headline
```
How AOF Compares
```

### Table Structure

| Feature | AOF | LangChain | CrewAI | Kagent | Agno |
|---------|-----|-----------|--------|--------|------|
| **Configuration** | YAML | Python | Python | YAML (K8s CRD) | Python |
| **CLI Style** | kubectl-like | Custom | Custom | kubectl | Custom |
| **Kubernetes Required** | No | No | No | **Yes** | No |
| **Python Required** | No | **Yes** | **Yes** | No | **Yes** |
| **Language** | Rust | Python | Python | Go | Python |
| **Single Binary** | Yes | No | No | Yes | No |
| **MCP Support** | Native | Plugin | No | No | Plugin |
| **Multi-Provider LLM** | Yes | Yes | Yes | Limited | Yes |
| **Workflow Orchestration** | Built-in (AgentFlow) | LangGraph | Built-in | Via Argo | No |
| **Slack/Discord Integration** | Native | Via code | Via code | No | Via code |
| **PagerDuty/OpsGenie** | Native | Via code | No | No | No |
| **Jira/GitHub Triggers** | Native | Via code | No | No | Via code |
| **Target Audience** | DevOps/SRE | Developers | Developers | K8s Admins | Developers |
| **License** | Apache 2.0 | MIT | MIT | Apache 2.0 | Apache 2.0 |

### Comparison Callouts (4 cards below table)

**Card 1: vs LangChain/CrewAI**
```
Title: "No Python Required"
AOF is YAML-native. Define agents declaratively without writing
or maintaining Python code. Perfect for teams that live in YAML.
```

**Card 2: vs Kagent**
```
Title: "No Kubernetes Lock-in"
Kagent requires a K8s cluster. AOF runs anywhere—your laptop,
VMs, containers, or K8s. No cluster overhead for simple agents.
```

**Card 3: vs All Python Frameworks**
```
Title: "Native Ops Integrations"
Other frameworks require writing Python code for every integration.
AOF has Slack, PagerDuty, Jira, Discord, and GitHub built-in.
Just YAML configuration—no coding required.
```

**Card 4: vs All**
```
Title: "Built for Ops, By Ops"
Other frameworks target developers building chatbots. AOF is the only
framework purpose-built for incident response, automation, and operations.
```

---

## Section 5: Native Ops Integrations

### Layout
- Full-width section with integration logo grid
- Trigger types and tool types separated

### Headline
```
Connects to Your Entire Ops Stack
```

### Subheadline
```
Native integrations with the tools DevOps teams actually use.
No plugins. No middleware. Just YAML.
```

### Trigger Integrations (Event Sources)
*"Start workflows from any of these:"*

| Integration | Trigger Type | Use Case |
|-------------|--------------|----------|
| **Slack** | Messages, slash commands, reactions | "Deploy to prod" commands |
| **PagerDuty** | Incidents, alerts | Auto-remediation workflows |
| **GitHub** | PRs, issues, commits, releases | Code review, CI/CD |
| **Jira** | Ticket created/updated | Sprint automation |
| **Discord** | Messages, commands | Team notifications |
| **Webhooks** | Any HTTP POST | Custom integrations |
| **Cron/Schedule** | Time-based | Daily reports, health checks |
| **Kafka** | Message streams | Event-driven automation |

### Tool Integrations (Actions)
*"Take action in any of these:"*

| Integration | Capabilities |
|-------------|--------------|
| **Slack** | Send messages, create channels, update threads |
| **PagerDuty** | Acknowledge, resolve, escalate incidents |
| **GitHub** | Create PRs, comment, merge, create issues |
| **Jira** | Create/update tickets, transitions |
| **Discord** | Send messages, embeds, reactions |
| **Shell** | kubectl, helm, docker, terraform, ansible |
| **HTTP** | Any REST API |
| **MCP Servers** | Extensible tool protocol |

### Integration Code Example
```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: incident-to-jira
spec:
  # Trigger: PagerDuty incident fires
  trigger:
    type: PagerDuty
    config:
      events: [incident.triggered]

  nodes:
    # AI diagnoses the issue
    - id: diagnose
      type: Agent
      config:
        agent: diagnostic-agent

    # Create Jira ticket with diagnosis
    - id: create-ticket
      type: Jira
      config:
        action: create_issue
        project: OPS
        type: Incident
        summary: "{{incident.title}}"
        description: "{{diagnose.output}}"

    # Notify team on Slack
    - id: notify
      type: Slack
      config:
        channel: "#incidents"
        message: "🚨 {{incident.title}} - Jira: {{create-ticket.url}}"

    # Also post to Discord
    - id: discord-notify
      type: Discord
      config:
        channel_id: "123456789"
        embed:
          title: "Incident: {{incident.title}}"
          color: 0xFF0000
```

### Integration Logos Grid
Display logos in a grid:
- Slack
- Discord
- PagerDuty
- OpsGenie
- GitHub
- GitLab
- Jira
- Linear
- Datadog
- Prometheus
- Grafana
- AWS
- GCP
- Azure

---

## Section 6: Use Cases

### Layout
- 4-column card grid
- Icons with titles and descriptions

### Headline
```
What Can You Build?
```

### Use Case Cards

**Card 1: Icon - Alert/Bell**
```
Title: Incident Response
Description: Auto-diagnose alerts, suggest fixes, execute
approved remediations. Integrate with PagerDuty, OpsGenie, Slack.
```

**Card 2: Icon - Code Review**
```
Title: PR Review Bots
Description: Automated code review for security, performance,
and best practices. Comments directly on GitHub PRs.
```

**Card 3: Icon - Chat Bubble**
```
Title: Slack/Teams Bots
Description: On-call assistants that answer questions,
run commands, and triage issues 24/7.
```

**Card 4: Icon - Chart/Report**
```
Title: Automated Reports
Description: Daily cluster health checks, cost reports,
security scans—delivered on schedule.
```

---

## Section 6: Architecture Overview

### Layout
- Centered diagram with explanatory text

### Headline
```
Simple, Modular Architecture
```

### Diagram (ASCII art or SVG)
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
│(OpenAI/Claude/ │  │(kubectl/git/db) │  │(Slack/PagerDuty)│
│ Gemini/Ollama) │  │                 │  │                 │
└────────────────┘  └─────────────────┘  └─────────────────┘
```

### Building Blocks (3 cards below diagram)

**Card 1: Agent**
```
Single AI assistant with specific instructions and tools.
Like a K8s Pod—the smallest deployable unit.
```

**Card 2: AgentFleet**
```
Team of agents working together. Multiple perspectives
on the same task. Like a K8s Deployment.
```

**Card 3: AgentFlow**
```
Visual workflow DAG for complex automation. Triggers,
conditions, human approvals. Like n8n or Argo Workflows.
```

---

## Section 7: Multi-Provider LLM Support

### Layout
- Logo grid with provider names

### Headline
```
Bring Your Own LLM
```

### Subheadline
```
Switch providers with one line. No code changes.
```

### Provider Grid (logos + names)
- OpenAI (GPT-4, GPT-4 Turbo)
- Anthropic (Claude 3.5 Sonnet, Opus)
- Google (Gemini 2.5 Flash, Pro)
- Ollama (Llama 3, Mistral, CodeLlama)
- Groq (Llama 3.1, Mixtral)
- AWS Bedrock (coming soon)

### Code Example
```yaml
# Switch providers instantly
spec:
  model: openai:gpt-4           # OpenAI
  model: anthropic:claude-3-5-sonnet-20241022  # Anthropic
  model: google:gemini-2.5-flash  # Google
  model: ollama:llama3          # Local (free!)
```

---

## Section 8: Code Example - Incident Response

### Layout
- Full-width code block with syntax highlighting
- Dark background

### Headline
```
Production-Ready in Minutes
```

### Subheadline
```
Complete incident response flow in 30 lines of YAML.
```

### Code Example
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
        agent: diagnostic-agent

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

---

## Section 9: Quick Start

### Layout
- 3-step numbered guide
- Terminal examples for each step

### Headline
```
Get Started in 5 Minutes
```

### Step 1
```
Title: Install aofctl
Code: cargo install aofctl
      # or
      curl -sSL https://aof.sh/install.sh | bash
```

### Step 2
```
Title: Create an Agent
Code: cat > agent.yaml <<EOF
      apiVersion: aof.dev/v1
      kind: Agent
      metadata:
        name: my-assistant
      spec:
        model: openai:gpt-4
        instructions: "You are a helpful DevOps assistant."
        tools:
          - shell
      EOF
```

### Step 3
```
Title: Run It
Code: export OPENAI_API_KEY=sk-...
      aofctl run agent agent.yaml
```

### CTA Button (large, centered, orange-700)
```
Read the Full Getting Started Guide →
```
Link: https://docs.aof.sh/docs/intro

---

## Section 10: Open Source

### Layout
- Centered content with GitHub stats

### Headline
```
Open Source. Apache 2.0.
```

### Description
```
AOF is fully open source under the Apache 2.0 license.
Use it commercially, modify it, contribute back.
```

### GitHub Stats (dynamic badges)
- Stars count
- Forks count
- Contributors count
- License badge

### CTA
```
Star on GitHub →
```
Link: https://github.com/agenticdevops/aof

---

## Section 11: Footer CTA

### Layout
- Full-width dark section
- Centered large CTA

### Headline
```
Ready to Build AI Agents the Ops Way?
```

### Subheadline
```
From zero to production agent in 5 minutes. No Python required.
```

### Primary CTA Button (large, orange-700)
```
Get Started →
```
Link: https://docs.aof.sh/docs/intro

### Secondary Links (smaller, below CTA)
- Documentation: https://docs.aof.sh
- GitHub: https://github.com/agenticdevops/aof
- Examples: https://docs.aof.sh/docs/examples

---

## Section 12: Footer

### Layout
- 4-column footer with links
- Copyright and social icons

### Column 1: Product
- Features
- Documentation
- Examples
- Pricing (Free & Open Source)

### Column 2: Resources
- Getting Started
- Tutorials
- API Reference
- CLI Reference

### Column 3: Community
- GitHub
- Discussions
- Contributing
- Code of Conduct

### Column 4: Company
- About
- Blog (coming soon)
- Contact
- Twitter/X

### Bottom Bar
```
© 2024 OpsFlow. Apache 2.0 License.
```

---

## Design System

### Colors
- **Background**: zinc-950 (#09090b), zinc-900 (#18181b)
- **Primary Accent**: orange-700 (#c2410c), orange-600 (#ea580c)
- **Text Primary**: white
- **Text Secondary**: zinc-400 (#a1a1aa)
- **Code Background**: zinc-800 (#27272a)
- **Success**: green-500
- **Links**: orange-500

### Typography
- **Headlines**: Inter or system font, bold
- **Body**: Inter or system font, regular
- **Code**: JetBrains Mono or Fira Code

### Components
- **Buttons**: Rounded corners (8px), orange gradient for primary
- **Code Blocks**: Dark background, syntax highlighting, copy button
- **Cards**: zinc-900 background, subtle border, hover effect
- **Icons**: Lucide icons (consistent with docs site)

---

## SEO Metadata

### Title
```
AOF - The AI Agent Framework for DevOps | Build Agents with YAML
```

### Description
```
Build AI agents with Kubernetes-style YAML. No Python required.
The only agent framework built for DevOps, SRE, and Platform Engineers.
Open source, Apache 2.0.
```

### Keywords
```
AI agents, DevOps automation, SRE tools, Kubernetes YAML,
LLM framework, incident response, agentic ops, MCP,
LangChain alternative, CrewAI alternative, Kagent alternative
```

### Open Graph
- Title: AOF - AI Agents for DevOps
- Description: Build AI agents with YAML. No Python required.
- Image: Social card with terminal preview

---

## Technical Notes for Builder.io

### Animations
- Hero terminal: Typing animation for YAML and bash commands
- Feature blocks: Fade-in on scroll
- Code examples: Syntax highlighting (use Prism.js or Shiki)

### Responsive Breakpoints
- Desktop: 1280px+
- Tablet: 768px - 1279px
- Mobile: < 768px

### Performance
- Lazy load images below fold
- Preload hero fonts
- Minimize JS bundle

### Analytics
- Track CTA clicks (Get Started, GitHub)
- Track scroll depth
- Track time on page

---

## Assets Needed

1. **Logo**: AOF logo in SVG (light and dark versions)
2. **Social Card**: 1200x630 Open Graph image
3. **Provider Logos**: OpenAI, Anthropic, Google, Ollama, Groq
4. **Architecture Diagram**: Clean SVG version
5. **Favicon**: 32x32 and 16x16 versions

---

## Key Messages to Emphasize

1. **"No Python Required"** - Repeat this often
2. **"YAML-First"** - Familiar to the target audience
3. **"kubectl-style"** - Instant recognition
4. **"Built for Ops, By Ops"** - Only framework with this focus
5. **"Runs Anywhere"** - No K8s lock-in (vs Kagent)
6. **"Single Binary"** - No dependency hell
7. **"Open Source"** - Apache 2.0, use commercially
8. **"Native Ops Integrations"** - Slack, Discord, PagerDuty, Jira, GitHub out of the box
9. **"Event-Driven Automation"** - Trigger from any ops tool, act on any ops tool

---

## Competitor Positioning Summary

| Competitor | AOF's Counter-Position |
|------------|----------------------|
| **LangChain** | "No Python required. YAML-first for ops teams." |
| **CrewAI** | "No Python. kubectl-style CLI you already know." |
| **Kagent** | "No Kubernetes required. Runs anywhere." |
| **Agno** | "Built for ops automation, not just chatbots." |
| **All Python frameworks** | "Single Rust binary. Zero dependencies." |

---

*Last Updated: December 2024*
