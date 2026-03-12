# Workspace Configuration Specification

**File:** `agentix.yaml` (workspace root)
**apiVersion:** `openagentix.dev/v1`
**kind:** `Workspace`
**Spec version:** v1 (2026-03-12)

---

## Overview

The workspace configuration file (`agentix.yaml`) is the entry point for an OpenAgentiX workspace. It defines:

- **Defaults** — model, timeout, and other settings inherited by all agents unless they override
- **Provider credentials** — API keys and endpoints for LLM providers
- **Gateway settings** — host, port, and other gateway configuration
- **Agent discovery** — where to find agent YAML files

The file lives in the workspace root directory — the same directory where you run `agentix` commands.

---

## Top-Level Structure

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: <string>           # required
  labels: <map>            # optional
  annotations: <map>       # optional
spec:
  defaults:
    model: <string>        # default model for all agents
    max_iterations: <int>  # default: 10
    timeout: <string>      # default: "5m"
    mode: <enum>           # default: "autonomous"
  providers:
    anthropic:
      api_key: <string>
    openai:
      api_key: <string>
    google:
      api_key: <string>
    ollama:
      base_url: <string>
  gateway:
    host: <string>         # default: "127.0.0.1"
    port: <int>            # default: 7777
  agents_dir: <string>     # default: "./agents"
```

---

## metadata Fields

### `metadata.name` (required)

The workspace name — typically matches the project or team name.

- **Type:** string
- **Validation:** Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, maximum 63 characters
- **Examples:** `"my-project"`, `"platform-team"`, `"acme-ops"`

### `metadata.labels` (optional)

Arbitrary key-value metadata for workspace identification and tooling.

```yaml
labels:
  environment: production
  team: platform
```

### `metadata.annotations` (optional)

Non-identifying metadata — notes, links, tooling hints.

```yaml
annotations:
  description: "Platform engineering automation workspace"
  owner: "platform-team@example.com"
```

---

## spec.defaults

Default values inherited by all agents in this workspace. In the GitAgent directory format,
`agent.yaml` carries only metadata and model preference — runtime behavior fields (`max_iterations`,
`timeout`, `mode`) are configured here in the workspace defaults, not per-agent.

An agent that omits a field uses the workspace default. The built-in defaults apply when neither
the agent nor the workspace specifies a value.

### `spec.defaults.model` (optional)

Default LLM model for all agents. Same format as `spec.model` in Agent YAML — `provider/model`.

- **Type:** string
- **Format:** `<provider>/<model-name>`
- **Example:** `"anthropic/claude-sonnet-4-6"`
- **Resolution:** If an agent does not specify `spec.model`, this value is used. If both are absent, validation fails with an error at load time.

### `spec.defaults.max_iterations` (optional)

Default maximum ReAct loop iterations for all agents.

- **Type:** integer
- **Default:** `10`
- **Validation:** 1–100 (inclusive)

### `spec.defaults.timeout` (optional)

Default wall-clock timeout for all agent runs.

- **Type:** string (duration)
- **Default:** `"5m"`
- **Format:** `\d+(s|m|h)` — e.g., `"30s"`, `"5m"`, `"1h"`

### `spec.defaults.mode` (optional)

Default execution mode for all agents.

- **Type:** enum
- **Default:** `"autonomous"`
- **Values:** `autonomous`, `semi-autonomous`, `manual`

---

## spec.providers

Provider credentials and endpoint configuration. Provider config is workspace-only — it cannot be set per-agent. This is intentional: credentials belong to the workspace operator, not individual agent definitions.

**Security:** Provider fields support `${ENV_VAR}` syntax. Always use environment variable references for API keys — never hardcode secrets in `agentix.yaml`.

### `spec.providers.anthropic`

```yaml
providers:
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
```

| Field | Required | Description |
|-------|----------|-------------|
| `api_key` | yes (if using Anthropic models) | Anthropic API key |

### `spec.providers.openai`

```yaml
providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"  # optional, for compatible APIs
```

| Field | Required | Description |
|-------|----------|-------------|
| `api_key` | yes (if using OpenAI models) | OpenAI API key |
| `base_url` | no | Override API endpoint (useful for Azure OpenAI or compatible APIs) |

### `spec.providers.google`

```yaml
providers:
  google:
    api_key: "${GOOGLE_API_KEY}"
