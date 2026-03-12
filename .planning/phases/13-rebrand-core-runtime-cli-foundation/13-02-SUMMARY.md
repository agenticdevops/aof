---
phase: 13
plan: 02
subsystem: docs/spec
tags: [spec, agent-yaml, workspace-config, openagentix, contracts-first]
dependency_graph:
  requires: []
  provides: [SPEC-01, SPEC-02, SPEC-04, SPEC-05]
  affects: [13-03, 13-04, 13-05, 13-06, 13-07, 13-08, 13-09]
tech_stack:
  added: []
  patterns: [contracts-first, kubernetes-style-yaml, env-var-expansion]
key_files:
  created:
    - docs/spec/agent-yaml-v1.md
    - docs/spec/workspace-config.md
  modified: []
decisions:
  - "Agent YAML uses apiVersion: openagentix.dev/v1 / kind: Agent — full Kubernetes style"
  - "Unified spec.tools list with type discriminator (cli/mcp/shell) — no separate tool sections"
  - "provider/model explicit notation enforced — exactly one slash required"
  - "system_prompt and system_prompt_file are mutually exclusive"
  - "Three-tier resolution: agent YAML > workspace defaults > built-in defaults"
  - "Provider credentials are workspace-only — cannot be set per-agent"
  - "metadata.version is semver for config versioning, separate from spec changes"
  - "Triggers and notifications fields defined now, implemented in Phases 15-16"
  - "Approval/budget/telemetry fields defined now, implemented in Phases 17-20"
metrics:
  duration: "3m 30s"
  completed: "2026-03-12"
  tasks_completed: 2
  files_created: 2
  files_modified: 0
---

# Phase 13 Plan 02: Agent YAML v1 Specification and Workspace Config Summary

**One-liner:** Canonical `openagentix.dev/v1` Agent YAML and `agentix.yaml` workspace configuration specifications with full field definitions, validation rules, and working examples.

---

## What Was Built

Two specification documents establishing the contracts-first foundation for all Phase 13+ implementation:

1. **`docs/spec/agent-yaml-v1.md`** — The canonical Agent YAML v1 specification. Covers the full Kubernetes-style structure (`apiVersion`, `kind`, `metadata`, `spec`), all 20+ spec fields, validation rules with regex patterns and error messages, and three complete working examples (minimal, full-featured, external prompt file).

2. **`docs/spec/workspace-config.md`** — The workspace configuration specification for `agentix.yaml`. Covers defaults, provider credentials (with `${ENV_VAR}` expansion), gateway settings, agent discovery, the three-tier resolution order, and two complete examples (minimal, full production).

---

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Write Agent YAML v1 specification | ab52a0b | docs/spec/agent-yaml-v1.md |
| 2 | Write workspace configuration specification | 5179fc4 | docs/spec/workspace-config.md |

---

## Key Decisions Made

**Unified tools list:** All tool types (CLI, MCP, shell) live under a single `spec.tools` list with a `type` discriminator field. This avoids the complexity of separate `cli_tools`, `mcp_tools`, `shell_tools` keys and makes the schema simpler to validate.

**provider/model format enforced:** The `spec.model` field requires exactly one `/` separator (e.g., `anthropic/claude-sonnet-4-6`). This enables unambiguous provider routing without separate `provider` and `model` fields.

**Workspace-only provider credentials:** API keys and provider endpoints live in `agentix.yaml` only — not in individual agent files. This keeps secrets out of agent definitions that may be checked into version control.

**Phase-aware field documentation:** Fields like `spec.triggers`, `spec.approval`, `spec.budget`, and `spec.telemetry` are defined in the spec now but clearly annotated with the phase where their enforcement logic ships. This prevents breaking changes when those phases implement the runtime behavior.

**`metadata.version` for config versioning:** The `metadata.version` field (semver) tracks changes to the agent's configuration definition — not the OpenAgentiX spec version (which is `apiVersion`). Useful for auditing what config was active during an incident.

---

## Deviations from Plan

None — plan executed exactly as written. Both spec documents were created with all required fields, examples, and validation rules.

---

## Requirements Fulfilled

- **SPEC-01:** Agent YAML uses `apiVersion: openagentix.dev/v1` — documented and validated
- **SPEC-02:** Full spec fields documented — model, mode, tools, mcp_servers, triggers, notifications, approval, budget, telemetry, system_prompt, system_prompt_file, env
- **SPEC-04:** Namespace field documented in metadata with default `"default"` and DNS-compatible validation
- **SPEC-05:** Version field documented in metadata with semver format validation

---

## Self-Check

- [x] `docs/spec/agent-yaml-v1.md` exists and contains `apiVersion: openagentix.dev/v1`
- [x] `docs/spec/workspace-config.md` exists and contains `agentix.yaml` and `defaults:`
- [x] Commit ab52a0b exists (Task 1)
- [x] Commit 5179fc4 exists (Task 2)

## Self-Check: PASSED
