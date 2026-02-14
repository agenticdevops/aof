# Agent Persona System

The AOF persona system gives agents distinct identities, communication styles, and behavioral boundaries. Instead of generic AI assistants, your agents become recognizable team members with consistent personalities.

## Overview

Personas are defined in two workspace files:

- **AGENTS.md** -- Agent roster with identity, capabilities, and skills
- **SOUL.md** -- Personality guidance with communication style, values, and boundaries

These files are human-editable, version-controlled, and loaded at runtime by the AOF daemon.

## AGENTS.md Format

The agent roster is a YAML file defining all agents in your squad:

```yaml
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    avatar: "\U0001F916"
    personality_traits:
      - methodical
      - detail-oriented
      - proactive
    can:
      - kubectl operations
      - pod debugging
      - log analysis
    cannot:
      - modify cluster RBAC
      - delete persistent volumes without approval
    skills:
      - kubectl
      - pod-debugging
      - log-analysis
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique lowercase-hyphenated identifier (e.g., `k8s-monitor`) |
| `name` | Yes | Display name shown in UI and messages |
| `role` | Yes | Role description (e.g., "Infrastructure Specialist") |
| `avatar` | Yes | Single emoji character for visual identity |
| `personality_traits` | Yes | List of adjectives describing the agent's character |
| `can` | Yes | List of capabilities the agent is allowed to perform |
| `cannot` | Yes | List of boundaries the agent must respect |
| `skills` | Yes | List of tool/skill references (link to available tools) |

### Validation Rules

- IDs must be lowercase-hyphenated (`a-z`, `0-9`, `-`)
- No duplicate IDs across agents
- Avatar must be a single emoji (Unicode grapheme cluster)
- All list fields must have at least one entry

## SOUL.md Format

The personality guide uses Markdown with YAML frontmatter per agent:

```markdown
## k8s-monitor

\`\`\`yaml
id: k8s-monitor
communication_style: formal-technical
tone: calm-professional
values:
  - system-stability
  - transparency
personality_summary: "A methodical Kubernetes specialist."
boundaries:
  - "Never suggest changes that trade stability for speed"
default_intro: "I'm Kubernetes Monitor, your infrastructure specialist."
\`\`\`

### Communication Style Guide

You are methodical and data-driven. You favor precision over speed.
When you discover issues, explain them clearly with context.
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Must match an agent ID from AGENTS.md |
| `communication_style` | Yes | Style descriptor (e.g., `formal-technical`) |
| `tone` | Yes | Tone descriptor (e.g., `calm-professional`) |
| `values` | Yes | Core values guiding agent decisions |
| `personality_summary` | Yes | One-line personality description |
| `boundaries` | Yes | Hard behavioral rules for this agent |
| `default_intro` | Yes | Message when agent joins a squad |

### Communication Guide

The prose section after the YAML block provides detailed guidance for the agent's communication style. This text is included in the system prompt and should describe:

- How the agent communicates (style, structure, verbosity)
- When to be proactive vs. reactive
- When to escalate to humans
- How to handle uncertainty

## Safety Features

### Prompt Injection Detection

The persona system scans all text fields for prompt injection attempts:

- "ignore all previous instructions"
- "forget instructions"
- "disregard ... prompt"
- "override system"

If injection is detected, the file fails validation with a clear error message identifying the offending field.

### Validation on Load

Every time persona files are loaded (startup or file change), the system validates:

1. All required fields are present
2. IDs follow the correct format
3. Avatars are valid emoji
4. Soul IDs match agent IDs (reference integrity)
5. No prompt injection in text fields

Errors include the exact field path (e.g., `agents[0].avatar`) for easy debugging.

## File Watching

The daemon watches AGENTS.md and SOUL.md for changes. When a file is modified:

1. The file is re-read and parsed
2. Validation runs on the new content
3. If valid, agents receive updated personas
4. If invalid, the old personas remain active and an error is logged

Changes are coalesced (rapid writes produce a single reload event).

## Caching

Loaded personas are cached in memory using SHA256 content hashing. The cache is only invalidated when file content actually changes, avoiding unnecessary re-parsing on repeated access.

## Architecture

```
workspace/AGENTS.md --> AgentLoader --> Vec<Agent>
                                             |
workspace/SOUL.md  --> SoulLoader  --> HashMap<String, Soul>
                                             |
                                     validate_personas()
                                             |
                                     PersonaWatcher (file watch)
                                             |
                                     PersonaUpdate event
```

## Example: Creating a New Agent

1. Add the agent to `workspace/AGENTS.md`:

```yaml
  - id: my-agent
    name: My Custom Agent
    role: Custom Role
    avatar: "\U0001F9D9"
    personality_traits:
      - creative
      - thorough
    can:
      - perform custom operations
    cannot:
      - access production without approval
    skills:
      - custom-tool
```

2. Add personality guidance to `workspace/SOUL.md`:

```markdown
## my-agent

\`\`\`yaml
id: my-agent
communication_style: casual-friendly
tone: enthusiastic
values:
  - creativity
  - user-experience
personality_summary: "An enthusiastic agent that loves building creative solutions."
boundaries:
  - "Always ask before making breaking changes"
default_intro: "Hey! I'm your custom agent, ready to help build something awesome."
\`\`\`

### Communication Style Guide

You are enthusiastic and creative. You suggest multiple approaches and explain tradeoffs clearly.
```

