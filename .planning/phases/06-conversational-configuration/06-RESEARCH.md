# Phase 6: Conversational Configuration - Research

**Researched:** 2026-02-14
**Domain:** Conversational intent classification, natural language agent generation, orchestrator agent design, squad templates, skill teaching, session management
**Confidence:** HIGH for architecture, MEDIUM for specific implementation patterns

## Summary

Phase 6 wraps the agent creation and management system in a conversational interface. Users no longer write YAML files to create agents — they describe what they need in natural language, and the system generates agents with appropriate personas, skills, and schedules. The core concept is an orchestrator agent that understands user intent (create_agent, build_squad, configure_schedule, teach_skill, modify_agent) and delegates to specialist agents that generate AGENTS.md, SOUL.md, and SKILL.md files.

This phase sits on top of Phase 5 (agent personas), Phase 4 (Mission Control UI for the interface), Phase 3 (messaging gateway for squad announcements), and Phase 2 (skills platform for skill assignment and discovery). Phase 6 adds the conversational layer that makes agent creation feel natural instead of requiring YAML expertise.

**Primary recommendation:** Implement a three-tier architecture:
1. **Intent Classification** — Use Claude 3.5 Sonnet with few-shot examples to classify user requests into 7 core intents
2. **Specialist Agent Delegation** — Orchestrator routes to agent_creator, squad_builder, scheduler, skill_teacher, agent_modifier
3. **YAML Generation & Review** — Each specialist generates candidate files, system shows preview, user confirms before writing workspace files

**Key insight:** The conversational interface doesn't replace YAML — it generates it. Power users can still edit YAML directly (CONV-06). Conversation + YAML editing are two interfaces to the same underlying file system. Session tools (Phase 7) will eventually allow agent-to-agent messaging for multi-turn refinement, but MVP can use request-response pairs with user review.

---

## Standard Stack

### Core
| Library/Tool | Version | Purpose | Why Standard |
|--------------|---------|---------|--------------|
| Claude 3.5 Sonnet | Latest | Intent classification, YAML generation, skill teaching | Best-in-class for instruction-following, structured output (JSON mode). Proven in aof-llm already. |
| aof-llm (Anthropic provider) | v0.4.0 | LLM abstraction layer for Claude calls | Already in AOF stack, multi-provider support, consistent interface |
| serde_json | 1.0 | Intent classification responses (JSON mode) | AOF already uses, structured intent + confidence scores |
| serde_yaml | 0.9+ | YAML file parsing and generation | AOF stack, seamless Rust serialization |
| tokio | 1.35+ | Async orchestrator agent runtime | AOF foundation, event-driven execution model |
| chrono | 0.4 | Cron expression parsing for schedules | Phase 2 already integrated, timezone-aware |
| regex | 1.10 | Validate cron expressions, sanitize user input | Prevent generation of malformed schedules |
| anyhow | 1.0 | Error handling across intent → generation → validation pipeline | AOF standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| uuid | 1.6 | Unique IDs for generation sessions, agent IDs | Trace intent → generated agent |
| askama | 0.12 | Template rendering for AGENTS.md/SOUL.md stubs | Strong typing for prompt templates, avoids string concat bugs |
| tempfile | 3.8 | Temporary workspace for preview before confirming | Atomic file writes, safe rollback if user rejects |
| watchdir | 0.0.1 | Watch workspace files for external edits | Detect when user manually edits YAML, reload |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Claude 3.5 Sonnet for intent classification | Open-source models (Llama 2) | Open-source cheaper, but lower accuracy on edge cases (intent ambiguity). AOF's hosted operation justifies API cost for reliability. |
| JSON mode for intent responses | String parsing | JSON mode is strict (no hallucination risk), 100% reliable. String parsing error-prone. |
| Specialist agent delegation | Single monolithic agent | Monolithic easier to implement, but specialist agents enable parallel execution (future) and clearer separation of concerns. |
| Preview before writing | Direct write to workspace | Preview prevents accidents (user deletes agent by mistake), builds trust in system. |
| Cron scheduling | Polling-based ("every 30 min" in task queue) | Cron is standard, scales to 1000+ triggers, proven (Phase 2). Polling wastes resources. |
| File-based YAML generation | Database-backed agents | Files are inspectable, diffable, mergeable. Database adds operational burden. |

**Installation (Cargo.toml):**
```toml
# Core conversational layer
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1.35", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.8"
regex = "1.10"
uuid = { version = "1.6", features = ["v4", "serde"] }
anyhow = "1.0"

# Template rendering
askama = "0.12"

# File operations
tempfile = "3.8"
tokio-util = "0.7"  # Watch files

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Reuse from aof-llm
aof-llm = { path = "../aof-llm" }
aof-personas = { path = "../aof-personas" }
aof-core = { path = "../aof-core" }
```

---

## Intent Classification Strategy

### Intent Taxonomy (7 core intents + MVP scope)

| Intent | User Example | Phase 6? | Specialist Agent | Output |
|--------|--------------|---------|------------------|--------|
| **create_agent** | "I need a Kubernetes monitoring agent" | ✅ MVP | agent_creator | AGENTS.md entry + SOUL.md personality + skill assignments |
| **build_squad** | "Build me an incident response team" | ✅ MVP | squad_builder | Multiple AGENTS.md entries + squad composition (via squads.yaml) |
| **configure_schedule** | "Check my cluster every 30 minutes" | ✅ MVP | scheduler | Trigger config (cron expression + agent ID) |
| **teach_skill** | "Learn how to debug Postgres connections" | ✅ MVP | skill_teacher | SKILL.md file with steps, validation, examples |
| **modify_agent** | "Give the K8s agent Prometheus access" | ⚠️ Phase 7 | agent_modifier | Update AGENTS.md/SOUL.md fields |
| **list_agents** | "What agents do I have?" | ⚠️ Defer | - | Query AGENTS.md, display in chat |
| **deploy_agent** | "Activate the monitoring agent I just created" | ⚠️ Defer | - | Reload persona loaders, emit introduction event |

**MVP scope (Phase 6):** create_agent, build_squad, configure_schedule, teach_skill (4 intents)
**Deferred (Phase 7+):** modify_agent, list_agents, deploy_agent

### Intent Classification Flow

```
User Message
    ↓
Intent Classifier (Claude 3.5 Sonnet, JSON mode)
    ├─ Extract intent type
    ├─ Extract confidence (0-1)
    ├─ Extract parameters (e.g., agent_type, skills, schedule)
    └─ Extract clarifying questions if confidence < 0.8
    ↓
    ├─ Confidence ≥ 0.8 → Route to specialist agent
    │
    └─ Confidence < 0.8 → Ask clarifying questions in chat
        ↓
        User provides clarification
        ↓
        Re-classify with updated context
```

### Intent Classification System Prompt

```
You are an intent classifier for an agentic ops system.
Classify user messages into one of these categories:

1. create_agent: User wants to create a single agent (e.g., "I need a Kubernetes monitor")
2. build_squad: User wants to create multiple agents as a team (e.g., "Build incident response squad")
3. configure_schedule: User wants to set up a cron schedule (e.g., "Check cluster every 30 min")
4. teach_skill: User wants to create a reusable skill (e.g., "Learn how to debug Postgres")
5. unknown: Doesn't match any category, or request is ambiguous

Respond in JSON:
{
  "intent": "create_agent" | "build_squad" | "configure_schedule" | "teach_skill" | "unknown",
  "confidence": 0.0-1.0,
  "parameters": {
    "agent_type": string,  // e.g., "kubernetes", "incident-response"
    "skills": [string],    // e.g., ["pod-debugging", "log-analysis"]
    "schedule": string,    // e.g., "every 30 minutes" or "*/30 * * * *"
    "skill_name": string,  // e.g., "postgres-debugging"
    "description": string  // User's description
  },
  "clarifying_questions": [string] // If confidence < 0.8
}

Important:
- Be conservative: prefer lower confidence if user request is ambiguous
- Extract parameters even if confidence is low
- Suggest clarifying questions for ambiguous requests
```

