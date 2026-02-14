# Conversational Agent Creation

Create agents by describing what you need in natural language. AOF generates validated configuration files ready for your workspace.

## Overview

Instead of manually writing YAML configuration, describe your agent in plain English:

**You:** "I need a K8s monitoring agent that can check cluster health"

**AOF:** Generates a complete agent with:
- Valid AGENTS.md YAML entry (id, role, skills, capabilities)
- SOUL.md personality file (communication style, tone, values)
- Preview of both files before writing
- Confirmation step so you can review and approve

No YAML syntax to learn. No looking up available skills. Just describe what you need.

## How It Works

### Step-by-Step Flow

1. **Describe Your Agent**
   - Tell AOF what the agent should do
   - Mention the domain or problem area
   - Optionally specify skills if you know them

2. **System Generates Configuration**
   - Classifies your intent (agent creation)
   - Loads available skills from your workspace
   - Generates AGENTS.md YAML entry
   - Creates matching SOUL.md personality

3. **Preview Files**
   - See exactly what will be created
   - AGENTS.md shows agent metadata (name, role, skills)
   - SOUL.md shows personality (communication style, tone)

4. **Confirm or Cancel**
   - Approve to write files to your workspace
   - Cancel to discard and try again

5. **Agent Created**
   - Files written to workspace
   - Agent immediately available for execution

## Examples

### Example 1: Kubernetes Monitor

**User:**
```
I need a K8s monitoring agent that watches cluster health and alerts on issues
```

**Generated AGENTS.md Entry:**
```yaml
id: k8s-health-monitor
name: Kubernetes Health Monitor
role: Infrastructure Watchdog
avatar: 🔍
personality_traits:
  - vigilant
  - detail-oriented
  - proactive
can:
  - monitor cluster health metrics
  - detect anomalies in pod status
  - alert on threshold violations
  - analyze node resource usage
cannot:
  - modify production deployments
  - delete resources
  - execute kubectl apply commands
skills:
  - k8s-diagnostics
  - metrics-analysis
```

**Generated SOUL.md:**
```markdown
```yaml
id: k8s-health-monitor
communication_style: formal-technical
tone: calm-professional
values:
  - reliability
  - accuracy
  - transparency
personality_summary: A methodical agent focused on cluster stability
boundaries:
  - Never modify production without approval
  - Always verify data before alerting
default_intro: Hello, I'm monitoring your Kubernetes cluster for health issues
```

## Communication Style

I communicate with precision and clarity, reporting observations in a factual manner.
When I detect anomalies, I provide specific metrics and context to help you understand
the issue. I remain calm even during critical alerts, ensuring you have the information
needed to make informed decisions.
```

### Example 2: Log Analyzer

**User:**
```
Build me an agent that can parse application logs and find errors
```

**Generated:**
```yaml
id: log-analyzer
name: Application Log Analyzer
role: Diagnostic Specialist
avatar: 📝
personality_traits:
  - analytical
  - thorough
  - pattern-focused
can:
  - parse structured and unstructured logs
  - identify error patterns
  - correlate events across log streams
  - generate diagnostic reports
cannot:
  - modify log files
  - delete logs
  - change logging configuration
skills:
  - log-parser
  - pattern-matching
```

### Example 3: Incident Responder

**User:**
```
I want an incident response agent for production outages
```

**Generated:**
```yaml
id: incident-responder
name: Production Incident Responder
role: Emergency Coordinator
avatar: 🚨
personality_traits:
  - decisive
  - systematic
  - cool-headed
  - escalation-aware
can:
  - coordinate incident response
  - gather diagnostic data
  - communicate status updates
  - document incident timeline
cannot:
  - make production changes without approval
  - bypass change control processes
skills:
  - incident-triage
  - k8s-diagnostics
  - log-parser
```

### Example 4: Custom Domain Agent

**User:**
```
Create an agent that helps with database performance tuning for Postgres
```

**Generated:**
```yaml
id: postgres-tuner
name: PostgreSQL Performance Tuner
role: Database Optimization Specialist
avatar: 🐘
personality_traits:
  - performance-focused
  - data-driven
  - patient
  - methodical
can:
  - analyze query execution plans
  - recommend index optimizations
  - identify slow queries
  - suggest configuration improvements
cannot:
  - execute DDL statements
  - modify production data
  - change database configuration directly
skills:
  - postgres-diagnostics
  - query-analysis
```

## What Gets Generated

Each agent creation produces two files:

### AGENTS.md Entry

The agent roster entry contains:

| Field | Description | Example |
|-------|-------------|---------|
| `id` | Unique lowercase-hyphenated identifier | `k8s-monitor` |
| `name` | Human-readable name | `Kubernetes Monitor` |
| `role` | Agent's primary function | `Infrastructure Specialist` |
| `avatar` | Single emoji for UI display | 🔍 |
| `personality_traits` | 3-5 adjectives describing behavior | `[vigilant, methodical]` |
| `can` | List of 3-4 capabilities | `[monitor clusters, ...]` |
| `cannot` | List of 2-3 boundaries | `[modify production, ...]` |
| `skills` | Available skills from your workspace | `[k8s-diagnostics]` |

### SOUL.md Personality

The personality file contains:

**YAML Frontmatter:**
- `id`: Must match AGENTS.md
- `communication_style`: How the agent communicates (formal-technical, casual-friendly, etc.)
- `tone`: Emotional quality (calm-professional, warm-encouraging, etc.)
- `values`: Core principles (reliability, accuracy, etc.)
- `personality_summary`: One-sentence description
- `boundaries`: Communication and behavioral limits
- `default_intro`: Opening message when agent starts

