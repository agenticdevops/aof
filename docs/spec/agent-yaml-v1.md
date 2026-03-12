# Agent Manifest Specification (agent.yaml)

**Spec version:** 0.1.0
**File:** `agent.yaml` (inside an agent directory)

---

## Overview

`agent.yaml` is the **minimal manifest** for a GitAgent-compatible agent definition. It carries
only metadata and model preference. All behavior lives in markdown files:

| What | Where |
|------|-------|
| Identity and goals | `SOUL.md` |
| Hard constraints | `RULES.md` |
| Capability modules | `skills/<name>/SKILL.md` |
| Tool definitions | `tools/<name>.yaml` |
| Runtime behavior | workspace `agentix.yaml` defaults |

If you are looking for the full directory layout — directory structure, skills/, tools/, agents/,
hooks/ — see [agent-directory-structure.md](agent-directory-structure.md).

---

## Manifest Structure

```yaml
spec_version: "0.1.0"           # Required: OpenAgentiX agent spec version
name: my-agent                   # Required: unique name (DNS-compatible, lowercase, hyphens)
version: 0.1.0                   # Optional: semver agent version
description: A helpful assistant # Optional: human-readable description
model:
  preferred: anthropic/claude-sonnet-4-6  # Optional: "provider/model" format
extends: https://github.com/org/base-agent.git  # Optional: inherit from another agent dir
dependencies:                    # Optional: external sub-agent dependencies
  - name: fact-checker
    source: https://github.com/org/fact-checker.git
    version: ^1.0.0
    mount: agents/fact-checker
```

---

## Field Reference

### `spec_version` (required)

- **Type:** string
- **Description:** The OpenAgentiX agent directory spec version this `agent.yaml` conforms to.
- **Current value:** `"0.1.0"`
- **Usage:** The runtime uses this to determine how to parse the directory. Unknown versions are
  rejected with an error message indicating which versions are supported.

```yaml
spec_version: "0.1.0"
```

---

### `name` (required)

- **Type:** string
- **Validation:** Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, maximum 63 characters
- **Description:** The agent's unique identifier within its workspace. Used as the agent's name in
  the gateway REST API, CLI output, and inter-agent delegation.
- **Examples:** `cost-analyzer`, `db-optimizer`, `security-scanner`

```yaml
name: db-optimizer
```

---

### `version` (optional)

- **Type:** string (semver)
- **Format:** `MAJOR.MINOR.PATCH` (e.g., `"1.0.0"`, `"2.1.3"`)
- **Description:** Tracks the agent definition version. Useful for change management and for
  `dependencies` version range matching.

```yaml
version: 1.2.0
```

---

### `description` (optional)

- **Type:** string
- **Description:** Human-readable description shown in `agentix agents` output and the gateway
  dashboard.

```yaml
description: Analyzes slow PostgreSQL queries and recommends index improvements
```

---

### `model.preferred` (optional)

- **Type:** string
- **Format:** `provider/model` — exactly one `/` separator required
- **Description:** The LLM model this agent prefers to use. If omitted, falls back to the workspace
  `spec.defaults.model` in `agentix.yaml`. If both are absent, the runtime reports an error at
  load time.
- **Supported providers:**

  | Provider | Example values |
  |----------|---------------|
  | `anthropic` | `anthropic/claude-sonnet-4-6`, `anthropic/claude-3-5-haiku` |
  | `openai` | `openai/gpt-4o`, `openai/gpt-4o-mini` |
  | `google` | `google/gemini-2.0-flash`, `google/gemini-1.5-pro` |
  | `ollama` | `ollama/llama3`, `ollama/mistral` |

```yaml
model:
  preferred: anthropic/claude-sonnet-4-6
```

---

### `extends` (optional)

- **Type:** string (git URL)
- **Description:** Inherit a base agent definition from a remote git repository. The runtime clones
  the base agent directory, then overlays the current agent's files on top of it. Files present in
  the current agent directory take precedence over files in the base.
- **Use case:** Share a common `RULES.md` or `skills/` across multiple agents without duplication.

```yaml
extends: https://github.com/openagentix/base-agents.git
```

**Overlay behavior:**

| Base has | Current has | Result |
|----------|-------------|--------|
| `SOUL.md` | `SOUL.md` | Current agent's `SOUL.md` is used |
| `RULES.md` | nothing | Base `RULES.md` is inherited |
| `skills/k8s/` | nothing | Base skill is inherited |
| `skills/custom/` | `skills/custom/` | Current agent's skill takes precedence |

---

### `dependencies` (optional)

- **Type:** list of sub-agent dependency specs
- **Description:** Declare external sub-agents that should be fetched and mounted into this agent's
  `agents/` directory. Enables importing published sub-agents without bundling them in the same
  repository.

Each dependency entry:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Local alias for this sub-agent within `agents/` |
| `source` | yes | Git URL or local path to the sub-agent directory |
| `version` | no | Semver range (e.g., `^1.0.0`, `>=2.0.0`) |
| `mount` | no | Where to mount inside `agents/` (defaults to `agents/<name>`) |

