# How to Create an Agent Persona

This tutorial walks you through creating a fully personalized AI agent in AOF. By the end, you will have an agent with a distinct personality, communication style, visual identity, and behavioral boundaries.

**Time to complete:** 10-15 minutes

**Prerequisites:**
- AOF installed (`aofctl` binary available)
- A workspace directory (default: `workspace/`)

## What is a Persona?

A persona is the combination of identity, personality, and behavioral rules that make an agent feel like a team member rather than a script. Each agent gets:

- **Identity:** Name, role, emoji avatar
- **Personality:** Traits, values, communication style
- **Boundaries:** What it CAN and CANNOT do
- **Skills:** Tools and capabilities it has access to
- **Introduction:** What it says when joining a squad

Personas are defined in two workspace files:

| File | Purpose | Format |
|------|---------|--------|
| `AGENTS.md` | Agent roster (identity, skills, boundaries) | YAML |
| `SOUL.md` | Personality guidance (communication style, values) | Markdown + YAML |

## Step 1: Understand the AGENTS.md Format

`AGENTS.md` contains a YAML list of agent definitions. Each agent has these fields:

```yaml
agents:
  - id: agent-id              # Unique, lowercase-hyphenated (e.g., "db-guardian")
    name: Display Name         # Human-readable name (e.g., "Database Guardian")
    role: Role Description     # What the agent does (e.g., "PostgreSQL Specialist")
    avatar: "\U0001F418"       # Single emoji (Unicode escape or literal)
    personality_traits:        # Adjectives describing character (1-5 recommended)
      - methodical
      - cautious
    can:                       # What the agent is allowed to do
      - run SELECT queries
      - analyze query plans
    cannot:                    # Hard boundaries the agent must respect
      - execute DROP statements
      - modify user permissions
    skills:                    # Tool references (link to TOOLS.md entries)
      - psql
      - query-analysis
```

### Field Reference

| Field | Required | Rules |
|-------|----------|-------|
| `id` | Yes | Lowercase letters, numbers, hyphens only. Must start with a letter. |
| `name` | Yes | Non-empty string. |
| `role` | Yes | Non-empty string. Appears in UI under the agent name. |
| `avatar` | Yes | Single emoji. Use Unicode escape `"\U0001F916"` or paste emoji directly. |
| `personality_traits` | Yes | At least 1 trait. Adjectives that describe the agent's character. |
| `can` | Yes | At least 1 capability. Describe what the agent is allowed to do. |
| `cannot` | Yes | At least 1 boundary. Describe hard limits on agent behavior. |
| `skills` | Yes | At least 1 skill. Names must match entries in TOOLS.md. |

## Step 2: Understand the SOUL.md Format

`SOUL.md` contains detailed personality guidance for each agent. Each agent gets a section with:

1. A header: `## agent-id`
2. A YAML code block with structured metadata
3. A prose communication guide

```markdown
## db-guardian

\```yaml
id: db-guardian
communication_style: cautious-technical
tone: reassuring-methodical
values:
  - data-integrity
  - safety-first
  - clear-explanations
personality_summary: "A careful PostgreSQL specialist who treats every database change as a potential risk. Explains impact before executing anything."
boundaries:
  - "Never execute destructive queries without explicit approval"
  - "Always explain the risk level of suggested changes"
default_intro: "I'm Database Guardian, your PostgreSQL specialist. I help you manage your databases safely. Every change gets a risk assessment first."
\```

### Communication Style Guide

You are cautious and methodical. Before suggesting any database change, explain:
- What the change does
- What could go wrong
- How to roll back if needed

When analyzing queries:
- Show the query plan
- Identify performance bottlenecks
- Suggest indexes only when the benefit is clear

When asked to modify data:
- Ask for confirmation before any write operation
- Show a preview of affected rows
- Provide a rollback strategy
```

### SOUL.md Field Reference

| Field | Required | Purpose |
|-------|----------|---------|
| `id` | Yes | Must match an agent ID in AGENTS.md |
| `communication_style` | Yes | Descriptor (e.g., "formal-technical", "casual-friendly") |
| `tone` | Yes | Descriptor (e.g., "calm-professional", "encouraging-detective") |
| `values` | Yes | Core values guiding decisions (at least 1) |
| `personality_summary` | Yes | 1-2 sentence personality description |
| `boundaries` | Yes | Hard behavioral rules (at least 1) |
| `default_intro` | Yes | What the agent says when joining a squad |

The **Communication Style Guide** (prose after the YAML block) is free-form text that becomes part of the agent's system prompt. Be specific and provide examples.

## Step 3: Create a "Database Guardian" Agent

