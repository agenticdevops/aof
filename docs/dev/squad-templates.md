# Squad Templates - Developer Guide

Internal documentation for the squad template system in aof-conversational.

## Overview

Squad templates are pre-built agent team configurations that provide rapid deployment of coordinated agent groups for common operational scenarios. Templates are embedded in Rust code (not YAML files) for reliability.

## Template Structure

### SquadTemplate

```rust
pub struct SquadTemplate {
    pub name: String,                      // Template identifier (e.g., "incident-response")
    pub description: String,               // User-facing description
    pub agents: Vec<TemplateAgent>,        // Agent definitions (3-4 agents per squad)
    pub squad_config: SquadConfig,         // Coordination config
    pub customization_hints: Vec<String>,  // What can be customized
}
```

### TemplateAgent

```rust
pub struct TemplateAgent {
    pub id: String,                        // Agent ID (lowercase-hyphenated)
    pub name: String,                      // Display name
    pub role: String,                      // Role description
    pub avatar: String,                    // Emoji avatar (single character)
    pub personality_traits: Vec<String>,   // 3-5 personality adjectives
    pub skills: Vec<String>,               // Skill references (2-4)
    pub can: Vec<String>,                  // Capability list (3-4)
    pub cannot: Vec<String>,               // Boundary list (2-3)
}
```

### SquadConfig

```rust
pub struct SquadConfig {
    pub coordination: String,    // e.g., "hierarchical - triage leads"
    pub communication: String,   // e.g., "broadcast to all on critical findings"
}
```

## Available Templates

| Template | Agents | Use Case |
|----------|--------|----------|
| incident-response | 4 | Rapid incident triage, investigation, remediation |
| monitoring | 3 | Proactive health monitoring and alerting |
| deployment | 3 | Safe deployment automation with validation |
| cost-optimization | 3 | Cloud cost analysis and savings implementation |

## Adding New Templates

**Step 1: Create template module** (`src/templates/your_template.rs`)

```rust
use super::{SquadConfig, SquadTemplate, TemplateAgent};

pub fn template() -> SquadTemplate {
    SquadTemplate {
        name: "your-template".to_string(),
        description: "Brief description".to_string(),
        agents: vec![
            TemplateAgent {
                id: "agent-1".to_string(),
                name: "Agent Name".to_string(),
                role: "Role Description".to_string(),
                avatar: "🤖".to_string(),
                personality_traits: vec![
                    "trait1".to_string(),
                    "trait2".to_string(),
                    "trait3".to_string(),
                ],
                skills: vec!["skill-1".to_string(), "skill-2".to_string()],
                can: vec![
                    "Capability 1".to_string(),
                    "Capability 2".to_string(),
                    "Capability 3".to_string(),
                ],
                cannot: vec![
                    "Boundary 1".to_string(),
                    "Boundary 2".to_string(),
                ],
            },
            // 2-3 more agents...
        ],
        squad_config: SquadConfig {
            coordination: "coordination-pattern".to_string(),
            communication: "communication-pattern".to_string(),
        },
        customization_hints: vec![
            "Customization hint 1".to_string(),
            "Customization hint 2".to_string(),
        ],
    }
}
```

**Step 2: Register in `mod.rs`**

```rust
pub mod your_template;

impl SquadTemplateLibrary {
    pub fn load_builtin() -> Self {
        let mut templates = HashMap::new();
        templates.insert("your-template".to_string(), your_template::template());
        // ... other templates
        Self { templates }
    }
}
```

**Step 3: Test**

```bash
cargo test -p aof-conversational --lib templates
```

## Domain Customization

### Current Implementation (MVP)

Simple text-based customization without Claude:
- Appends domain to role descriptions
- Updates personality summaries
- Preserves original skills and capabilities

### Future Enhancement

Claude-based customization (deferred to Phase 7):
- Generate domain-specific can/cannot lists
- Adapt personality prose for domain expertise
- Suggest domain-specific skills to teach

## Skill Teaching Pipeline

### Generation Flow

```
Description → Derive Name → Check Duplicate → Generate Content → Validate → Save
```

### SkillError Types

| Error | Trigger |
|-------|---------|
| MissingName | No `name:` in frontmatter |
| MissingDescription | No `description:` in frontmatter |
| TooFewSteps | Less than 2 `##` headers |
| NoCodeExamples | No code blocks (```) |
| NoValidationCriteria | No Validation/Verification section |

### Validation Rules

A valid SKILL.md must have:
1. YAML frontmatter (starts with `---`)
2. `name:` and `description:` fields
3. At least 2 `##` section headers (e.g., Steps, Validation)
4. At least 1 code block
5. Validation or Verification section

### Retry Strategy

On validation failure:
1. First attempt fails → log errors
2. Return error to user with specific issues
3. User can retry with more detail

No auto-retry in MVP (deferred to Phase 7 multi-turn refinement).

## File Output

### AGENTS.md Entry

```yaml
- id: agent-id
  name: Agent Name
  role: Role Description
  avatar: 🤖
  personality_traits:
    - trait1
    - trait2
  skills:
    - skill-1
  can:
    - Capability 1
  cannot:
    - Boundary 1
```

### SOUL.md Section

```markdown
## agent-id

\```yaml
id: agent-id
communication_style: professional
tone: helpful and focused
personality_summary: Role Description specialized in domain
values:
  - accuracy
  - efficiency
boundaries:
  - Boundary 1
default_intro: I'm Agent Name, specialized in Role Description for domain.
\```

# Communication Style

Professional and focused on domain expertise.
```

### squads.yaml Entry

```yaml
squads:
  - name: template-name
    agents:
      - agent-1
      - agent-2
    coordination:
      type: hierarchical - leader leads
      communication: broadcast to all
```

## Integration Points

### Orchestrator Registration

```rust
let orchestrator = Orchestrator::new(model, session_store)
    .with_squad_builder(model.clone(), workspace_path)
    .with_skill_teacher(skills_path);
```

### Intent Routing

- `IntentType::BuildSquad` → SquadBuilder
- `IntentType::TeachSkill` → SkillTeacher

### Session Management

Both specialists return `SpecialistOutput` with `requires_confirmation: true`, triggering preview flow.

## Testing

Run all template and specialist tests:

```bash
cargo test -p aof-conversational --lib templates
cargo test -p aof-conversational --lib skill_teacher
```