```

| Field | Required | Description |
|-------|----------|-------------|
| `api_key` | yes (if using Google models) | Google AI API key (from Google AI Studio) |

### `spec.providers.ollama`

```yaml
providers:
  ollama:
    base_url: "http://localhost:11434"
```

| Field | Required | Description |
|-------|----------|-------------|
| `base_url` | no | Ollama server URL. Default: `http://localhost:11434` |

Ollama does not require an API key. The server must be running before agents that use Ollama models are invoked.

---

## spec.gateway

Configuration for the OpenAgentiX gateway service.

```yaml
gateway:
  host: "127.0.0.1"
  port: 7777
```

| Field | Default | Description |
|-------|---------|-------------|
| `host` | `"127.0.0.1"` | Interface to bind. Use `"0.0.0.0"` to listen on all interfaces. |
| `port` | `7777` | TCP port for the gateway API server |

---

## spec.agents_dir

Root directory for agent discovery. Relative paths are resolved from the workspace root (the
directory containing `agentix.yaml`).

- **Type:** string (path)
- **Default:** `"./agents"`

```yaml
agents_dir: "./agents"
```

### Agent Discovery

The gateway scans `agents_dir` at startup and discovers agents using the following rules:

| What is found | Format | Loaded as |
|---------------|--------|-----------|
| A subdirectory containing `agent.yaml` | GitAgent directory format (v0.1.0) | Full directory-based agent |
| A `*.yaml` or `*.yml` file with `kind: Agent` | Flat YAML format (backward compat) | Legacy monolithic agent |
| Any other file or directory | — | Ignored |

**Example workspace layout:**

```
./agents/
├── db-optimizer/         ← directory format (preferred)
│   ├── agent.yaml
│   └── SOUL.md
├── k8s-scanner/          ← directory format
│   ├── agent.yaml
│   └── SOUL.md
└── legacy-agent.yaml     ← flat YAML format (still supported)
```

The runtime detects the format automatically:
- If a path under `agents_dir` is a **directory** and contains `agent.yaml` → load as GitAgent
  directory format (see [agent-directory-structure.md](agent-directory-structure.md))
- If a path under `agents_dir` is a **`*.yaml` file** at the top level → attempt to load as flat
  `kind: Agent` YAML (backward compatibility)

Subdirectories that do not contain `agent.yaml` are silently skipped (forward compatibility).

---

## Environment Variable Expansion

String values in `agentix.yaml` support `${VAR_NAME}` syntax for pulling values from the host environment.

**Rules:**

- Expansion applies to all string fields in `spec.providers` and `spec.gateway`
- Expansion also applies to `spec.defaults.model` if the model name is dynamic (rare, but supported)
- Syntax: `${VAR_NAME}` — curly braces required, no spaces
- Expansion happens at load time, before validation
- If a referenced environment variable is not set, the runtime reports a validation error:
  ```
  spec.providers.anthropic.api_key: environment variable "ANTHROPIC_API_KEY" is not set
  ```
- Literal `${` can be escaped as `$${` if needed (unusual)

**Example:**

```yaml
spec:
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"    # expanded at load time
    openai:
      api_key: "${OPENAI_API_KEY}"       # expanded at load time
  gateway:
    port: 7777                            # numeric, no expansion needed
```

---

## Configuration Resolution Order

When an agent value is being resolved, the following precedence applies (highest to lowest):

| Priority | Source | Description |
|----------|--------|-------------|
| 1 (highest) | Agent YAML field | Explicit value in the agent's `spec` |
| 2 | Workspace defaults | `spec.defaults.*` in `agentix.yaml` |
| 3 (lowest) | Built-in defaults | Hardcoded defaults in the OpenAgentiX runtime |