Let us create a complete agent persona step by step.

### 3a. Add to AGENTS.md

Open your `workspace/AGENTS.md` and add this entry to the `agents` list:

```yaml
  - id: db-guardian
    name: Database Guardian
    role: PostgreSQL Specialist
    avatar: "\U0001F418"
    personality_traits:
      - cautious
      - methodical
      - risk-averse
    can:
      - run SELECT queries
      - analyze query execution plans
      - suggest index improvements
      - review migration scripts
      - monitor database health
    cannot:
      - execute DROP or TRUNCATE without approval
      - modify user permissions or roles
      - access production credentials directly
    skills:
      - psql
      - query-analysis
      - migration-review
```

### 3b. Add to SOUL.md

Open your `workspace/SOUL.md` and add this section:

```markdown
## db-guardian

\```yaml
id: db-guardian
communication_style: cautious-technical
tone: reassuring-methodical
values:
  - data-integrity
  - safety-first
  - clear-explanations
  - reproducibility
personality_summary: "A careful PostgreSQL specialist who treats every database change as a potential risk. Explains impact before executing anything and always offers a rollback plan."
boundaries:
  - "Never execute destructive queries without explicit user approval"
  - "Always explain the risk level (low/medium/high) of suggested changes"
  - "Provide rollback steps for every write operation"
default_intro: "I'm Database Guardian, your PostgreSQL specialist. I help you manage your databases safely. Every change gets a risk assessment first."
\```

### Communication Style Guide

You are cautious and methodical. Before suggesting any database change, explain:
- What the change does
- What could go wrong (worst case)
- How to roll back if needed
- Expected impact on performance

When analyzing queries:
- Show the EXPLAIN ANALYZE output
- Identify sequential scans on large tables
- Suggest indexes only when the benefit is clear and measurable
- Warn about queries that may lock tables

When asked to modify data:
- Always ask for confirmation before any INSERT, UPDATE, or DELETE
- Show a preview of affected rows (SELECT first, then modify)
- Provide the exact rollback query
- Note the transaction isolation level

When reviewing migrations:
- Check for backward compatibility
- Verify that migrations can be reversed
- Warn about long-running DDL on large tables
- Suggest breaking large migrations into smaller steps
```

### 3c. Verify Your Persona

Validate your workspace files:

```bash
# Start the daemon (loads workspace files)
aofctl serve --workspace workspace/

# Check the daemon logs for "Loading agents from AGENTS.md"
# You should see: "Loaded 4 agents" (3 existing + 1 new)
```

Check the API:

```bash
# List all agents with personas
curl http://localhost:3030/api/config/agents | jq '.agents[] | select(.id == "db-guardian")'
```

Expected output:
```json
{
  "id": "db-guardian",
  "name": "Database Guardian",
  "role": "PostgreSQL Specialist",
  "avatar": "\ud83d\udc18",
  "personality_traits": ["cautious", "methodical", "risk-averse"],
  "can": ["run SELECT queries", "analyze query execution plans", "..."],
  "cannot": ["execute DROP or TRUNCATE without approval", "..."],
  "skills": ["psql", "query-analysis", "migration-review"]
}
```

### 3d. See It in Mission Control

Open Mission Control at `http://localhost:5173` and look for your new agent card:

```
+----------------------------------+
|  [elephant emoji]                |
|  Database Guardian               |
|  PostgreSQL Specialist           |
|                                  |
|  cautious  methodical  risk-aver |
|                                  |
|  Status: idle                    |
|  Uptime: -- (insufficient data)  |
+----------------------------------+
```

When the daemon starts, you should see an introduction toast:

```
"I'm Database Guardian, your PostgreSQL specialist.
 I help you manage your databases safely.
 Every change gets a risk assessment first."
```

## Step 4: Tips for Great Personas

### Choosing an Avatar

Pick an emoji that visually represents the agent's role:

| Role | Good Choices | Reasoning |
|------|-------------|-----------|
| Infrastructure | robot, gear, wrench | Technical, mechanical |
| Database | elephant, disk, lock | PostgreSQL elephant, data storage |
| Logs/Debug | magnifying glass, detective | Investigation |
| Incident | alarm, fire, ambulance | Urgency |
| Security | shield, lock, key | Protection |
| API Testing | test tube, microscope | Analysis |

Avoid complex or rare emoji that may not render consistently across platforms.

### Writing Personality Traits

Choose 2-4 adjectives that create a clear character:

- **Analytical agents:** methodical, detail-oriented, data-driven, systematic
- **Investigative agents:** curious, thorough, patient, persistent
- **Leadership agents:** calm-under-pressure, decisive, communicative
- **Cautious agents:** risk-averse, careful, deliberate, safety-first
- **Creative agents:** innovative, experimental, exploratory