```yaml
dependencies:
  - name: fact-checker
    source: https://github.com/openagentix/fact-checker.git
    version: ^1.0.0
    mount: agents/fact-checker
  - name: summarizer
    source: ./local-agents/summarizer
```

---

## What Does NOT Belong in agent.yaml

The following belong to markdown files or workspace defaults — not `agent.yaml`:

| Field | Where it lives |
|-------|---------------|
| System prompt / identity | `SOUL.md` |
| Hard constraints | `RULES.md` |
| Tool definitions | `tools/<name>.yaml` |
| Skill instructions | `skills/<name>/SKILL.md` |
| Max iterations | workspace `spec.defaults.max_iterations` |
| Timeout | workspace `spec.defaults.timeout` |
| Execution mode (autonomous/semi-autonomous/manual) | workspace `spec.defaults.mode` |
| Triggers (cron, webhook, etc.) | Phase 15 feature |
| Budget limits | Phase 17 feature |
| Notifications | Phase 16 feature |
| Approval gates | Phase 20 feature |
| Provider credentials | workspace `spec.providers` in `agentix.yaml` |

This separation is intentional: `agent.yaml` is portable and can be published to a public
repository without leaking operational configuration.

---

## Validation Rules

The OpenAgentiX runtime validates `agent.yaml` at load time and reports errors with precise field
paths (rustc-style output using `serde_path_to_error`).

| Field | Rule | Error |
|-------|------|-------|
| `spec_version` | Must be a recognized version string | `spec_version: unsupported version "X" — expected "0.1.0"` |
| `name` | Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, max 63 chars | `name: invalid format — must be lowercase alphanumeric with hyphens` |
| `version` | Must be valid semver if present (`MAJOR.MINOR.PATCH`) | `version: must be semver format (e.g., "1.0.0")` |
| `model.preferred` | Must contain exactly one `/` if present | `model.preferred: must be "provider/model" format` |
| `dependencies[*].name` | Must be valid name format | `dependencies[N].name: invalid format` |
| `dependencies[*].source` | Must be a valid git URL or local path | `dependencies[N].source: invalid URL or path` |

---

## Examples

### Example 1: Minimal agent.yaml

```yaml
spec_version: "0.1.0"
name: hello-world
model:
  preferred: anthropic/claude-sonnet-4-6
```

This is the smallest valid `agent.yaml`. Behavior is entirely defined by `SOUL.md` in the same
directory. All runtime settings (max iterations, timeout, mode) are inherited from the workspace
defaults.

---

### Example 2: Full agent.yaml

```yaml
spec_version: "0.1.0"
name: db-optimizer
version: 1.2.0
description: Analyzes slow PostgreSQL queries and recommends index improvements
model:
  preferred: anthropic/claude-sonnet-4-6
```

With this `agent.yaml`, the corresponding directory structure provides all behavior:

```
agents/db-optimizer/
├── agent.yaml         ← this file
├── SOUL.md            ← identity and goals
├── RULES.md           ← never drop indexes without approval
├── skills/
│   └── postgres-tuning/
│       └── SKILL.md   ← EXPLAIN ANALYZE technique, index strategy
└── tools/
    ├── run-explain.yaml
    └── list-indexes.yaml
```

---

### Example 3: Agent with extends

```yaml
spec_version: "0.1.0"
name: my-sre-agent
description: SRE agent with shared security rules from org base
model:
  preferred: anthropic/claude-sonnet-4-6
extends: https://github.com/my-org/base-sre-agent.git
```

The runtime fetches the base agent directory and merges it with this agent's files. The current
agent's `SOUL.md` overrides the base, but inherited `RULES.md` and `skills/` from the base are
used unless the current agent provides its own.

---

## Flat YAML Format (Backward Compatibility)

The OpenAgentiX runtime also supports the **flat YAML format** introduced before the GitAgent
directory structure. This format uses Kubernetes-style `apiVersion`/`kind` headers and packs
everything — system prompt, tools, triggers — into a single file.

**Flat format header:**
```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: my-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: |
    You are a helpful assistant.
```

**`AgentManifestFormat` enum** (internal runtime type):

| Variant | Detection | Description |
|---------|-----------|-------------|
| `Directory` | Path is a directory containing `agent.yaml` | GitAgent-compatible (this spec) |
| `FlatYaml` | Path is a `*.yaml` file with `kind: Agent` | Legacy monolithic format |

Both formats are supported when `agents_dir` is scanned. New agents should use the directory
format. The flat format is preserved for backward compatibility and will not be removed.

See [workspace-config.md](workspace-config.md) for how `agents_dir` discovers both formats.

---

## Related Specifications

- [agent-directory-structure.md](agent-directory-structure.md) — full directory layout, system
  prompt assembly, skills, tools, multi-agent hierarchy
- [workspace-config.md](workspace-config.md) — `agents_dir` scanning, workspace defaults, provider
  credentials
- `agent.yaml` is parsed by `agentix-core` crate (`AgentManifest` type)
- Validation uses `serde_path_to_error` for precise field-level error reporting
