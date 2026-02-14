---
phase: 05-agent-personas
plan: "02"
subsystem: personas
tags: [prompt-composition, token-counting, caching, injection-detection, system-prompt, sha256]

# Dependency graph
requires:
  - phase: 05-01
    provides: Agent/Soul types, AgentLoader, SoulLoader, validation, AgentCache
provides:
  - PromptComposer with 7-layer instruction composition
  - Token counting and graceful truncation
  - SHA256-based prompt caching
  - Tool-to-skill linking from TOOLS.md
  - Prompt injection detection (6 regex patterns)
  - AgentExecutor persona prompt integration
affects: [05-03, 05-04, 05-05, aof-runtime]

# Tech tracking
tech-stack:
  added: []
  patterns: [7-layer instruction composition, SHA256 cache invalidation, graceful truncation by priority]

key-files:
  created:
    - crates/aof-personas/src/composer.rs
    - crates/aof-personas/tests/composer_tests.rs
    - crates/aof-personas/tests/integration_composer_test.rs
    - docs/dev/prompt-composition.md
    - docs/examples/composed-prompts.md
  modified:
    - crates/aof-personas/src/lib.rs
    - crates/aof-runtime/src/executor/agent_executor.rs

key-decisions:
  - "7-layer instruction composition: base -> role -> personality -> communication -> capabilities -> tools -> behavioral rules"
  - "Token estimation at len/4 (Claude approximation) with 8000 token default limit"
  - "Truncation priority: behavioral rules first, then tools, then communication, never personality/boundaries"
  - "Persona prompt as optional override in AgentExecutor (config.system_prompt takes precedence)"
  - "SHA256 hash of agent+soul+tool data for cache invalidation"
  - "6 regex injection patterns: ignore previous, forget instructions, disregard prompt, override system, new identity, ignore above"

patterns-established:
  - "Prompt composition: structured section headers [SECTION] for debuggability"
  - "Graceful degradation: agents without SOUL.md get default personality from traits"
  - "Tool linking: skills map to TOOLS.md by exact name match, unknown tools marked as not found"

# Metrics
duration: 813s
completed: 2026-02-14
---

# Phase 5 Plan 02: System Prompt Composition Engine Summary

**7-layer dynamic prompt composition from AGENTS.md + SOUL.md + TOOLS.md with token-limited truncation, SHA256 caching, and injection detection**

## Performance

- **Duration:** 813s (13.5 minutes)
- **Started:** 2026-02-14T04:17:12Z
- **Completed:** 2026-02-14T04:30:45Z
- **Tasks:** 9/9
- **Files modified:** 7

## Accomplishments

- PromptComposer with 7-layer instruction composition producing distinct prompts per agent personality
- Token counting (len/4) and graceful truncation preserving personality while dropping low-priority sections
- SHA256-based prompt caching with hit/miss tracking and async cache stats
- Tool-to-skill linking from TOOLS.md with deduplication and missing-tool warnings
- Prompt injection detection (6 regex patterns) in validate_and_compose()
- AgentExecutor integration via with_persona_prompt() builder (backward compatible)
- 45 new tests (19 inline + 18 external + 8 integration) all passing
- Developer documentation and 3 composed prompt examples

## Task Commits

Each task was committed atomically:

1. **Task 1: PromptComposer struct and 7-layer composition** - `6fdf506d` (feat)
2. **Task 2: Token counting and graceful truncation** - `2a1998ff` (feat)
3. **Task 3: Prompt caching with SHA256 invalidation** - `80bd71ee` (feat)
4. **Task 4: Tool reference linking (skills to TOOLS.md)** - `e96c5252` (feat)
5. **Task 5: Security validation and injection detection** - `5602f343` (feat)
6. **Task 6: 18 comprehensive unit tests** - `8d587482` (test)
7. **Task 7: AgentExecutor persona prompt integration** - `796eabe4` (feat)
8. **Task 8: End-to-end integration test** - `dde85c8d` (test)
9. **Task 9: Developer documentation and examples** - `3a825834` (docs)

## Files Created/Modified

- `crates/aof-personas/src/composer.rs` - PromptComposer with 7-layer composition, token counting, caching, injection detection
- `crates/aof-personas/src/lib.rs` - Added composer module and re-exports
- `crates/aof-personas/tests/composer_tests.rs` - 18 comprehensive tests with reference agents
- `crates/aof-personas/tests/integration_composer_test.rs` - 8 end-to-end workflow tests
- `crates/aof-runtime/src/executor/agent_executor.rs` - persona_prompt field and with_persona_prompt() builder
- `docs/dev/prompt-composition.md` - Architecture, caching, tool linking, security, troubleshooting
- `docs/examples/composed-prompts.md` - Real composed prompts for k8s-monitor, log-analyzer, incident-responder

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| **7-layer instruction composition** | Clear separation of concerns, each layer has distinct purpose, section headers aid debugging |
| **Token estimation at len/4** | Claude standard approximation, conservative, sufficient for budget management without tokenizer dependency |
| **8000 token default limit** | Leaves room for conversation context, most agents compose to 500-2000 tokens naturally |
| **Truncation by priority** | Behavioral rules are generic (drop first), personality is essential (never drop), tools can be summarized |
| **SHA256 for cache invalidation** | Deterministic, efficient, same pattern used elsewhere in AOF (version_hash in config.rs) |
| **Persona prompt as optional override** | Backward compatible, config.system_prompt takes precedence for expert mode |
| **6 injection regex patterns** | Extended from 4 in plan to cover "you are now a different" and "ignore the above" variants |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed injection test targeting wrong field**
- **Found during:** Task 5 (injection detection tests)
- **Issue:** Test set injection text in `default_intro` which doesn't appear in composed prompts
- **Fix:** Changed test to inject into `personality_summary` and `communication_guide` which flow into composed output
- **Files modified:** crates/aof-personas/src/composer.rs
- **Verification:** Injection detection test now correctly catches injected text in composed prompt
- **Committed in:** 5602f343

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Minor test correction to target correct injection surface. No scope creep.

## Issues Encountered

- events.rs from 05-01 references aof-core types that exist but initially caused a transient compilation issue during concurrent tool indexing. Resolved by ensuring clean build state.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- PromptComposer ready for downstream consumers (05-03 introduction events, 05-04 UI, 05-05 reliability)
- AgentExecutor integration is opt-in via with_persona_prompt() (no breaking changes)
- All 98 tests passing across aof-personas crate (including pre-existing 05-01 tests)
- Documentation complete for developer onboarding

## Self-Check: PASSED

All files verified present:
- `crates/aof-personas/src/composer.rs` - FOUND
- `crates/aof-personas/tests/composer_tests.rs` - FOUND
- `crates/aof-personas/tests/integration_composer_test.rs` - FOUND
- `crates/aof-runtime/src/executor/agent_executor.rs` - FOUND (modified)
- `docs/dev/prompt-composition.md` - FOUND
- `docs/examples/composed-prompts.md` - FOUND

All commits verified:
- `6fdf506d` - FOUND
- `2a1998ff` - FOUND
- `80bd71ee` - FOUND
- `e96c5252` - FOUND
- `5602f343` - FOUND
- `8d587482` - FOUND
- `796eabe4` - FOUND
- `dde85c8d` - FOUND
- `3a825834` - FOUND

---
*Phase: 05-agent-personas*
*Completed: 2026-02-14*
