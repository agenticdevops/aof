# Skills Platform Architecture

## Overview

Skills are the operational capability modules that agents use to perform work. Each skill is a SKILL.md file following the agentskills.io standard, containing markdown instructions with YAML frontmatter defining metadata and requirements.

**Key Purpose:** Enable agents to discover and execute operational capabilities with validated requirements gating.

## Architecture

### Skill Format (agentskills.io Standard)

Located in: `skills/*/SKILL.md`

Each skill is a directory containing at minimum a SKILL.md file:

```
skills/
├── k8s-debug/
│   └── SKILL.md
├── prometheus-query/
│   └── SKILL.md
└── incident-diagnose/
    └── SKILL.md
```

### SKILL.md Structure

**Frontmatter (YAML):**
```yaml
---
name: k8s-debug
description: "Kubernetes pod debugging and troubleshooting"
homepage: "https://docs.aof.sh/skills/k8s-debug"
metadata:
  emoji: "🐳"
  version: "1.0.0"
  requires:
    bins: ["kubectl", "jq"]      # Required binaries
    env: []                       # Required env vars
    config: ["~/.kube/config"]   # Required config files
  tags: ["kubernetes", "debugging"]
  author: "AOF Team"
  license: "Apache 2.0"
---
```

**Markdown Content:**
```markdown
# Kubernetes Debug Skill

Expert guidance for debugging Kubernetes workloads...

## When to Use This Skill

- Pod is in CrashLoopBackOff
- Need to debug application behavior
- ...

## Skills & Capabilities

- Retrieve pod status
- Analyze error patterns
- ...

## Steps

1. **Get pod status** — kubectl get pod {name} -o wide
2. **Check events** — kubectl describe pod {name}
3. ...
```

## Components

### SkillRegistry (aof-skills)

Located in: `crates/aof-skills/src/registry.rs`

**Responsibilities:**
- Load skills from multiple sources (workspace, bundled, enterprise)
- Cache loaded skills in memory
- Provide skill search and matching
- Check requirements before offering skills
- Hot-reload skills on file changes

**Key Methods:**
```rust
pub async fn load(&self) -> Result<()>
pub async fn get(&self, name: &str) -> Option<Skill>
pub async fn eligible(&self) -> Vec<Skill>
pub async fn match_skills(&self, intent: &str) -> Vec<Skill>
pub async fn search(&self, query: &str) -> Vec<SkillSearchResult>
pub async fn check_skill(&self, name: &str) -> Result<RequirementCheck>
```

### AgentSkillsValidator (aof-skills)

Located in: `crates/aof-skills/src/registry.rs`

**Validation Methods:**

```rust
pub fn validate_frontmatter(&self, skill: &Skill) -> ValidationReport
pub fn validate_markdown(&self, skill: &Skill) -> ValidationReport
pub fn validate_claude_compatibility(&self, skill: &Skill) -> bool
```

**Checks:**
- Required fields: name, description
- Metadata structure: emoji, version, requires
- Tags for searchability
- Markdown sections: "When to Use", "Steps"

### RequirementChecker (aof-skills)

Located in: `crates/aof-skills/src/requirements.rs`

**Capabilities:**
- Check binary availability (PATH)
- Verify environment variables
- Confirm config file existence
- OS compatibility checking
- Graceful degradation (partial eligibility)

## Integration Points

### 1. Skill Discovery

Location: `crates/aof-skills/src/loader.rs`

Skills are discovered by scanning filesystem:
- `~/.aof/skills/` (workspace, highest precedence)
- `/usr/local/share/aof/skills/` (bundled)
- Enterprise registry (future)

### 2. Progressive Disclosure

**match_skills() Method:**
```rust
let matched = registry.match_skills("debug pod").await;
// Returns: [k8s-debug, k8s-logs, incident-diagnose, ...]
```

Matching algorithm:
1. Search skill name, description, tags against intent
2. Score each match (0.0-1.0)
3. Filter by threshold (0.5)
4. Return sorted by relevance

### 3. Requirements Gating

**Before Offering Skill:**
```rust
let check = registry.check_skill("k8s-debug").await?;
if !check.eligible {
    println!("kubectl not found. Install: brew install kubectl");
}
```

**Requirements Enforcement:**
- If binary missing: skill marked unavailable
- If env var missing: skill marked unavailable
- If config missing: skill marked unavailable
- Installation suggestions provided

