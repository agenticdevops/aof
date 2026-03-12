# Agent Directory Structure Specification

**Spec version:** 0.1.0 (GitAgent-compatible)
**Format:** Directory-based agent definition

---

## Overview

In OpenAgentiX, an agent definition is a **directory**, not a monolithic YAML file. This format is
GitAgent-compatible: the directory can be committed to version control, extended via `extends`, and
composed with sub-agents via an `agents/` subdirectory.

The minimal manifest (`agent.yaml`) carries only metadata and model preference. All behavior lives
in markdown files (`SOUL.md`, `RULES.md`, `skills/`) that the runtime assembles into a system prompt
at load time.

This document is the **primary format reference**. See [agent-yaml-v1.md](agent-yaml-v1.md) for the
`agent.yaml` manifest field reference, and [workspace-config.md](workspace-config.md) for workspace
discovery configuration.

---

## Directory Layout

```
agents/my-agent/
├── agent.yaml        # Required: minimal manifest (metadata + model only)
├── SOUL.md           # Required: identity and system prompt
├── RULES.md          # Optional: hard constraints the agent must never violate
├── AGENTS.md         # Optional: framework-agnostic fallback prompt
├── skills/           # Optional: reusable capability modules
│   └── skill-name/
│       ├── SKILL.md  # Skill instructions (appended to system prompt)
│       └── scripts/  # Optional: helper scripts for this skill
├── tools/            # Optional: MCP-compatible tool definitions (YAML schemas)
│   └── kubectl.yaml  # One file per tool
├── workflows/        # Optional: multi-step procedure definitions
├── knowledge/        # Optional: reference documents injected into context
│   └── runbook.md
├── memory/
│   └── runtime/      # Optional: live state files (dailylog.md, context.md)
├── hooks/            # Optional: lifecycle hooks
│   ├── bootstrap.md  # Instructions run before agent starts
│   └── teardown.md   # Instructions run after agent completes
├── agents/           # Optional: sub-agent definitions (recursive — multi-agent)
│   ├── sub-agent-a/
│   │   ├── agent.yaml
│   │   └── SOUL.md
│   └── sub-agent-b/
│       ├── agent.yaml
│       └── SOUL.md
├── examples/         # Optional: few-shot examples for calibration
└── .gitagent/        # Runtime state (gitignored by OpenAgentiX)
```

---

## Required Files

The agent **fails to load** if either of these is missing or empty:

| File | Description |
|------|-------------|
| `agent.yaml` | Minimal manifest — name, spec version, optional model preference |
| `SOUL.md` | Identity and system prompt — the agent's persona, goals, and capabilities |

---

## Optional Files and Directories

| Path | Description |
|------|-------------|
| `RULES.md` | Hard constraints the agent must never violate (appended to system prompt) |
| `AGENTS.md` | Framework-agnostic fallback prompt (for use by other agent frameworks) |
| `skills/` | Reusable capability modules; each subdirectory is one skill |
| `tools/` | Tool definitions; each `.yaml` file defines one tool |
| `workflows/` | Multi-step procedure definitions |
| `knowledge/` | Reference documents (Markdown) injected as context |
| `memory/runtime/` | Live state files written and read by the agent during a run |
| `hooks/bootstrap.md` | Instructions executed before the agent starts a run |
| `hooks/teardown.md` | Instructions executed after the agent completes a run |
| `agents/` | Sub-agent definitions (recursive — enables multi-agent hierarchies) |
| `examples/` | Few-shot calibration examples |
| `.gitagent/` | Runtime state directory (not committed — auto-added to `.gitignore`) |

---

## How the Runtime Assembles the System Prompt

The runtime constructs a single system prompt string from the agent directory contents. Assembly
order is deterministic:

1. **Base:** Read `SOUL.md` as the primary system prompt content.

2. **Constraints:** If `RULES.md` exists and is non-empty, append its content after a `## Constraints` header:
   ```
   ## Constraints
   <content of RULES.md>
   ```

3. **Skills:** For each skill directory in `skills/` (sorted alphabetically):
   - Read `SKILL.md`
   - Append after a `## Skill: <skill-name>` header:
     ```
     ## Skill: postgres-tuning
     <content of skills/postgres-tuning/SKILL.md>
     ```

4. **Knowledge:** If `knowledge/` contains Markdown files (`.md`), append their contents after a
   `## Reference Material` header. Files are read in alphabetical order.

5. **Bootstrap hook:** If `hooks/bootstrap.md` exists, append after a `## Pre-Run Instructions` header.

The resulting assembled text becomes the `system_prompt` passed to the LLM for every run. The
runtime does not cache this assembly between runs — it is re-assembled fresh on each invocation so
that edits to any source file take effect immediately.

---

## skills/ Directory

Each subdirectory in `skills/` is a **skill module**. Skills are composable capability units — an
agent can have multiple skills, and each adds specialized instructions to the assembled system
prompt.

**Skill directory layout:**

```
skills/postgres-tuning/
├── SKILL.md    # Required: domain instructions for this skill
└── scripts/    # Optional: helper scripts referenced by this skill
    └── explain-analyze.sh
```