### Confidence Thresholds & Routing

```
Confidence ≥ 0.8
├─ Route directly to specialist agent
├─ Generate YAML immediately
└─ Show preview for user confirmation

Confidence 0.5-0.79
├─ Ask 1-2 clarifying questions
├─ Wait for user response
└─ Re-classify, then proceed

Confidence < 0.5
├─ Apologize, explain what you understood
├─ Suggest examples of what you can do
└─ Ask user to rephrase
```

### Few-Shot Examples for Classifier

Embed these in the system prompt to improve classification:

```json
{
  "examples": [
    {
      "user": "I need a bot that watches my Kubernetes cluster for pod crashes",
      "intent": "create_agent",
      "confidence": 0.95,
      "parameters": { "agent_type": "kubernetes-monitor", "skills": ["pod-debugging", "log-analysis", "alerting"] }
    },
    {
      "user": "Build me a team to respond to incidents",
      "intent": "build_squad",
      "confidence": 0.85,
      "parameters": { "squad_type": "incident-response", "team_size": 3 }
    },
    {
      "user": "Check the cluster every 30 minutes",
      "intent": "configure_schedule",
      "confidence": 0.9,
      "parameters": { "schedule": "*/30 * * * *", "agent_id": "k8s-monitor" }
    },
    {
      "user": "Make the bot aware of our Postgres performance anti-patterns",
      "intent": "teach_skill",
      "confidence": 0.88,
      "parameters": { "skill_name": "postgres-performance-debugging", "domain": "database" }
    }
  ]
}
```

---

## Orchestrator Agent Design

### Architecture

```
User Chat (Mission Control UI or Slack)
    ↓
    ┌─────────────────────────────────────────┐
    │ Orchestrator Agent (Main Thread)         │
    │ - Intent classification                  │
    │ - Context management (session memory)    │
    │ - Multi-turn conversation support        │
    └─────────────────────────────────────────┘
    ↓
    ├─→ Agent Creator Specialist       → Generates AGENTS.md entry
    ├─→ Squad Builder Specialist       → Generates squad composition
    ├─→ Scheduler Specialist           → Generates cron + agent binding
    └─→ Skill Teacher Specialist       → Generates SKILL.md
    ↓
    ┌─────────────────────────────────────────┐
    │ File Generation & Preview                │
    │ - Render Markdown templates              │
    │ - Validate YAML syntax                   │
    │ - Show preview in chat                   │
    └─────────────────────────────────────────┘
    ↓
    User Review & Confirmation
    ↓
    ┌─────────────────────────────────────────┐
    │ Workspace File Persistence               │
    │ - Write to workspace/AGENTS.md           │
    │ - Write to workspace/SOUL.md             │
    │ - Write to skills/[skill-name]/SKILL.md  │
    └─────────────────────────────────────────┘
```

### Orchestrator Session State

Orchestrator maintains per-conversation state:

```rust
struct ConversationSession {
    pub session_id: String,                    // UUID
    pub user_id: String,                       // From auth (Phase 8)
    pub messages: Vec<ConversationMessage>,    // Chat history
    pub current_intent: Option<Intent>,        // Classified intent
    pub pending_files: HashMap<String, String>, // File path → content (preview)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,             // Garbage collect after 30min idle
}

struct ConversationMessage {
    pub role: Role,  // "user" | "assistant" | "system"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

**Session lifetime:**
- Created: User opens chat in Mission Control UI
- Active: User sends messages, state updated
- Expires: 30 minutes without activity, cleaned up (files discarded)
- Persisted: If user confirms file creation, session result saved to audit log

### Orchestrator System Prompt Structure

```
# Role & Context
You are the Orchestrator Agent for an agentic ops system.
Your role is to understand what users want and guide them through creating agents.

# Your Capabilities
1. Create single agents from natural language descriptions
2. Build agent squads (teams) for specific use cases
3. Set up schedules and heartbeats
4. Teach the system new skills
5. Preview files before the user saves them

# Boundaries
- You CANNOT: Directly modify running agents (Phase 7+)
- You CANNOT: Deploy agents to production (requires confirmation)
- You CANNOT: Access external systems (agents will do that)
- You CAN: Suggest, preview, and guide creation

# Communication Style
- Be helpful and assume the user knows their domain (Kubernetes, incident response, etc.)
- Ask clarifying questions when intent is unclear
- Show previews of what you're about to create
- Celebrate when agents are created successfully

# Process
1. Understand what the user wants
2. Ask clarifying questions if needed
3. Propose a solution (agent, squad, skill, schedule)
4. Generate YAML files as preview
5. Ask for confirmation
6. Save files to workspace on confirmation
```

### Multi-Turn Conversation Support

The orchestrator should handle multi-turn exchanges:

```
User: "I need a Kubernetes monitoring agent"
Orchestrator: "Great! I'll help you create one. Let me ask a few questions:
  1. What should it monitor? (e.g., pod crashes, CPU usage, memory)
  2. How often should it check? (e.g., every 5 minutes)
  3. What platforms should it alert on? (e.g., Slack, Discord, email)"

User: "Check for pod crashes every 5 minutes, alert in Slack"
Orchestrator: "Perfect. Here's what I'll create: [preview AGENTS.md entry]
Should I create this agent?"

User: "Yes, but add Prometheus access"
Orchestrator: "Done. Updated preview: [modified AGENTS.md]
Confirm?"

User: "Yes"
Orchestrator: "Agent created! 🎉 Find it in workspace/AGENTS.md. It will introduce itself
in the squad chat when activated."
```

### Session Memory Strategy

Store conversation history for:
- Context retrieval (user says "add Prometheus to the agent we just created")
- Decision audit (why was agent created with those skills)
- Error recovery (user changes mind, rolls back)

**NOT persisted after session ends** (MVP scope):
- Conversation history discarded if user closes browser
- Pending files not saved
- Full context management deferred to Phase 7 (session tools)

---

## Agent Generation Pipeline

### Flow: Intent → Specialist Agent → YAML → Preview → Confirmation → Persistence

```
┌────────────────────────┐
│ 1. Intent Received     │ "I need a K8s monitoring agent"
└────────────┬───────────┘
             ↓
┌────────────────────────────────────────────┐
│ 2. Classify & Extract Parameters           │
│ - Intent: create_agent                     │
│ - Agent type: kubernetes-monitor           │
│ - Skills: [pod-debugging, log-analysis]    │
│ - Confidence: 0.92                         │
└────────────┬───────────────────────────────┘
             ↓
┌────────────────────────────────────────────┐
│ 3. Route to Specialist Agent               │
│ Agent Creator Specialist (Claude 3.5)      │
│ Input: {agent_type, skills, description}   │
│ Task: Generate AGENTS.md entry + SOUL.md   │
└────────────┬───────────────────────────────┘
             ↓
┌────────────────────────────────────────────┐
│ 4. Generation (Specialist Prompt)          │
│ "Generate a Kubernetes monitoring agent... │
│ with skills: pod-debugging, log-analysis"  │
│ Output format: YAML (AGENTS.md entry)      │
└────────────┬───────────────────────────────┘
             ↓