3. The daemon detects the change and loads the new agent automatically.

## Integration with System Prompts

Persona data flows into system prompt composition (Phase 5-02). The composed prompt layers:

1. Base instructions
2. Role definition (from AGENTS.md)
3. Personality and values (from SOUL.md)
4. Communication style (from SOUL.md)
5. Capabilities and boundaries (from AGENTS.md)
6. Available tools (from skills mapping)

## Introduction Events

When the daemon starts, each agent emits an introduction event. These events:

- Flow through the Phase 1 broadcast channel
- Appear in WebSocket connections (Mission Control UI)
- Route to messaging platforms (Slack, Discord) via the gateway
- Use the `default_intro` from SOUL.md as the introduction message

### Event Structure

Introduction events are `CoordinationEvent` instances with an `introduction` field:

```json
{
  "event_id": "uuid",
  "agent_id": "k8s-monitor",
  "timestamp": "2026-02-14T10:30:00Z",
  "activity": { "activity_type": "Info", "message": "Kubernetes Monitor introduced: ..." },
  "introduction": {
    "agent_id": "k8s-monitor",
    "agent_name": "Kubernetes Monitor",
    "role": "Infrastructure Specialist",
    "avatar": "\u{1F916}",
    "intro_message": "I'm Kubernetes Monitor, your infrastructure specialist...",
    "personality_summary": "A methodical specialist...",
    "skills": ["kubectl", "pod-debugging", "log-analysis", "alerting"]
  }
}
```

### Squad-Specific Introductions

Create an optional `workspace/squads.yaml` to customize introductions per squad:

```yaml
squads:
  - name: incident-response
    agents:
      - id: incident-responder
        intro_override: "Incident mode activated. I'm coordinating the response."
      - id: log-analyzer
        intro_override: "Standing by to dig into the incident logs."
      - id: k8s-monitor
        # No override - uses SOUL.md default_intro
```

When `squads.yaml` is present and an agent has an `intro_override`, it replaces the `default_intro` from SOUL.md. If the file is missing or an agent has no override, the SOUL.md value (or fallback) is used.

### Fallback Behavior

When no SOUL.md entry exists for an agent, the introduction uses a fallback:

```
"I'm [agent name], your [agent role]."
```

For example: "I'm Log Analyzer, your Debugging Expert."

## Reliability Metrics

The persona system includes computed reliability metrics for each agent, derived from the Phase 1 event stream. These metrics help users calibrate trust in agent recommendations.

### Metrics Computed

| Metric | Formula | Description |
|--------|---------|-------------|
| **Uptime %** | (total - errors) / total * 100 | Percentage of events that were not errors |
| **Success Rate %** | completed / total * 100 | Percentage of events that completed successfully |

### Minimum Data Threshold

Metrics require at least **10 events** before displaying percentages. Below this threshold, the UI shows "--" instead of potentially misleading numbers.

### Color Coding

Metrics are color-coded in the Mission Control UI:

| Range | Color | Meaning |
|-------|-------|---------|
| >= 95% | Green | Excellent reliability |
| 80-94% | Yellow | Good, but watch for degradation |
| 60-79% | Orange | Degraded, investigate |
| < 60% | Red | Poor reliability, action needed |

### API Endpoint

```bash
# Get metrics for a specific agent
curl http://localhost:8080/api/agents/k8s-monitor/metrics
```

Response:

```json
{
  "agent_id": "k8s-monitor",
  "uptime_percent": 95.5,
  "success_rate": 92.0,
  "event_count": 50,
  "last_update": "2026-02-14T10:30:00Z",
  "last_error": null
}
```

### Real-Time Updates

Metrics update automatically as new events arrive:

1. Events broadcast through the Phase 1 broadcast channel
2. A background subscriber updates the `ReliabilityCache`
3. The Mission Control UI polls `/api/agents/:id/metrics` every 5 seconds
4. AgentCard badges update with new values and color coding

### Rust API

```rust
use aof_personas::{ReliabilityCache, ReliabilityMetrics, compute_agent_metrics};

// Create a cache (auto-subscribed to event bus in serve.rs)
let cache = ReliabilityCache::default_capacity();

// Manual computation from a list of events
let metrics = compute_agent_metrics("k8s-monitor", &events);

// Cache-based access
let metrics = cache.get_metrics("k8s-monitor").await;
```

## API Reference

```rust
use aof_personas::{AgentLoader, SoulLoader, validate_personas, AgentCache};
use aof_personas::events::{build_introduction_event, build_introduction_event_batch};

// Load from files
let agents = AgentLoader::load_from_file("workspace/AGENTS.md").await?;
let souls = SoulLoader::load_from_file("workspace/SOUL.md").await?;

// Validate
validate_personas(&agents, &souls)?;

// Build introduction events for all agents
let events = build_introduction_event_batch(&agents, &souls, &session_id);
for event in events {
    event_bus.emit(event);
}

// Use caching for repeated access
let cache = AgentCache::new();
let agents = cache.load_agents("workspace/AGENTS.md").await?;
let souls = cache.load_souls("workspace/SOUL.md").await?;
```