The `SKILL.md` file contains domain-specific instructions. Example:

```markdown
## PostgreSQL Performance Expert

When analyzing query performance:
- Always run EXPLAIN ANALYZE before recommending indexes
- Consider table statistics (pg_statistic) before suggesting index types
- Never DROP indexes without understanding impact on write workload
- Prefer partial indexes over full indexes for low-cardinality columns
```

**Multiple skills example:**

An agent with skills in both Kubernetes and PostgreSQL:

```
agents/platform-sre/
├── agent.yaml
├── SOUL.md
└── skills/
    ├── kubernetes-ops/
    │   └── SKILL.md
    └── postgres-tuning/
        └── SKILL.md
```

Both skills are appended to the system prompt in alphabetical order:
1. `## Skill: kubernetes-ops` + content
2. `## Skill: postgres-tuning` + content

---

## tools/ Directory

Each `.yaml` file in `tools/` defines a single tool available to the agent. The runtime loads all
`.yaml` files in this directory and makes them available to the LLM.

**Tool YAML schema:**

```yaml
name: kubectl
description: Run kubectl commands against a Kubernetes cluster
type: cli
command: kubectl
args: ["{{subcommand}}", "{{args}}"]
```

**Tool types:**

| `type` | Description | Required fields |
|--------|-------------|-----------------|
| `cli` | Execute a specific CLI binary with defined arguments | `command` |
| `shell` | Execute an arbitrary shell command | `command` |
| `mcp` | Expose tools from an MCP server | `server` |

The tool schema in `tools/*.yaml` is equivalent to one entry from `spec.tools` in the flat Agent
YAML format. Template variables in args use `{{var}}` syntax.

**Example — `tools/kubectl.yaml`:**

```yaml
name: kubectl
description: Run kubectl commands against a Kubernetes cluster
type: cli
command: kubectl
args: ["{{subcommand}}", "{{args}}"]
```

**Example — `tools/postgres-query.yaml`:**

```yaml
name: postgres-query
description: Run a SQL query against the configured PostgreSQL database
type: shell
command: "psql ${DATABASE_URL} -c '{{query}}'"
```

---

## agents/ Subdirectory (Multi-Agent)

When an agent directory contains an `agents/` subdirectory, each subdirectory therein is itself a
valid agent definition (same format, recursive). This enables **hierarchical multi-agent
architectures**: a coordinator agent can delegate tasks to specialized sub-agents.

**Sub-agent loading:** The runtime loads sub-agents **lazily** — only when the parent agent first
delegates to one. Sub-agents are not loaded at workspace startup.

**Example — coordinator with specialized sub-agents:**

```
agents/coordinator/
├── agent.yaml
├── SOUL.md           # "You are a coordinator. Delegate to your sub-agents."
└── agents/
    ├── researcher/
    │   ├── agent.yaml
    │   └── SOUL.md   # "You are a research specialist."
    └── writer/
        ├── agent.yaml
        └── SOUL.md   # "You are a technical writer."
```

**Nesting:** Sub-agents can themselves have `agents/` subdirectories. There is no enforced depth
limit, but deep nesting (> 3 levels) is discouraged for maintainability.

**Sub-agent identity:** A sub-agent's `name` field in `agent.yaml` must be unique within its parent
agent's `agents/` directory. It does not need to be globally unique across the workspace.

---

## hooks/ Directory

Hooks are markdown files containing instructions injected at specific points in the agent lifecycle.

| File | When invoked | Use cases |
|------|-------------|-----------|
| `hooks/bootstrap.md` | Before each run starts | Load context from `memory/runtime/`, check environment, set goals |
| `hooks/teardown.md` | After each run completes | Update `memory/runtime/dailylog.md`, summarize findings |

Hook content is appended to the assembled system prompt (bootstrap) or passed as a post-run
instruction (teardown). Hooks are optional — if the file does not exist, the lifecycle stage is
skipped.

---

## knowledge/ Directory

The `knowledge/` directory contains reference documents that are injected into the agent's context
at assembly time. These are static documents the agent should be aware of but not modify.

**Supported format:** Markdown (`.md`)

**Loading behavior:** All `.md` files are read in alphabetical filename order and appended after a
`## Reference Material` header.

**Use cases:**
- Runbooks and operational procedures
- Architecture decision records (ADRs)
- API documentation
- Domain glossaries

---

## memory/runtime/ Directory

The `memory/runtime/` directory is the agent's **live state** — files the agent reads and writes
during operation. Unlike `knowledge/` (static, read-only), `memory/runtime/` files change between
runs.

**Common files (by convention):**

| File | Purpose |
|------|---------|
| `dailylog.md` | Append-only log of actions taken today |
| `context.md` | Current task context and open threads |
| `decisions.md` | Decisions made and their rationale |

The runtime does not enforce specific filenames — any file the agent references is valid.

---

## Validation Rules

The OpenAgentiX runtime validates agent directories at load time:

