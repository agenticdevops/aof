# Phase 13: Rebrand + Core Runtime + CLI Foundation - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Transform AOF into OpenAgentiX — full rebrand (crates, binary, repo, docs), new agent YAML spec (`openagentix.dev/v1`), ReAct loop runtime, streaming output, and gateway-first architecture. Clean break from v1.0 — no backward compatibility, no migration path. OpenAgentiX is day one for users.

</domain>

<decisions>
## Implementation Decisions

### Rebrand Scope
- **Clean break** — delete all v1.0 code (React web-app, agent personas, SOUL.md, v1.0 YAML parsing, quickstart agents, v1.0 docs). Build fresh.
- Rename all crates: `aof-core` → `agentix-core`, `aof-llm` → `agentix-llm`, `aof-mcp` → `agentix-mcp`, `aof-runtime` → `agentix-runtime`, `aof-memory` → `agentix-memory`, `aof-triggers` → `agentix-triggers`, `aofctl` → `agentix` (binary)
- GitHub repo moves to `openagentix/agentix` (new org + new repo name)
- Domain: `openagentix.org` — docs at `docs.openagentix.org`
- SPEC-03 (backward compatibility) becomes a deletion task — remove all v1.0 field names, formats, and compat code

### Agent YAML Spec
- **Full Kubernetes style**: `apiVersion: openagentix.dev/v1`, `kind: Agent`, `metadata` (name, namespace, labels, annotations), `spec`
- System prompt: `spec.system_prompt` for inline, `spec.system_prompt_file` for external .md file reference
- **Unified `spec.tools` list** — all tools (CLI, MCP, shell) under one key with a `type` field to distinguish (`cli`, `mcp`, `shell`)
- Model format: `provider/model` explicit notation (e.g., `anthropic/claude-sonnet-4-6`, `openai/gpt-4o`)
- Workspace-level defaults in `agentix.yaml` — agents inherit `model`, `max_iterations`, etc. unless they override

### Gateway-First Architecture
- **The core product is the gateway service**, not individual agent execution
- Follow the OpenClaw established pattern:
  - `agentix onboard` — interactive setup (API keys, workspace config, first agent)
  - `agentix gateway start` — boots the system (runtime + API + channels)
  - Agents are YAML configs applied to the running gateway
- No one-shot `agentix run <file>` mode — agents live inside the gateway
- CLI commands talk to the running gateway (`agentix agents`, `agentix logs`, etc.)
- This fundamentally changes CLI-01/CLI-02 — replace with gateway-centric commands

### CLI Output & Streaming
- **Streaming narrative** for ReAct loop display: labeled sections `[Plan]`, `[Act]`, `[Observe]`, `[Reflect]` streaming in real-time
- `--quiet` flag: final result only (no loop details)
- `--json` flag: structured JSON output for every event (machine-parseable, CI pipelines)
- **Rustc-style error display**: pointer to exact line/field in YAML, expected values, help links. Uses `serde_path_to_error`.
- Colors with auto-detect: on when TTY, off when piped. Respect `NO_COLOR` env var.

### ReAct Loop
- **ReAct only** — every agent goes through the plan → act → observe → reflect loop. No separate "direct" mode.
- Tool failures feed back into the loop as the "observe" step — the agent sees the error and decides what to do (retry, alternate approach, or give up). Agent is in control.

### Model Provider Strategy
- Support as many providers as possible at launch: OpenAI, Anthropic, Google (Gemini), Local/Ollama, and more
- **BYOK + consumer subscriptions** — users can authenticate with their ChatGPT, Claude, and Gemini subscriptions, not just API keys. Use an LLM proxy or external SDK if needed to enable this.
- Provider/model explicit format in YAML: `model: anthropic/claude-sonnet-4-6`
- Workspace defaults with agent-level override

### Project Identity
- **Open-source enterprise agentic automation platform**
- Target: DevOps/platform engineers AND developers building agent apps
- Enterprise focus: approval workflows, cost tracking, audit trails, observability
- Ship with pre-built starter **Packs** in v2.0 (DBA optimizer, cost analyzer, security scanner, etc.)
- Pre-built agent bundles are called **Packs** (not "Hands" like OpenFang)

### Claude's Discretion
- CLI command surface beyond gateway core (which subcommands exist, exact flags)
- `agentix validate` as standalone vs built into other commands
- `agentix init` scaffold contents — keep only what's genuinely needed, no empty placeholder directories
- ReAct loop defaults (max iterations, timeout)
- Error message wording and help text
- Provider fallback chain behavior
- Exact workspace config file structure

</decisions>

<specifics>
## Specific Ideas

- Follow OpenClaw's onboarding/gateway pattern — `agentix onboard` + `agentix gateway start`. Users from that ecosystem should feel at home.
- Reference architectures: OpenClaw (TypeScript gateway, OG pattern), OpenFang (Rust agent OS, 14 crates), ZeroClaw (ultra-lightweight Rust)
- Rustc-style errors are a UX differentiator — make YAML validation feel polished, not like generic "parse error at line 8"
- Consumer subscription auth (ChatGPT/Claude/Gemini accounts, not just API keys) is a differentiator against every other framework that requires API billing

</specifics>

<deferred>
## Deferred Ideas

- Which specific starter Packs to ship — Phase 14+ decision
- OAuth login with Codex/Claude for Command Center — Phase 22 scope
- Which specific starter agent packs to ship — Phase 14+ decision
- `agentix docs` subcommand for inline documentation — evaluate during CLI design

</deferred>

---

*Phase: 13-rebrand-core-runtime-cli-foundation*
*Context gathered: 2026-03-12*
