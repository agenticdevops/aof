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

## API Reference

```rust
use aof_personas::{AgentLoader, SoulLoader, validate_personas, AgentCache};

// Load from files
let agents = AgentLoader::load_from_file("workspace/AGENTS.md").await?;
let souls = SoulLoader::load_from_file("workspace/SOUL.md").await?;

// Validate
validate_personas(&agents, &souls)?;

// Use caching for repeated access
let cache = AgentCache::new();
let agents = cache.load_agents("workspace/AGENTS.md").await?;
let souls = cache.load_souls("workspace/SOUL.md").await?;
```