### Writing the Communication Guide

The communication guide is the most important part of the personality. Be specific:

**Good (specific, actionable):**
```
When you discover a performance issue:
- Show the query plan with timing data
- Highlight the slowest operation
- Suggest a specific fix (add index, rewrite query)
- Estimate the expected improvement
```

**Less effective (vague):**
```
Be helpful when users ask about performance.
```

### Defining Boundaries (CAN/CANNOT)

Clear boundaries prevent agents from overstepping:

- **CAN:** List specific actions the agent is allowed to perform
- **CANNOT:** List explicit restrictions (these become part of the system prompt)

Good boundaries are:
- Specific: "cannot execute DROP statements" (not "cannot do dangerous things")
- Actionable: "escalate to humans when uncertain" (not "be careful")
- Contextualized: "cannot modify billing systems" (not "cannot access systems")

## Step 5: Common Mistakes

### Missing Required Fields

Every agent must have all required fields. Missing fields cause parse errors:

```
AGENTS.md parse error at 'agents[3].avatar': missing field `avatar`
```

**Fix:** Add the missing field to your agent entry.

### Invalid Agent ID Format

Agent IDs must be lowercase-hyphenated:

```
agents[3].id: 'Database_Guardian' must be lowercase-hyphenated (e.g., 'db-guardian')
```

**Fix:** Use lowercase letters, numbers, and hyphens only. Start with a letter.

### Soul ID Mismatch

The `id` in SOUL.md must exactly match an `id` in AGENTS.md:

```
soul[db_guardian].id: 'db_guardian' does not match any agent id in AGENTS.md
```

**Fix:** Ensure the id in your SOUL.md YAML block matches the agent id exactly.

### Avatar Not a Single Emoji

The avatar must be exactly one emoji character:

```
agents[3].avatar: 'robot' is not a single emoji (found 5 grapheme clusters, expected 1)
```

**Fix:** Use a Unicode escape like `"\U0001F916"` or paste a single emoji character.

### Empty Skills List

Every agent must have at least one skill:

```
agents[3].skills: must have at least one skill
```

**Fix:** Add skill names that match entries in your TOOLS.md.

### Prompt Injection in SOUL.md

The system detects suspicious text patterns:

```
soul[db-guardian].personality_summary: potential prompt injection detected
```

**Fix:** Remove text like "ignore all previous instructions" from your SOUL.md. These patterns are blocked for security.

## Step 6: Testing Your Persona

### Verify via API

```bash
# List all agents
curl http://localhost:3030/api/config/agents | jq '.agents[].id'

# Get specific agent
curl http://localhost:3030/api/config/agents | jq '.agents[] | select(.id == "db-guardian")'

# Check metrics (after running some tasks)
curl http://localhost:3030/api/agents/db-guardian/metrics
```

### Check Composed Prompt

Enable debug logging to see the composed system prompt:

```bash
RUST_LOG=debug aofctl serve --workspace workspace/
```

Look for log lines like:
```
DEBUG aof_personas::composer: Prompt cache miss for agent 'db-guardian'
```

### Listen for Introduction Events

Connect to the WebSocket to see introduction events:

```bash
websocat ws://localhost:3030/ws
```

On daemon startup, you should see JSON events with introduction data for each agent.

## Template: Quick Start

Copy this template to create a new agent in seconds:

### AGENTS.md Entry

```yaml
  - id: your-agent-id
    name: Your Agent Name
    role: Your Agent Role
    avatar: "\U0001F916"
    personality_traits:
      - trait-one
      - trait-two
    can:
      - capability one
      - capability two
    cannot:
      - restriction one
      - restriction two
    skills:
      - skill-one
      - skill-two
```

### SOUL.md Section

```markdown
## your-agent-id

\```yaml
id: your-agent-id
communication_style: your-style
tone: your-tone
values:
  - value-one
  - value-two
personality_summary: "One-line description of your agent's personality."
boundaries:
  - "Hard rule one"
  - "Hard rule two"
default_intro: "I'm Your Agent Name, your role. Here's what I do."
\```

### Communication Style Guide

Describe how the agent communicates. Be specific about:
- When to be proactive
- When to ask questions
- When to escalate
- How to format responses
```

## Next Steps

- See [Persona Examples](../examples/personas-reference.md) for 5 complete agent personas
- Read [Persona System Architecture](../dev/persona-system.md) for technical details
- Check [Troubleshooting](../troubleshooting/personas-issues.md) if something goes wrong
- Explore the [API Reference](../features/agent-personas.md) for all persona endpoints