┌────────────────────────────────────────────┐
│ 5. Validation                              │
│ - YAML syntax check (serde_yaml)           │
│ - Agent ID uniqueness (check existing)     │
│ - Skill discovery (verify skills exist)    │
│ - Persona validation                       │
└────────────┬───────────────────────────────┘
             ↓ (if validation succeeds)
┌────────────────────────────────────────────┐
│ 6. Preview                                 │
│ Show user the files to be created:         │
│ - AGENTS.md entry snippet                  │
│ - SOUL.md personality preview              │
│ - Skills that will be assigned             │
│ "Does this look right?"                    │
└────────────┬───────────────────────────────┘
             ↓ (if user confirms)
┌────────────────────────────────────────────┐
│ 7. Persistence                             │
│ - Append to workspace/AGENTS.md            │
│ - Create workspace/SOUL_[agent_id].md      │
│ - Log creation to audit trail              │
│ - Reload persona loaders (Phase 5)         │
└────────────┬───────────────────────────────┘
             ↓
┌────────────────────────────────────────────┐
│ 8. Success Response                        │
│ "Agent created! It will introduce itself   │
│ when activated. Here's what it can do: ..." │
└────────────────────────────────────────────┘
```

### AGENTS.md Entry Generation Template

**Input to specialist agent:**
```
Generate a YAML entry for AGENTS.md with the following constraints:
- Agent ID: [kebab-case unique ID]
- Agent name: [User-friendly name]
- Role: [What the agent does]
- Avatar: [Single emoji]
- Personality traits: [3-5 traits, e.g., methodical, detail-oriented]
- Skills: [Array of skill IDs agent will use]
- CAN: [What agent is capable of]
- CANNOT: [What agent explicitly cannot do]

Format output as YAML, ready to be inserted into AGENTS.md agents list.
```

**Specialist agent output example:**
```yaml
- id: k8s-monitor
  name: "Kubernetes Monitor"
  role: "Infrastructure Specialist"
  avatar: "🤖"
  personality_traits:
    - methodical
    - detail-oriented
    - proactive
  can:
    - Monitor pod health via kubectl
    - Analyze container logs
    - Query metrics from Prometheus
    - Alert on anomalies
  cannot:
    - Modify cluster infrastructure
    - Execute arbitrary commands
    - Access production databases directly
  skills:
    - pod-debugging
    - log-analysis
    - prometheus-queries
```

### SOUL.md Personality Generation

**Input to specialist agent:**
```
Generate a SOUL.md personality section for a Kubernetes monitoring agent.
Consider:
- Communication style: professional, methodical, detail-focused
- Confidence level: high (this is core competency)
- Error handling: thorough investigation before suggesting fixes
- Preferences: proactive reporting, minimal false alarms

Output as Markdown with YAML frontmatter, ready to use as workspace/SOUL.md.
```

**Specialist agent output example:**
```markdown
---
agent_id: k8s-monitor
emoji: 🤖
traits:
  - methodical
  - detail-oriented
  - proactive
---

# Kubernetes Monitor Personality

## Communication Style
I speak with the precision of a systems engineer. I avoid speculation and
report only what I've verified. When something is wrong, I explain the root
cause and suggest fixes backed by evidence from logs and metrics.

## Strengths
- Thorough investigation of infrastructure issues
- Attention to detail in log analysis
- Proactive alerting before problems escalate

## Decision-Making
I err on the side of caution. If I'm not confident in a diagnosis, I
escalate to the incident response team rather than making assumptions.
```

### Skill Generation for teach_skill Intent

**Input to specialist agent:**
```
User wants to teach the system: "Learn how to debug Postgres connections"

Generate a SKILL.md file that:
1. Captures the steps the user describes
2. Includes validation criteria (how to verify the skill worked)
3. Has concrete examples (e.g., common error messages)
4. Lists requirements (e.g., psql CLI, database credentials)

Output as Markdown with YAML frontmatter.
```

**Output example:**
```markdown
---
name: postgres-connection-debugging
description: Debug and diagnose Postgres connection issues
metadata:
  emoji: 🐘
  version: 1.0.0
  author: Your Team
  requires:
    bins: ["psql"]
    env: ["PG_HOST", "PG_USER", "PG_PASSWORD"]
  tags:
    - database
    - debugging
    - postgresql
---

# Postgres Connection Debugging

## Steps

1. **Test connectivity**
   ```bash
   psql -h $PG_HOST -U $PG_USER -c "SELECT 1"
   ```
   Expected output: "1" with no errors

2. **Check network reachability**
   - If connection times out, verify network ACLs
   - Ensure port 5432 is open from client to server

3. **Validate credentials**
   - Check PG_USER and PG_PASSWORD are correct
   - Verify user has login privilege in pg_roles

## Common Issues

- **FATAL: password authentication failed**: Check credentials
- **FATAL: no pg_hba.conf entry for...**: Database host config missing client IP
- **Connection refused**: Database not running or listening on wrong interface

## Validation

✓ psql connects successfully
✓ SELECT query returns expected result
✓ Connection timing is acceptable (<1s)
```

### Validation Rules

```rust
pub enum GenerationError {
    // Syntax errors
    InvalidYaml(String),
    MalformedCron(String),

    // Semantic errors
    DuplicateAgentId(String),
    SkillNotFound(String),
    InvalidSchedule(String),

    // Generation errors
    HallucinatedCapabilities(String), // Agent claimed skills it doesn't have
    ConflictingSkills(String),        // Incompatible skill combinations
    TokenLimitExceeded,               // Generated prompt too long
}