### 4. Hot-Reload

Location: `crates/aof-skills/src/watcher.rs`

File watcher detects changes to SKILL.md:
- Parses updated skill
- Re-validates frontmatter
- Updates in-memory cache
- No daemon restart needed

**Trigger:** File save
**Latency:** <1 second

## Bundled Skills

Location: `skills/*/SKILL.md`

**13 Core Skills (Phase 2):**
1. **k8s-debug** — Pod troubleshooting (kubectl, jq)
2. **k8s-logs** — Log retrieval (kubectl, grep)
3. **prometheus-query** — Metric queries (curl, jq)
4. **loki-search** — Log search (curl, jq)
5. **git-operations** — Git commands (git)
6. **docker-operations** — Container management (docker)
7. **shell-execute** — Shell scripting (bash, sh)
8. **http-testing** — API testing (curl, jq)
9. **incident-diagnose** — Multi-source analysis (kubectl, curl, jq)
10. **argocd-deploy** — ArgoCD operations (argocd, kubectl)
11. **database-debug** — Database debugging (psql/mysql)
12. **network-debug** — Network troubleshooting (netstat, curl)
13. **incident-postmortem** — Postmortem generation (jq)

## Usage Example

```rust
// 1. Create registry
let registry = SkillRegistry::default_registry();

// 2. Load skills
registry.load().await?;

// 3. Match by intent (progressive disclosure)
let matched = registry.match_skills("debug pod crashes").await;

// 4. Check requirements
for skill in &matched {
    let check = registry.check_skill(&skill.name).await?;
    if check.eligible {
        println!("Available: {}", skill.name);
    } else {
        println!("Need: {}", check.missing_requirements.join(", "));
    }
}

// 5. Get skill for LLM consumption
if let Some(skill) = registry.get("k8s-debug").await {
    let prompt = aof_skills::build_skills_prompt(&[skill]);
}
```

## Configuration

### Environment Variables

```bash
AOF_SKILLS_WORKSPACE_DIR=/home/user/my-skills   # Extra skill directory
AOF_SKILLS_ENTERPRISE_URL=...                    # Enterprise registry URL
```

### YAML Config (Future)

```yaml
spec:
  skills:
    workspace_dir: /home/user/my-skills
    bundled_dirs:
      - /usr/local/share/aof/skills
    enable_hot_reload: true
    cache_ttl_secs: 300
```

## Adding a New Skill

1. Create directory:
   ```bash
   mkdir -p skills/my-skill/
   ```

2. Create SKILL.md with frontmatter:
   ```yaml
   ---
   name: my-skill
   description: "..."
   metadata:
     requires:
       bins: ["tool1", "tool2"]
     tags: ["category"]
   ---
   
   # My Skill
   
   Instructions...
   ```

3. Validate:
   ```bash
   cargo test --lib skill_loading
   ```

4. Commit to git (hot-reload picks it up)

## Testing Skills

### Unit Tests

Located in: `crates/aof-skills/src/registry.rs` (tests module)

```bash
cargo test --package aof-skills --lib
```

### Claude Compatibility Check

Manually verify skill parses as Claude tool:
```rust
let validator = AgentSkillsValidator::new();
assert!(validator.validate_claude_compatibility(&skill));
```

### Requirement Verification

Test that requirements checking works:
```bash
# Missing kubectl
AOF_PATH=/nonexistent cargo test --lib requirements

# With kubectl available
which kubectl && cargo test --lib requirements
```

## Performance Characteristics

### Loading
- Initial load: 50-100ms (50 skills)
- Hot-reload: <1s per file
- Memory: ~5MB per 100 skills

### Matching
- match_skills(): 5-10ms (50 skills, simple keyword matching)
- With embeddings (future): 50-100ms per query

### Requirements Check
- Binary check: 1-5ms (PATH scan)
- Config file check: <1ms (file exists)
- Parallelized across skills

## Future Enhancements

### Phase 3+
- Skill versioning and pinning per agent
- Semantic skill matching with embeddings
- Skill marketplace and central registry
- Version compatibility matrix

### Phase 8 (Production)
- Enterprise skill repository integration
- RBAC-based skill access control
- Skill usage analytics and recommendations
- Automatic skill dependency resolution