**Prose Section:**
- `## Communication Style`: 2-3 paragraphs describing how the agent interacts
- Details communication patterns, reporting style, and interaction approach

## Editing Generated Files

After generation, you can manually edit either file:

### Modify AGENTS.md

1. Open `workspace/AGENTS.md`
2. Find the generated agent entry
3. Adjust fields as needed (add skills, change role, etc.)
4. Save file

Changes take effect immediately. AOF validates on load, so invalid YAML will be caught.

### Modify SOUL.md

1. Open `workspace/SOUL.md`
2. Find the section for your agent (search by `id: agent-name`)
3. Edit frontmatter or prose as needed
4. Save file

Personality changes affect how the agent communicates but don't change its capabilities.

### Power User Workflow

For teams with YAML expertise:

1. Use conversational creation to bootstrap initial agent
2. Review generated files
3. Fine-tune manually for specific requirements
4. Commit to version control

This combines rapid prototyping (conversational) with precise control (manual editing).

## Available Skills

Skills determine what tools and capabilities an agent has access to. AOF only generates agents with skills that exist in your workspace.

### See Available Skills

```bash
aofctl skills list
```

This shows all loaded skills from:
- Workspace (`workspace/skills/`)
- Enterprise registry (if configured)
- Bundled skills (shipped with AOF)

### Skill Categories

Common skill categories:

- **Kubernetes**: `k8s-diagnostics`, `k8s-monitor`, `helm-ops`
- **Logging**: `log-parser`, `log-aggregation`
- **Metrics**: `metrics-analysis`, `prometheus-query`
- **Incidents**: `incident-triage`, `runbook-executor`
- **Databases**: `postgres-diagnostics`, `mysql-tuning`
- **Custom**: Your organization's domain-specific skills

### Teaching New Skills

If the agent needs a skill that doesn't exist:

**You:** "Teach AOF how to debug Redis connection pools"

AOF will guide you through creating a new skill file. Once created, future agents can use it.

See: [Conversational Skill Teaching](./conversational-skill-teaching.md) (Phase 6, Plan 3)

## Troubleshooting

### Agent Not Created

**Symptom:** AOF returns an error instead of generating files.

**Common Causes:**

1. **Duplicate ID**
   - Error: `Duplicate agent ID: existing-agent`
   - Solution: Your workspace already has an agent with that ID. Describe a different agent or manually edit AGENTS.md to remove the duplicate.

2. **Invalid Skills**
   - Error: `Skill not found: custom-debugger. Similar: [debugger, log-debugger]`
   - Solution: The LLM generated a skill that doesn't exist. Check `aofctl skills list` for similar skills, or teach AOF the new skill.

3. **Validation Failed**
   - Error: `Invalid emoji avatar: ABC`
   - Solution: The generated agent used text instead of an emoji. Try again or manually fix the YAML.

### Files Not Written

**Symptom:** Files previewed but not saved to workspace.

**Cause:** You need to confirm the preview.

**Solution:**
```
AOF: Review the files below and confirm to add this agent.
You: yes / confirm
```

If you canceled (`no` / `cancel`), the files are discarded. Start over to regenerate.

### Agent Behavior Wrong

**Symptom:** Agent created successfully but doesn't behave as expected.

**Cause:** Personality or skills don't match the task.

**Solution:**
1. Check SOUL.md - does the communication style fit?
2. Check AGENTS.md `skills` - does it have the right tools?
3. Edit files manually or regenerate with more specific description

### Hallucinated Skills

**Symptom:** Generated agent includes skills that don't exist.

**Cause:** LLM invented plausible-sounding skill names.

**Protection:** AOF auto-detects and removes hallucinated skills. If ALL skills were invalid, you'll get an error with the list of available skills.

**Solution:**
- Use skills from `aofctl skills list`
- Teach AOF the new skill if needed
- Describe the agent with more domain context so LLM selects better matches

## Next Steps

- **Build a Squad:** Once you have multiple agents, [create coordinating squads](./conversational-squad-building.md)
- **Schedule Tasks:** [Set up triggers and schedules](./conversational-schedule-config.md) for automated operations
- **Teach Skills:** [Add custom skills](./conversational-skill-teaching.md) for domain-specific capabilities

## FAQ

**Q: Can I create multiple agents at once?**

A: Not yet. Create one agent at a time, confirm, then create the next. Squad building (Phase 6, Plan 3) allows batch creation of coordinating teams.

**Q: How do I delete an agent?**

A: Manually edit `workspace/AGENTS.md` and remove the agent's YAML entry. Also remove the corresponding section from `SOUL.md` if present.

**Q: Can agents modify their own personalities?**

A: No. SOUL.md is static configuration. Agents read it at startup but can't modify it at runtime.

**Q: What if I don't like the generated personality?**

A: Edit `SOUL.md` manually or regenerate the agent with more specific instructions (e.g., "Create a friendly, casual K8s monitor" vs "Create a K8s monitor").

**Q: Are skills required?**

A: Technically no (skills list can be empty), but agents without skills can't execute tools. They'll be limited to conversation and reasoning.

**Q: Can I use this in CI/CD?**

A: Conversational creation is interactive. For automated workflows, write AGENTS.md and SOUL.md directly or template them from a script.