| Check | Rule | Error |
|-------|------|-------|
| `agent.yaml` exists | Required | `agent "X": agent.yaml not found` |
| `agent.yaml` parseable | Must be valid YAML matching agent manifest schema | `agent "X": agent.yaml: field Y: ...` |
| `SOUL.md` exists | Required | `agent "X": SOUL.md not found` |
| `SOUL.md` non-empty | Must have non-whitespace content | `agent "X": SOUL.md is empty` |
| `agents/` sub-agents | Each subdirectory must itself be a valid agent directory | `agent "X": sub-agent "Y": SOUL.md not found` |
| `tools/*.yaml` | Each file must match the tool schema | `agent "X": tools/Z.yaml: field type: unknown value` |
| Unknown directories | Ignored (forward compatibility) | — |

---

## Forward Compatibility

Unknown files and directories in an agent directory are **silently ignored**. This ensures that
agent definitions written for a newer version of the spec continue to load in older runtimes (with
reduced functionality).

---

## Backward Compatibility with Flat YAML Format

The OpenAgentiX runtime also supports the **flat YAML format** (the `apiVersion: openagentix.dev/v1
/ kind: Agent` format). Both formats are supported when `agents_dir` is scanned:

| Discovery result | Format | Loaded as |
|-----------------|--------|-----------|
| A directory containing `agent.yaml` | GitAgent directory format | Full directory-based agent |
| A `*.yaml` file at the top level | Flat YAML with `kind: Agent` | Backward-compatible flat agent |

The flat format is documented in [agent-yaml-v1.md](agent-yaml-v1.md) under "Flat YAML Format
(Backward Compatibility)".

---

## Examples

### Example 1: Minimal Agent Directory

The smallest valid agent — just the two required files.

```
agents/hello-world/
├── agent.yaml
└── SOUL.md
```

**`agent.yaml`:**
```yaml
spec_version: "0.1.0"
name: hello-world
model:
  preferred: anthropic/claude-sonnet-4-6
```

**`SOUL.md`:**
```markdown
You are a helpful assistant. Answer questions concisely and accurately.
```

**Assembled system prompt:**
```
You are a helpful assistant. Answer questions concisely and accurately.
```

---

### Example 2: Full-Featured Agent Directory

A database optimization agent with skills, tools, hooks, and knowledge.

```
agents/db-optimizer/
├── agent.yaml
├── SOUL.md
├── RULES.md
├── skills/
│   └── postgres-tuning/
│       └── SKILL.md
├── tools/
│   ├── run-explain.yaml
│   └── list-indexes.yaml
├── knowledge/
│   └── postgres-tuning-runbook.md
├── hooks/
│   └── bootstrap.md
└── memory/
    └── runtime/
        ├── dailylog.md
        └── context.md
```

**`agent.yaml`:**
```yaml
spec_version: "0.1.0"
name: db-optimizer
version: 1.2.0
description: Analyzes slow PostgreSQL queries and recommends index improvements
model:
  preferred: anthropic/claude-sonnet-4-6
```

**`SOUL.md`:**
```markdown
You are a PostgreSQL performance expert. Your goal is to identify slow queries
and recommend index improvements. Always explain your reasoning. Never DROP
indexes without explicit approval. Focus on queries with execution time > 1s
or sequential scans on tables with more than 10K rows.
```

**`RULES.md`:**
```markdown
- Never execute destructive SQL (DROP, DELETE, TRUNCATE) without explicit human approval
- Never output raw connection strings or credentials
- Always cite the source query when making recommendations
```

**Assembled system prompt:**
```
You are a PostgreSQL performance expert...  [SOUL.md content]

## Constraints
- Never execute destructive SQL...          [RULES.md content]

## Skill: postgres-tuning
## PostgreSQL Performance Expert            [skills/postgres-tuning/SKILL.md]
...

## Reference Material
[content of knowledge/postgres-tuning-runbook.md]

## Pre-Run Instructions
[content of hooks/bootstrap.md]
```

---

### Example 3: Multi-Agent Coordinator

A coordinator agent that delegates to specialized sub-agents.

```
agents/platform-coordinator/
├── agent.yaml
├── SOUL.md
└── agents/
    ├── db-specialist/
    │   ├── agent.yaml
    │   └── SOUL.md
    └── k8s-specialist/
        ├── agent.yaml
        └── SOUL.md
```

**`agent.yaml`:**
```yaml
spec_version: "0.1.0"
name: platform-coordinator
description: Coordinates platform operations by delegating to specialists
model:
  preferred: anthropic/claude-sonnet-4-6
```

**`SOUL.md`:**
```markdown
You are a platform operations coordinator. You receive high-level operational
tasks and delegate them to the appropriate specialist agents:

- Database issues → db-specialist
- Kubernetes issues → k8s-specialist

Break down complex tasks into sub-tasks. Synthesize results from specialists
into a clear summary for the human operator.
```

---

## Related Specifications

- [agent-yaml-v1.md](agent-yaml-v1.md) — `agent.yaml` manifest field reference
- [workspace-config.md](workspace-config.md) — workspace discovery and `agents_dir` configuration
- The runtime assembles `AgentDefinition` from directory contents in `agentix-core` crate
