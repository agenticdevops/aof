# Agent Generation Pipeline

Developer documentation for the natural language to AGENTS.md + SOUL.md generation pipeline.

## Pipeline Overview

The agent creation flow transforms natural language descriptions into validated workspace files:

```
User: "I need a K8s monitoring agent"
  ↓
Intent Classification (confidence >= 0.8)
  ↓
AgentCreator Specialist
  ↓
├─ Load Context (existing agents, available skills)
├─ Generate Agent YAML (LLM call #1)
├─ Validate (7 validation checks)
├─ Auto-fix skill hallucinations if needed
├─ Generate SOUL.md (LLM call #2)
└─ Return SpecialistOutput (requires confirmation)
  ↓
User confirms
  ↓
Files written to workspace
```

## Prompt Engineering

### Agent Generation Prompt

The agent generation prompt (`build_agent_generation_prompt`) is carefully structured to prevent hallucinations:

**Key Requirements:**
1. Output ONLY valid YAML (no markdown fences, no explanation)
2. Use lowercase-hyphenated format for `id` (e.g., 'k8s-monitor')
3. Include all required fields: id, name, role, avatar, personality_traits, can, cannot, skills
4. Avatar must be a single emoji character
5. **Skills MUST ONLY use skills from the available list** (critical for preventing hallucination)

**Why Available Skills Are Included:**

The full list of available skills is embedded in the prompt:

```
8. skills must ONLY use skills from this available list:

   - k8s-diagnostics
   - log-parser
   - metrics-analysis
   ...
```

This constraint dramatically reduces hallucination. Without it, Claude will invent plausible-sounding but non-existent skill names like "k8s-monitoring" or "cluster-debugger".

### SOUL.md Generation Prompt

The personality generation prompt (`build_soul_generation_prompt`) instructs Claude to output markdown with YAML frontmatter:

**Output Format:**
1. YAML frontmatter block with: id, communication_style, tone, values, personality_summary, boundaries, default_intro
2. Followed by a `## Communication Style` markdown section with 2-3 paragraphs of prose

**Agent Context Provided:**
- Agent ID (must match AGENTS.md)
- Role (influences communication style)
- Personality traits (guide tone and values)
- Description (provides domain context)

## Validation Rules

The `validate_generated_agent` function performs 7 checks:

### 1. Agent ID Format

```rust
regex: ^[a-z0-9]+(-[a-z0-9]+)*$
```

Valid: `k8s-monitor`, `test-agent-123`, `simple`
Invalid: `Test-Agent`, `test_agent`, `test agent`, `test-`, `-test`

### 2. Duplicate ID Detection

Checks if the agent ID already exists in the loaded AGENTS.md file. Prevents overwrites.

### 3. Non-Empty Required Fields

- `name` must be non-empty
- `role` must be non-empty
- `personality_traits` must have at least 1 element

### 4. Single Emoji Avatar

Uses `unicode-segmentation` to count grapheme clusters. Validates that:
- Avatar has exactly 1 visual character
- That character is in known emoji Unicode blocks (emoticons, symbols, transport, etc.)

Invalid: `"ABC"` (text), `"🧪🤖"` (multiple emoji), `""` (empty)

### 5. Skill Existence Check

For each skill in the generated agent's `skills` list, verify it exists in the `available_skills` list from the SkillRegistry.

If a skill doesn't exist, `find_similar_skills` suggests 1-3 alternatives using substring matching and common prefix logic.

### 6. Capability Conflict Detection

Scans `can` and `cannot` lists for conflicts:
- Exact match (case-insensitive): `"deploy code"` in both → error
- Substring match: `"deploy code"` (can) vs `"deploy to production"` (cannot) → warning (potential conflict)

### 7. No Empty Personality Traits

Agent must have at least one personality trait defined.

## Hallucination Prevention

The system uses a two-layer defense against skill hallucination:

### Layer 1: Prompt Constraint (Primary)

Include the exhaustive list of available skills in the generation prompt. This prevents ~95% of hallucinations.

### Layer 2: Validation + Auto-Fix (Fallback)

If Claude generates a non-existent skill anyway:

1. `validate_generated_agent` returns `SkillNotFound` errors with suggestions
2. `validate_with_retry` checks for skill errors
3. Automatically removes hallucinated skills from the agent
4. If ALL skills were hallucinated → error to user
5. Otherwise, re-validate with corrected skills

**Example:**

```
Generated: skills: ["k8s-monitoring", "custom-debugger"]
Available: ["k8s-diagnostics", "k8s-monitor", "log-parser"]

Validation: SkillNotFound { skill: "k8s-monitoring", suggestions: ["k8s-monitor", "k8s-diagnostics"] }
Auto-fix: Remove both hallucinated skills → error (no valid skills remain)
```

## Error Recovery

### Invalid YAML

If Claude returns invalid YAML (wrapped in explanation text, malformed syntax):

1. `parse_agent_yaml` attempts to strip markdown code fences
2. Uses `serde_path_to_error` for precise error messages
3. `generate_agent` retries up to `MAX_GENERATION_RETRIES` (2) times
4. After retries exhausted → error returned to user with diagnostic

### Validation Failures

Non-skill validation errors (duplicate ID, invalid emoji, missing fields) fail immediately without retry. These are logical errors, not generation noise.

### SOUL.md Parsing Errors

`parse_soul_markdown` expects:

```markdown
```yaml
id: agent-id
...
```

## Communication Style

Prose content here.
```

If the format doesn't match, or the agent ID in SOUL.md doesn't match the expected ID, an error is returned.

## Extending

### Adding New Agent Fields

1. Add field to `Agent` type in `aof-personas`
2. Update `build_agent_generation_prompt` to instruct Claude to include it
3. Add validation rule in `validate_generated_agent` if needed
4. Update tests in `agent_creation_tests.rs`

### Modifying Validation Rules

All validation logic is in `src/validation.rs`. Rules are independent - add/remove/modify without affecting others.

### Adjusting Generation Prompts

Prompts are in `src/generation.rs`. Changes affect LLM behavior:

- Stricter prompt = lower hallucination rate, less creative output
- Looser prompt = more creative, higher hallucination risk

Balance based on your use case. Always include available skills list for hallucination prevention.

### Changing Retry Logic

`MAX_GENERATION_RETRIES` is defined in `agent_creator.rs`. Increase if you see frequent transient YAML errors. Decrease to fail faster.