pub fn validate_generated_agent(
    agent: &Agent,
    existing_agents: &[Agent],
    available_skills: &[String],
) -> Result<(), Vec<GenerationError>> {
    let mut errors = vec![];

    // Check ID uniqueness
    if existing_agents.iter().any(|a| a.id == agent.id) {
        errors.push(GenerationError::DuplicateAgentId(agent.id.clone()));
    }

    // Check all skills exist
    for skill in &agent.skills {
        if !available_skills.contains(skill) {
            errors.push(GenerationError::SkillNotFound(skill.clone()));
        }
    }

    // Check CAN/CANNOT consistency
    for capability in &agent.can {
        for forbidden in &agent.cannot {
            if capability.contains(forbidden) {
                errors.push(GenerationError::ConflictingSkills(
                    format!("Agent claims it both can and cannot: {}", capability)
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

---

## Squad Templates & Customization

### Pre-Built Squad Templates

Template library lives in `crates/aof-personas/squad_templates/`:

#### 1. Incident Response Squad

```yaml
name: incident-response
description: "Triage and resolve incidents collaboratively"
agents:
  - id: incident-triage
    name: "Incident Triage Agent"
    role: "First Responder"
    skills: [alert-parsing, severity-classification, context-gathering]

  - id: log-analyzer
    name: "Log Analyzer"
    role: "Forensics Specialist"
    skills: [log-search, pattern-matching, timeline-construction]

  - id: metric-checker
    name: "Metrics Specialist"
    role: "Performance Inspector"
    skills: [prometheus-queries, metric-correlation, anomaly-detection]

  - id: remediation-executor
    name: "Remediator"
    role: "Actions Specialist"
    skills: [kubectl-operations, service-restart, traffic-reroute]

squad_config:
  coordination: incident-response-flow
  escalation_path: [triage, log-analyzer, metric-checker, remediation-executor]
  communication: incident-war-room-channel
```

#### 2. Monitoring Squad

```yaml
name: monitoring
description: "Continuous health observation and alerting"
agents:
  - id: k8s-monitor
    name: "Kubernetes Monitor"
    role: "Cluster Health Warden"
    skills: [kubectl-operations, pod-health-check, event-parsing]

  - id: metric-monitor
    name: "Metrics Monitor"
    role: "Performance Warden"
    skills: [prometheus-queries, threshold-checking, alert-generation]

  - id: alert-router
    name: "Alert Router"
    role: "Communications Coordinator"
    skills: [slack-posting, pagerduty-integration, escalation-logic]

squad_config:
  coordination: heartbeat-loop (every 5 minutes)
  sla_targets: ["detect issues within 60s", "alert humans within 90s"]
```

#### 3. Deployment Squad

```yaml
name: deployment
description: "Safe, observable deployments with rollback capability"
agents:
  - id: pre-flight-checker
    name: "Pre-Flight Checker"
    role: "Validation Specialist"
    skills: [health-checks, dependency-verification, canary-simulation]

  - id: deployer
    name: "Deployer"
    role: "Execution Specialist"
    skills: [kubectl-apply, helm-deploy, image-rollout]

  - id: post-deploy-verifier
    name: "Post-Deploy Verifier"
    role: "Quality Specialist"
    skills: [health-monitoring, smoke-tests, metric-validation]

squad_config:
  coordination: deployment-workflow
  gates: [pre-flight → approval → deploy → verify]
  rollback_trigger: "if metrics degrade >10%"
```

#### 4. Cost Optimization Squad

```yaml
name: cost-optimization
description: "Identify and remediate cloud cost inefficiencies"
agents:
  - id: cost-analyzer
    name: "Cost Analyzer"
    role: "Financial Engineer"
    skills: [cloud-cost-parsing, trend-analysis, anomaly-detection]

  - id: optimizer-suggester
    name: "Optimizer"
    role: "Recommendation Engine"
    skills: [cost-saving-patterns, sizing-optimization, reserved-instance-analysis]

  - id: remediator
    name: "Implementation Specialist"
    role: "Executor"
    skills: [terraform-modification, instance-right-sizing, reservation-purchase]

squad_config:
  coordination: weekly-cost-review
  decision_gate: cost-threshold ($100+/month savings required)
```

### Squad Customization Pattern

When user says "Build incident response squad for Postgres":

```
User Intent: "Build incident response squad for Postgres"
    ↓
Specialist Agent (Squad Builder)
    - Template: incident-response
    - Customization: Domain-specific (Postgres)
    ↓
Customization Pipeline:
    1. Load incident-response template
    2. Replace generic log-analyzer with postgres-specific-analyzer
    3. Update skills: [postgresql-log-parsing, connection-pool-analysis]
    4. Adjust prompts: system prompts mention Postgres anti-patterns
    5. Add domain-specific tool: psql CLI access
    ↓
Output: Modified agent list + customized SOUL.md entries
```

**Customization template:**
```yaml
template: incident-response
domain: postgresql
customizations:
  agents:
    - id: log-analyzer → postgres-log-analyzer
      skills:
        add: [postgresql-log-parsing, connection-pool-analysis]
        remove: [generic-log-search]
      soul_additions: |
        # Postgres Expertise
        I'm familiar with PostgreSQL error codes, connection pool exhaustion,
        and common performance anti-patterns. I correlate slow query logs
        with lock contention and index bloat.

  tools:
    add: [psql-cli, pg-stat-analysis]

  coordination:
    add_step: "Check pg_stat_activity for blocking queries"
```

### Template Storage & Discovery

**Rust implementation:**
```rust
pub struct SquadTemplate {
    pub name: String,
    pub description: String,
    pub agents: Vec<TemplateAgent>,
    pub squad_config: SquadConfig,
    pub customization_hints: Vec<String>,
}

pub struct SquadTemplateLibrary {
    templates: HashMap<String, SquadTemplate>,
}

impl SquadTemplateLibrary {
    pub fn load_builtin() -> Self {
        // Load YAML from crates/aof-personas/squad_templates/*.yaml
    }

    pub fn get(&self, name: &str) -> Option<&SquadTemplate> {
        self.templates.get(name)
    }

    pub fn list(&self) -> Vec<&SquadTemplate> {
        // Return all templates with brief descriptions
    }

    pub fn customize(
        &self,
        template_name: &str,
        domain: &str,
    ) -> Result<CustomizedSquad, Error> {
        // Apply domain-specific customizations
    }
}
```

**Files structure:**
```
crates/aof-personas/
├── squad_templates/
│   ├── incident-response.yaml
│   ├── monitoring.yaml
│   ├── deployment.yaml
│   ├── cost-optimization.yaml
│   └── customizations/
│       ├── incident-response-postgresql.yaml
│       ├── incident-response-kubernetes.yaml
│       └── monitoring-aws.yaml
└── squad_builder.rs  // Logic to apply customizations
```

---

## Conversational Skill Building Workflow

### Single-Shot vs. Multi-Turn Approach

**Recommendation: Start with single-shot (MVP), add multi-turn in Phase 7**

**Single-shot (Phase 6 MVP):**
- User: "Learn how to debug our Postgres connections"
- System: Generates SKILL.md immediately from description
- Flow: Description → Claude generation → Preview → Confirmation → Save

**Multi-turn (Phase 7 with session tools):**
- User: "Learn how to debug our Postgres connections"
- System: "I'll create this skill. Tell me:
  1. What are the most common failure modes?
  2. What tools should agents use? (psql, logs, monitoring)
  3. How do you validate the skill worked?"
- System: Incorporates answers into richer SKILL.md

### Single-Shot Skill Generation

```
User: "Learn how to debug Postgres connections"
    ↓
Skill Teacher Specialist (Claude 3.5)
    Input prompt:
    """
    The user wants to teach the system this skill:
    "Learn how to debug Postgres connections"

    Generate a SKILL.md file that:
    1. Lists diagnostic steps (e.g., psql commands)
    2. Includes common error messages and fixes
    3. Has validation criteria (how to verify the skill worked)
    4. Lists tools/requirements (e.g., psql, database credentials)

    Output format: Markdown with YAML frontmatter (see SKILL.md spec)
    """

    Output: SKILL.md content
    ↓
Validation:
    - YAML frontmatter parses correctly
    - name, description fields present
    - Steps are clear and numbered
    - Examples include concrete commands

    ↓
Preview in chat:
    "I'll create this skill. Does it look right?"
    [Show 200-char preview]

    ↓
User: "Yes" / "Refine this" / "No, start over"

    ↓ (if "Yes")
Persistence:
    skills/postgres-connection-debugging/SKILL.md

    ↓
Success: "Skill created! Agents can now use it via 'postgres-connection-debugging'"
```

### Skill Validation Criteria

```rust
pub fn validate_generated_skill(skill: &Skill) -> Result<(), Vec<SkillError>> {
    let mut errors = vec![];

    // Frontmatter
    if skill.name.is_empty() {
        errors.push(SkillError::MissingName);
    }
    if skill.description.is_empty() {
        errors.push(SkillError::MissingDescription);
    }

    // Content
    if skill.content.is_empty() {
        errors.push(SkillError::NoSteps);
    }

    // Step quality
    let step_count = skill.content.matches("##").count();
    if step_count < 2 {
        errors.push(SkillError::TooFewSteps);
    }

    // Examples
    if !skill.content.contains("```") {
        errors.push(SkillError::NoCodeExamples);
    }

    // Validation criteria
    if !skill.content.contains("Validation") && !skill.content.contains("verification") {
        errors.push(SkillError::NoValidationCriteria);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Version Management

When user teaches skill twice:

```
First time: "Learn how to debug Postgres connections"
├─ Creates: skills/postgres-debugging/SKILL.md (v1.0.0)
└─ Metadata: author=user, created=2026-02-14

Second time: "Update Postgres debugging to include connection pool analysis"
├─ Detects: skills/postgres-debugging/SKILL.md already exists
├─ Options:
│  1. Update in-place (replace v1.0.0)
│  2. Create new version (postgres-debugging-v2)
│  3. Ask user which approach
└─ Recommendation: Ask user

Recommendation: Keep it simple for MVP
- Detect duplicate by name
- Ask: "Update existing skill or create new variant?"
- If update: back up old version (version control via git)
- If new: increment in name (postgres-debugging-connection-pool-analysis)
```

---

## Schedule Configuration from Conversation

### Natural Language to Cron Parsing

**User examples → cron expressions:**

| User Input | Cron Expression | Interpretation |
|------------|-----------------|-----------------|
| "every 30 minutes" | `*/30 * * * *` | Every 30 minutes |
| "every weekday at 6am EST" | `0 6 * * 1-5` + TZ=America/New_York | Monday-Friday 6:00am |
| "daily at noon" | `0 12 * * *` | Every day at 12:00pm UTC |
| "every Monday and Friday" | `0 0 * * 1,5` | Midnight Monday and Friday |
| "3x per day: 6am, 12pm, 6pm" | `0 6,12,18 * * *` | Three times daily |
| "business hours only (9-5)" | `0 9-17 * * *` | Every hour 9am-5pm |

### Schedule Configuration Specialist

```
User: "Check my cluster every 30 minutes"
    ↓
Intent Classification:
    Intent: configure_schedule
    Parameters: agent=k8s-monitor, frequency="every 30 minutes"
    ↓
Scheduler Specialist Input:
    """
    User wants to set up a schedule:
    "Check my cluster every 30 minutes"

    Available agents: [k8s-monitor, log-analyzer, cost-optimizer]

    1. Parse natural language schedule to cron expression
    2. Ask which agent should run on this schedule (if ambiguous)
    3. Generate trigger configuration

    Output:
    {
      "cron": "*/30 * * * *",
      "timezone": "UTC",
      "agent_id": "k8s-monitor",
      "description": "Check cluster health every 30 minutes"
    }
    """
    ↓
Schedule Parsing:
    "every 30 minutes" → */30 * * * * (cron standard)
    ↓
Validation:
    - Cron expression is valid
    - Agent exists in AGENTS.md
    - Schedule is not conflicting (optional: detect if agent already has schedule)
    ↓
Preview:
    "I'll set up the k8s-monitor to run every 30 minutes (UTC).
     You can change the timezone if needed."
    ↓
User confirmation → Write to config or triggers.yaml
```

### Trigger Configuration Output

```yaml
# triggers.yaml (or embedded in aofctl serve config)
schedules:
  - id: k8s-monitor-heartbeat
    agent_id: k8s-monitor
    trigger:
      type: Schedule
      schedule: "*/30 * * * *"
      timezone: UTC
    description: "Check cluster every 30 minutes"
```

### Cron Expression Validation

```rust
use chrono_tz::Tz;
use cron::Schedule;

pub fn validate_schedule(
    cron_expr: &str,
    timezone: &str,
) -> Result<ValidatedSchedule, ScheduleError> {
    // Parse cron
    let schedule: Schedule = cron_expr.parse()
        .map_err(|_| ScheduleError::InvalidCron)?;

    // Parse timezone
    let tz: Tz = timezone.parse()
        .map_err(|_| ScheduleError::InvalidTimezone)?;

    // Generate next 3 runs to verify
    let now = chrono::Utc::now();
    let upcoming = schedule.next_after(&now)
        .take(3)
        .collect::<Vec<_>>();

    if upcoming.is_empty() {
        return Err(ScheduleError::NoFutureRuns);
    }

    Ok(ValidatedSchedule {
        cron: cron_expr.to_string(),
        timezone: timezone.to_string(),
        next_runs: upcoming,
    })
}
```

---

## YAML Preview & Edit Layer (Power Users)

### Preview System

After specialist agent generates files, show preview:

```
┌─────────────────────────────────────────────────┐
│ Preview: Agent Created Successfully             │
├─────────────────────────────────────────────────┤
│                                                 │
│ File 1: workspace/AGENTS.md                    │
│ ──────────────────────────────────────────────  │
│ - id: postgres-monitor                         │
│   name: "Postgres Monitor"                    │
│   role: "Database Specialist"                 │
│   avatar: 🐘                                   │
│   skills:                                      │
│     - postgres-connection-debugging            │
│     - slow-query-analysis                     │
│                                                 │
│ [Show Full] [Edit Before Saving] [Confirm]    │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Edit-Before-Saving Flow

1. **System generates YAML**
2. **Show preview in modal**
3. **User can click "Edit"**
4. **Opens syntax-highlighted YAML editor**
5. **User makes changes**
6. **System re-validates changed YAML**
7. **User confirms "Save changes"**
8. **System writes to workspace files**

**Implementation:**
```rust
pub struct GenerationPreview {
    pub session_id: String,
    pub files: HashMap<String, FilePreview>,  // path → content
    pub validation_state: ValidationState,     // pending, valid, invalid
}

pub struct FilePreview {
    pub path: String,           // workspace/AGENTS.md
    pub content: String,        // Full file content
    pub syntax: String,         // "yaml", "markdown"
    pub editable: bool,         // Can user modify before saving?
}

pub enum ValidationState {
    Pending,
    Valid,
    Invalid(Vec<String>),  // Error messages
}
```

### Conversation Reference to YAML

Support user saying "Use what I showed you in the YAML":

```
User: [pastes custom YAML snippet]
User: "Use this in the agent definition"

Orchestrator:
1. Detect user provided raw YAML
2. Parse and validate
3. Incorporate into generated config
4. Show updated preview
5. Confirm with user
```

---

## Integration with Phase 4 Mission Control UI

### Conversational Interface Placement

**Location in UI:** New tab/section "Create Agent" or sidebar panel

```
┌─────────────────────────────────────────┐
│ AOF Mission Control                     │
├─────────────────────────────────────────┤
│ [Dashboard] [Squad Chat] [Tasks]        │
│ [Create Agent] ← NEW                    │
├─────────────────────────────────────────┤
│                                         │
│ ┌────────────────────────────────────┐ │
│ │ Create Agent                       │ │
│ │                                    │ │
│ │ Orchestrator: "What agent would   │ │
│ │ you like to create? (e.g., 'K8s   │ │
│ │ monitoring', 'incident response')  │ │
│ │                                    │ │
│ │ ┌──────────────────────────────┐  │ │
│ │ │ [User types here...          │  │ │
│ │ │                              │  │ │
│ │ │                              │  │ │
│ │ │                          [Send] │ │
│ │ └──────────────────────────────┘  │ │
│ │                                    │ │
│ └────────────────────────────────────┘ │
│                                         │
└─────────────────────────────────────────┘
```

### Chat vs. Wizard vs. Form Approach

**Recommendation: Chat-based (conversational) as primary, with optional wizard mode**

| Approach | Pros | Cons | Best For |
|----------|------|------|----------|
| **Chat** | Natural, conversational, flexible, scales to all intents | Requires more backend logic, less structured UX | MVP Phase 6, users comfortable with natural language |
| **Wizard** | Structured, step-by-step, clear input validation | Rigid (doesn't adapt to user needs), boilerplate | Advanced users who prefer forms, complex workflows |
| **Form** | Familiar, quick to implement, validates fields | Confusing (too many optional fields), not discoverable | Power users who know exactly what they want |

**Implementation plan:**
1. **Phase 6 MVP:** Chat interface (freeform text input)
2. **Phase 6 Phase 2:** Wizard toggle for users who prefer step-by-step
3. **Phase 7+:** Form mode for power users (YAML direct edit)

### Chat UI Component (React)

```typescript
interface ConversationMessage {
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
  filePreview?: GenerationPreview;  // If assistant generated files
}

interface CreateAgentPanel {
  sessionId: string;
  messages: ConversationMessage[];
  inputValue: string;
  isLoading: boolean;
  pendingFiles?: Map<string, string>;  // path → content
}

// Component structure:
<CreateAgentPanel>
  <ChatHistory>
    {messages.map(msg => <Message msg={msg} />)}
  </ChatHistory>

  {pendingFiles && (
    <FilePreview
      files={pendingFiles}
      onEdit={handleEditFile}
      onConfirm={handleSaveFiles}
      onCancel={handleCancelGeneration}
    />
  )}

  <ChatInput
    value={inputValue}
    onChange={handleInputChange}
    onSend={handleSendMessage}
    placeholder="e.g., 'Create a K8s monitoring agent'"
  />
</CreateAgentPanel>
```

### Real-Time Feedback & Validation

As user types:
- Show intent suggestions (e.g., "Did you mean: create_agent?")
- Show skill tags that match keywords typed
- Show agent templates that might apply

```typescript
// As user types, show sidebar with suggestions
const suggestions = [
  { type: "intent", label: "Create Agent", match: "create" },
  { type: "template", label: "Monitoring Squad", match: "monitor" },
  { type: "skill", label: "K8s operations", match: "k8s" },
];
```

---

## Integration Considerations

### Phase 4 UI (Where Conversation Lives)

- **New page:** `/dashboard/create-agent` (route in React Router)
- **New tab in main dashboard:** Toggle between "Overview" and "Create Agent"
- **Sidebar shortcut:** "Create Agent" button in navigation
- **Recommendation:** New page for MVP (cleaner separation), later add tab view

### Phase 5 Personas (Auto-Generation of Personality)

When creating agent, system should:

1. **Infer personality from keywords:**
   - "monitoring agent" → methodical, detail-oriented, proactive
   - "incident response" → decisive, thorough, calm under pressure
   - "cost optimizer" → analytical, precise, optimization-minded

2. **Generate SOUL.md automatically:**
   - Take user description + inferred traits
   - Pass to Claude: "Generate a personality section for this agent"
   - Show in preview before saving

3. **Capability inference:**
   - Based on skills, auto-generate CAN/CANNOT section
   - User can override in edit mode

### Phase 2 Skills (Skill Lookup & Assignment)

- Orchestrator queries available skills via Phase 2 API
- When generating agent, suggest relevant skills based on agent type
- Skill discovery in conversation: user says "what skills are available?" → list them

### Phase 7 Session Tools (Agent-to-Agent Communication)

**Phase 6 does NOT implement session tools.**
**Phase 7 will add:**
- Agent A asks Agent B to refine generated agent
- Multi-turn refinement loop (user says "improve this", agent asks clarifying questions)
- Agents collaborate on complex squad composition

**For Phase 6 MVP:**
- Request-response model (user → orchestrator → specialist → user)
- Preview → manual confirmation
- No agent-to-agent messaging

---

## Implementation Approach

### Recommended Tech Stack

| Component | Tech | Rationale |
|-----------|------|-----------|
| Intent classification | Claude 3.5 Sonnet (JSON mode) | Proven in production, structured output, handles ambiguity well |
| Orchestrator agent runtime | Rust + tokio | AOF native, async-first, integrates with aof-llm |
| Specialist agents | Claude 3.5 Sonnet | Same provider as orchestrator for consistency |
| YAML generation | serde_yaml + askama templates | Type-safe templates, avoid string concat errors |
| Conversation UI | React + Redux Toolkit | Already in Phase 4, familiar patterns |
| WebSocket communication | Existing Phase 1 infrastructure | Reuse event broadcaster |
| File watching | notify crate (Rust) | Detect external YAML edits, reload |

### Crates Needed

**New crate: `aof-conversational`** (or `aof-orchestrator`)

```toml
# crates/aof-conversational/Cargo.toml
[package]
name = "aof-conversational"
version.workspace = true

[dependencies]
aof-core = { path = "../aof-core" }
aof-llm = { path = "../aof-llm" }
aof-personas = { path = "../aof-personas" }
aof-runtime = { path = "../aof-runtime" }

tokio = { workspace = true, features = ["full"] }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

# Template rendering
askama = "0.12"

# Cron/scheduling
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.8"
regex = "1.10"

# File operations
tokio-util = "0.7"  # Watch files
notify = "6.1"
tempfile = "3.8"

# UUID for session IDs
uuid = { version = "1.6", features = ["v4", "serde"] }
```

### Session Management: In-Memory Cache

For MVP, store sessions in-memory with TTL:

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct ConversationSessionStore {
    sessions: Arc<RwLock<LruCache<String, SessionWithTimestamp>>>,
    ttl: Duration,
}

struct SessionWithTimestamp {
    session: ConversationSession,
    created_at: Instant,
    last_activity: Instant,
}

impl ConversationSessionStore {
    pub async fn get(&self, session_id: &str) -> Option<ConversationSession> {
        let mut sessions = self.sessions.write().await;

        if let Some(entry) = sessions.get(session_id) {
            if entry.last_activity.elapsed() < self.ttl {
                return Some(entry.session.clone());
            }
        }

        None  // TTL expired or not found
    }

    pub async fn update(&self, session: ConversationSession) {
        let mut sessions = self.sessions.write().await;
        sessions.put(session.session_id.clone(), SessionWithTimestamp {
            session,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        });
    }
}
```

### Suggested Plan Decomposition (4-5 plans for Phase 6)

**Plan 06-01: Intent Classification & Orchestrator Architecture** (1 week)
- Intent classifier (Claude + JSON mode)
- Orchestrator agent skeleton
- Session management (in-memory store)
- Conversation message routing
- Tests: Intent classification accuracy, session lifecycle

**Plan 06-02: Agent Generation Specialist** (1 week)
- Agent Creator specialist agent implementation
- AGENTS.md template rendering
- SOUL.md personality generation
- Validation pipeline (syntax, uniqueness, skill existence)
- Tests: Agent generation, validation, error messages

**Plan 06-03: Squad Templates & Skill Teaching** (1 week)
- Squad template library (incident response, monitoring, deployment)
- Squad Builder specialist agent
- Skill Teacher specialist agent
- SKILL.md generation and validation
- Tests: Template customization, skill generation, version management

**Plan 06-04: Schedule Configuration & Integration** (1 week)
- Scheduler specialist agent (natural language → cron)
- Cron expression validation and parsing
- Integration with Phase 2 scheduler
- Tests: Schedule parsing (edge cases: timezones, business hours)

**Plan 06-05: UI & Full Integration** (1.5 weeks)
- React components: ConversationPanel, ChatInput, FilePreview
- WebSocket integration with orchestrator
- Preview modal with edit capability
- Integration tests: end-to-end conversation → file creation
- Documentation and examples

### Estimated Effort Per Plan

| Plan | Tasks | LOC Est. | Days | Risk |
|------|-------|---------|------|------|
| 06-01 | Intent classifier, orchestrator, session store | 1200 | 4-5 | LOW |
| 06-02 | Agent generation, validation, templates | 1500 | 4-5 | LOW |
| 06-03 | Squad templates, skill teaching | 1800 | 5-6 | MEDIUM (skill teaching scope) |
| 06-04 | Schedule parsing, cron validation | 800 | 3-4 | LOW |
| 06-05 | React UI, WebSocket integration, E2E tests | 2000 | 6-7 | MEDIUM (UI complexity) |
| **Total** | | **7300** | **22-27 days** | |

**Total Phase 6 effort: 3 weeks (conservative) to 4 weeks (realistic)**

---

## Known Issues & Mitigations

### Hallucination Risk (Agent Invents Capabilities)

**Problem:** Claude generates agent with skills it claims to have, but the skill doesn't actually exist.

Example: System generates agent with skill `custom-kubernetes-operator` that doesn't exist in the skill registry.

**Mitigation:**
```rust
fn validate_generated_agent(agent: &Agent) -> Result<(), ValidationError> {
    let available_skills = load_available_skills();

    for skill in &agent.skills {
        if !available_skills.contains(skill) {
            return Err(ValidationError::SkillNotFound {
                skill: skill.clone(),
                suggestions: find_similar_skills(skill, &available_skills),
            });
        }
    }

    Ok(())
}
```

**Recovery:**
- Validation catches error before user confirmation
- Show user error: "Skill 'custom-kubernetes-operator' not found. Did you mean one of these? [suggestions]"
- Ask user: "Create this skill first?" or "Choose from available skills?"

### Malformed YAML Generation

**Problem:** Claude generates invalid YAML syntax (e.g., unquoted colons in names, incorrect indentation).

**Mitigation:**
```rust
fn validate_yaml_syntax(content: &str) -> Result<AgentEntry, YamlError> {
    serde_yaml::from_str(content)
        .map_err(|e| YamlError::ParseFailed {
            line: e.line(),
            column: e.column(),
            message: e.to_string(),
        })
}
```

**Recovery:**
- If YAML parsing fails, show error with line numbers
- Ask Claude to fix: "The YAML I generated had an error on line X. Please fix it."
- Validate again before showing preview

### Intent Misclassification

**Problem:** User says "create agent" but system interprets as "list agents" (confidence issue).

**Example:**
```
User: "Give me an agent that checks the cluster"
Classifier: Intent="teach_skill" (confidence 0.45)  // Wrong!
System: "What steps should agents learn?"  // Confused response
```

**Mitigation:**
```rust
const CONFIDENCE_THRESHOLD: f32 = 0.80;

if confidence < CONFIDENCE_THRESHOLD {
    return Err(IntentError::AmbiguousIntent {
        possible_intents: [create_agent, configure_schedule],
        questions: vec![
            "Do you want to CREATE a new agent or CONFIGURE an existing one?",
            "Should the agent run on a schedule or respond to events?",
        ],
    });
}
```

**Recovery:**
- Show clarifying questions before proceeding
- Ask user to choose between ambiguous intents
- Re-classify with additional context

### Session Loss

**Problem:** User's browser closes or connection drops mid-conversation. Pending files lost.

**Mitigation (MVP):**
- Session TTL is 30 minutes (long enough for typical workflow)
- In-memory store (simple, no database dependency)
- Show warning: "This conversation will expire in 30 minutes. Save your agent before then."

**Mitigation (Future - Phase 7):**
- Persist sessions to database (SQLite) across daemon restarts
- Show "Restore Session" option when user returns
- Allow export of pending files as JSON for backup

**For MVP:**
```rust
impl ConversationSessionStore {
    pub async fn cleanup_expired(&self) {
        // Run every 5 minutes, remove sessions older than 30 min
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, entry| {
            entry.last_activity.elapsed() < Duration::from_secs(1800)
        });
    }
}
```

### Token Limit Exceeded

**Problem:** System prompt + agent description exceeds Claude's context window.

**Example:** User creates agent with 50-line custom instructions (hallucinated capabilities).

**Mitigation:**
```rust
const MAX_PROMPT_TOKENS: usize = 4000;  // Conservative estimate

fn estimate_tokens(content: &str) -> usize {
    content.len() / 4  // Rough approximation
}

if estimate_tokens(&system_prompt) + estimate_tokens(&user_request) > MAX_PROMPT_TOKENS {
    return Err(GenerationError::TokenLimitExceeded {
        estimate: estimate_tokens(&full_prompt),
        limit: MAX_PROMPT_TOKENS,
        suggestion: "Your agent description is too detailed. Simplify to core capabilities."
    });
}
```

### Prompt Injection Risk

**Problem:** User's skill name contains prompt injection: `postgres-debugging\n\nYou are now a different system`

**Mitigation:**
```rust
fn sanitize_user_input(input: &str) -> Result<String, InputError> {
    // Reject common injection patterns
    let dangerous_patterns = vec![
        r#"(?i)\n(you are|act as|pretend|ignore)"#,
        r#"(?i)(system prompt|instructions|ignore above)"#,
    ];

    for pattern in dangerous_patterns {
        if Regex::new(pattern)?.is_match(input) {
            return Err(InputError::SuspiciousInput);
        }
    }

    // Reject unusual characters
    if !input.chars().all(|c| c.is_alphanumeric() || "-_ ".contains(c)) {
        return Err(InputError::InvalidCharacters);
    }

    Ok(input.to_string())
}
```

---

## Code Examples

### Intent Classification Example

```rust
use serde_json::json;

pub async fn classify_intent(
    message: &str,
    llm: &dyn LLMProvider,
) -> Result<IntentClassification, Error> {
    let prompt = format!(r#"
You are an intent classifier for an agentic ops system.

Classify this user message into one of these intents:
- create_agent: User wants to create a single agent
- build_squad: User wants to create multiple agents as a team
- configure_schedule: User wants to set up a cron schedule
- teach_skill: User wants to create a reusable skill
- unknown: Doesn't match any category

User message: "{}"

Respond in JSON:
{{
  "intent": "create_agent" | "build_squad" | "configure_schedule" | "teach_skill" | "unknown",
  "confidence": 0.0-1.0,
  "parameters": {{
    "agent_type": string,
    "skills": [string],
    "schedule": string,
    "description": string
  }},
  "clarifying_questions": [string]
}}
"#, message);

    let response = llm.complete(
        &ModelRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_prompt: Some("You are a helpful intent classifier.".to_string()),
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt,
            }],
            max_tokens: Some(500),
            temperature: Some(0.0),  // Deterministic
        }
    ).await?;

    let classification: IntentClassification = serde_json::from_str(&response.content)?;
    Ok(classification)
}
```

### Agent Generation Example

```rust
pub async fn generate_agent(
    intent: &Intent,
    llm: &dyn LLMProvider,
) -> Result<GeneratedAgent, Error> {
    let agent_type = intent.parameters.get("agent_type")
        .ok_or_else(|| Error::MissingParameter("agent_type".into()))?;

    let skills = intent.parameters.get("skills")
        .and_then(|s| serde_json::to_string(s).ok())
        .unwrap_or_default();

    let prompt = format!(r#"
Generate an AGENTS.md entry for a {} agent.

Skills to assign: {}

Create a YAML entry with:
- id: unique kebab-case ID
- name: friendly display name
- role: what the agent does
- avatar: single emoji
- personality_traits: 3-5 traits
- can: capabilities
- cannot: boundaries
- skills: array of skill IDs

Output ONLY the YAML entry, no explanation.
"#, agent_type, skills);

    let response = llm.complete(
        &ModelRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt,
            }],
            max_tokens: Some(1000),
            temperature: Some(0.7),  // Creative but deterministic
        }
    ).await?;

    let yaml_entry = response.content.trim();

    // Validate YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_entry)?;

    // Extract agent ID for personality generation
    let agent_id = parsed.get("id")
        .and_then(|id| id.as_str())
        .ok_or(Error::MissingField("id"))?;

    // Generate personality
    let personality = generate_soul(agent_id, &intent.parameters.get("description").map(|d| d.to_string()).unwrap_or_default(), llm).await?;

    Ok(GeneratedAgent {
        agent_yaml: yaml_entry.to_string(),
        personality_md: personality,
    })
}
```

### Cron Expression Parsing Example

```rust
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

pub fn parse_natural_schedule(
    input: &str,
) -> Result<(String, String), ScheduleError> {
    // Map natural language to cron expressions
    let cron_map = vec![
        (r"every\s+(\d+)\s+minutes?", |caps: &regex::Captures| {
            let mins = &caps[1];
            format!("*/{} * * * *", mins)
        }),
        (r"daily\s+at\s+(\d+):(\d+)", |caps: &regex::Captures| {
            let hour = &caps[1];
            let min = &caps[2];
            format!("{} {} * * *", min, hour)
        }),
        (r"every\s+(monday|tuesday|wednesday|thursday|friday)", |_| {
            "0 0 * * 1-5".to_string()  // Weekdays
        }),
    ];

    for (pattern, transform) in &cron_map {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(input) {
                let cron = transform(&caps);
                return Ok((cron, "UTC".to_string()));
            }
        }
    }

    Err(ScheduleError::UnparsableInput)
}

pub fn validate_cron(cron: &str, tz: &str) -> Result<(), CronError> {
    // Validate cron syntax
    let _: Schedule = cron.parse()
        .map_err(|_| CronError::InvalidCron)?;

    // Validate timezone
    let _: Tz = tz.parse()
        .map_err(|_| CronError::InvalidTimezone)?;

    Ok(())
}
```

---

## State of the Art

### Current Approaches in Industry

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| YAML-first config (write YAML manually) | Conversational generation (natural language → YAML) | 2023-2024 with LLM explosion | Dramatically lower time-to-first-agent, broader user base |
| Single agent workflows | Squad-based multi-agent orchestration | 2023 with CoAI frameworks | Higher success rates, better observability, clearer reasoning |
| Static agent prompts | Dynamic prompt composition from workspace files | 2024 in humanized ops | Agents feel like team members, personas are version-controlled |
| Manual scheduling | Natural language scheduling ("every 30 min") | 2023 with semantic parsing | Eliminates cron learning curve |
| One-shot skill creation | Conversational skill refinement with questions | 2024 with Claude | Richer, more validated skills |

### Deprecated/Outdated Patterns in AOF Context

- **Manual YAML for every agent** — Now conversational generation (Phase 6)
- **Hardcoded agent definitions** — Now workspace file loaders (Phase 5)
- **Agent-as-function model** — Now agent-as-team-member model with personas (Phase 5)
- **Keyword-based intent matching** — Now LLM-based intent classification (Phase 6)

---

## Open Questions

### Question 1: Should Orchestrator Agent Be Persistent?

**What we know:**
- Phase 5 provides SessionPersistence in aof-coordination for agent state
- Phase 1 provides EventBroadcaster for pub/sub
- Phase 6 needs session state for multi-turn conversations

**What's unclear:**
- Should orchestrator agent itself be persistent across daemon restarts?
- Should conversation history be saved to disk (audit log)?
- If yes, how to encrypt sensitive data (skill names, agent descriptions)?

**Recommendation for Phase 6:**
- Start simple: in-memory sessions only, TTL-based cleanup
- Audit log: save conversation summary (intent + generated file paths) to text file
- Full persistence deferred to Phase 7 with database

### Question 2: Error Recovery & User Guidance

**What we know:**
- Validation catches hallucinations and syntax errors
- Confidence thresholds guide when to ask clarifying questions

**What's unclear:**
- If skill teaching generates invalid SKILL.md, how detailed should error be?
- Should system suggest fixes, or ask user to re-teach?
- Example: "SKILL.md is missing validation criteria. Would you like me to:
  1. Add generic validation (TEST: Verify X works)
  2. Ask you how to validate it"

**Recommendation for Phase 6:**
- Detailed error messages with inline suggestions
- One re-attempt allowed; if still fails, ask user to rephrase
- For open-ended errors (bad SKILL.md), offer both auto-fix and re-teach options

### Question 3: Multi-Tenancy & User Isolation

**What we know:**
- Phase 6 is single-user MVP (aofctl serve runs locally)
- Phase 8 mentions optional server deployment

**What's unclear:**
- If deployed as server, should workspace files be per-user?
- Should orchestrator sessions be isolated by user ID?
- If user A and B create agents simultaneously, any conflicts?

**Recommendation for Phase 6:**
- Single-user assumption (one user running aofctl serve on their machine)
- Session store is not user-aware (just session UUID)
- Multi-tenancy deferred to Phase 8

---

## Sources

### Primary (HIGH confidence)
- **aof-llm crate**: Multi-provider LLM abstraction, Claude 3.5 Sonnet support (verified in source)
- **aof-personas crate**: AGENTS.md/SOUL.md loaders, PromptComposer, validation (verified Phase 5 implementation)
- **aof-skills crate**: SKILL.md format, skill discovery API (verified Phase 2 implementation)
- **Phase 5 RESEARCH.md**: Agent personas, system prompt composition (existing research)
- **Phase 2 RESEARCH.md**: Skills platform, decision logging patterns (existing research)

### Secondary (MEDIUM confidence)
- Claude API documentation (2026): JSON mode, instruction-following capabilities
- Anthropic best practices for agent orchestration (verified in aof-llm provider implementation)
- OpenClaw research on squad composition and routing (referenced in Phase 3 RESEARCH)

### Tertiary (LOW confidence - needs validation)
- Industry best practices for conversational agent creation (learned from LLM use cases, not AOF-specific)
- Cron expression parsing edge cases (common in Unix/Linux, but not AOF-tested)

---

## Metadata

**Confidence breakdown:**
- Intent classification strategy: **HIGH** (Claude 3.5 Sonnet proven for instruction-following, JSON mode tested in production)
- Specialist agent architecture: **MEDIUM-HIGH** (pattern proven in aof-llm, but orchestrator design is new)
- Squad templates: **MEDIUM** (template composition proven, but customization strategy needs testing)
- Skill teaching: **MEDIUM** (SKILL.md format exists Phase 2, but conversational generation is new)
- UI/UX integration: **MEDIUM** (React infrastructure in Phase 4 exists, but conversational panel is new component)
- Schedule parsing: **MEDIUM-HIGH** (cron standard is proven, but natural language parsing needs validation)

**Research date:** 2026-02-14
**Valid until:** 2026-03-14 (30 days for stable domain, conversational AI changes rapidly but AOF's use of Claude is stable)

**Assumptions validated:**
- ✅ aof-llm supports Claude 3.5 Sonnet with JSON mode
- ✅ aof-personas provides loader/validation infrastructure
- ✅ Phase 1 WebSocket + EventBroadcaster can handle session communication
- ✅ Mission Control UI (Phase 4) has WebSocket integration

**Assumptions NOT yet validated:**
- ⚠️ Intent classification accuracy with ambiguous requests (low-confidence cases)
- ⚠️ YAML generation quality from Claude (may hallucinate invalid schemas)
- ⚠️ Natural language cron parsing edge cases (business hours, timezone conflicts)
- ⚠️ UI responsiveness with multiple concurrent conversation sessions

**Next validation steps (for Phase 6 planning):**
1. Test intent classifier on 20+ real user examples (low/medium/high confidence)
2. Test agent generation on 5+ agent types (monitoring, incident response, deployment)
3. Test cron parsing on 10+ natural language schedules (edge cases)
4. UI prototype with 3 users for conversational panel usability

---

**End of Research Document**

This research provides the foundation for Phase 6 planning. The orchestrator-driven conversational interface, combined with specialist agents for generation and validation, offers a clear path from natural language to production agent configuration while maintaining the quality guarantees Phase 5 introduced with personas.