**Built-in defaults:**

| Field | Built-in Default |
|-------|-----------------|
| `mode` | `autonomous` |
| `max_iterations` | `10` |
| `timeout` | `"5m"` |
| `namespace` | `"default"` |

**Example resolution:**

Given this workspace config:

```yaml
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 15
    timeout: "10m"
```

And this agent:

```yaml
spec:
  max_iterations: 5
  timeout: "2m"
```

The resolved agent configuration is:

| Field | Value | Source |
|-------|-------|--------|
| `model` | `anthropic/claude-sonnet-4-6` | Workspace default |
| `max_iterations` | `5` | Agent override |
| `timeout` | `"2m"` | Agent override |
| `mode` | `autonomous` | Built-in default |

---

## Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| `metadata.name` | Must match `^[a-z][a-z0-9-]*[a-z0-9]$`, max 63 chars | `metadata.name: invalid format` |
| `spec.defaults.model` | Must be `provider/model` format if set | `spec.defaults.model: must be "provider/model" format` |
| `spec.defaults.max_iterations` | Integer 1–100 if set | `spec.defaults.max_iterations: must be between 1 and 100` |
| `spec.defaults.timeout` | Must match `\d+(s\|m\|h)` if set | `spec.defaults.timeout: invalid duration format` |
| `spec.defaults.mode` | Must be `autonomous`, `semi-autonomous`, or `manual` if set | `spec.defaults.mode: unknown value` |
| `spec.providers.*.api_key` | If `${VAR}` syntax used, env var must be set at runtime | `spec.providers.anthropic.api_key: env var "X" not set` |
| `spec.gateway.port` | Integer 1–65535 | `spec.gateway.port: must be a valid port number` |
| Model resolution | At least one of: agent `spec.model` or `spec.defaults.model` must resolve | `agent "X": no model configured — set spec.model or spec.defaults.model` |

---

## Examples

### Example 1: Minimal Workspace

A workspace with a single provider and model default. Enough to run agents that don't override any settings.

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: my-project
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
```

This workspace requires only `ANTHROPIC_API_KEY` to be set in the environment. Any agent in the `./agents` directory that uses an Anthropic model (or omits `spec.model`) will work.

### Example 2: Full Workspace Configuration

A production workspace with all providers configured, custom gateway settings, and sensible defaults.

```yaml
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: platform-ops
  labels:
    environment: production
    team: platform
  annotations:
    description: "Platform engineering automation — infrastructure, DB, and security agents"
    owner: "platform@example.com"

spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 15
    timeout: "10m"
    mode: autonomous

  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
    openai:
      api_key: "${OPENAI_API_KEY}"
    google:
      api_key: "${GOOGLE_API_KEY}"
    ollama:
      base_url: "http://localhost:11434"

  gateway:
    host: "0.0.0.0"
    port: 7777

  agents_dir: "./agents"
```

**Required environment variables for this workspace:**
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `GOOGLE_API_KEY`

Ollama does not require an API key.

---

## File Location and Discovery

The `agentix` CLI looks for `agentix.yaml` in the following order:

1. Path specified via `--config <path>` flag
2. `./agentix.yaml` in the current working directory
3. `~/.config/agentix/agentix.yaml` (user-level global config)

If none are found, the CLI prints an error with instructions to run `agentix onboard` or create the file manually.

---

## Related Specifications

- [Agent Directory Structure](agent-directory-structure.md) — GitAgent-compatible directory layout
  (primary agent format)
- [Agent Manifest (agent.yaml)](agent-yaml-v1.md) — minimal `agent.yaml` manifest field reference
- Workspace config is parsed by `agentix-core` crate (`WorkspaceConfig` type)
- Validation uses `serde_path_to_error` for precise field-level error reporting
- Provider credentials are resolved at gateway startup, not at agent load time
