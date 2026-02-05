---
sidebar_position: 3
title: Skill Reference
description: Complete specification for SKILL.md files
---

# Skill Reference

Complete specification for the `SKILL.md` format and the skills platform.

## File Format

### Location

Skills can be placed in any of these locations:

| Location | Precedence | Purpose |
|----------|------------|---------|
| `.claude/skills/<name>/SKILL.md` | Highest | Project-specific skills |
| `~/.aof/skills/<name>/SKILL.md` | High | User-wide skills |
| `<aof-dir>/skills/<name>/SKILL.md` | Lowest | Bundled skills |

### Structure

```
skill-name/
├── SKILL.md          # Required: Main skill definition
└── references/       # Optional: Additional documentation
    ├── examples.md
    └── troubleshooting.md
```

## Frontmatter Schema

### Complete Schema

```yaml
# Required fields
name: string          # Unique skill identifier
description: string   # Brief description (< 100 chars recommended)

# Optional fields
homepage: string      # URL to additional documentation

metadata:
  # Display
  emoji: string       # Single emoji for display

  # Versioning
  version: string     # Semantic version (e.g., "1.0.0")
  author: string      # Author name or team
  license: string     # License identifier

  # Requirements (all checked at load time)
  requires:
    bins: string[]    # Binaries that must be in PATH
    any_bins: string[] # At least one must be available
    env: string[]     # Environment variables that must be set
    config: string[]  # Config paths that must exist (~ expanded)

  # OS restriction
  os: string[]        # Allowed operating systems: darwin, linux, windows

  # Behavior
  always: boolean     # If true, always load regardless of requirements

  # Installation help
  install:
    - id: string      # Unique identifier for this installer
      kind: string    # brew, apt, dnf, npm, pip, cargo, manual
      package: string # Package name
      bins: string[]  # Binaries provided by this package
      url: string     # URL for manual installation

  # Categorization
  tags: string[]      # Tags for search and filtering
```

### Field Details

#### name
- **Type**: string (required)
- **Format**: kebab-case recommended
- **Examples**: `k8s-debug`, `prometheus-query`, `incident-diagnose`

#### description
- **Type**: string (required)
- **Best practice**: Keep under 100 characters
- **Examples**:
  - `"Kubernetes pod debugging and troubleshooting"`
  - `"PromQL queries for common monitoring scenarios"`

#### metadata.requires.bins
- **Type**: string[]
- **Behavior**: ALL listed binaries must be found in PATH
- **Check method**: Uses `which` command
- **Example**: `["kubectl", "helm"]` requires both kubectl AND helm

#### metadata.requires.any_bins
- **Type**: string[]
- **Behavior**: At least ONE listed binary must be found
- **Use case**: Alternative tools (e.g., docker OR podman)
- **Example**: `["docker", "podman"]` requires docker OR podman

#### metadata.requires.env
- **Type**: string[]
- **Behavior**: ALL listed environment variables must be set
- **Example**: `["KUBECONFIG", "AWS_PROFILE"]`

#### metadata.requires.config
- **Type**: string[]
- **Behavior**: ALL listed paths must exist
- **Path expansion**: `~` is expanded to home directory
- **Example**: `["~/.kube/config", "~/.aws/credentials"]`

#### metadata.os
- **Type**: string[]
- **Values**: `darwin`, `linux`, `windows`
- **Behavior**: Skill only eligible on listed operating systems
- **Example**: `["darwin", "linux"]` excludes Windows

#### metadata.always
- **Type**: boolean
- **Default**: false
- **Behavior**: When true, skill is always loaded regardless of requirements
- **Use case**: Skills with optional features or documentation-only skills

#### metadata.install
- **Type**: array of install specs
- **Purpose**: Help users install missing dependencies

Install spec fields:
| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier |
| `kind` | Yes | Installer type |
| `package` | Yes | Package name to install |
| `bins` | No | Binaries provided |
| `url` | No | Manual install URL |

Supported `kind` values:
| Kind | Description | Example |
|------|-------------|---------|
| `brew` | Homebrew (macOS/Linux) | `brew install kubectl` |
| `apt` | APT (Debian/Ubuntu) | `apt-get install kubectl` |
| `dnf` | DNF (Fedora/RHEL) | `dnf install kubectl` |
| `npm` | Node.js | `npm install -g tool` |
| `pip` | Python | `pip install tool` |
| `cargo` | Rust | `cargo install tool` |
| `manual` | Manual with URL | User visits URL |

## Content Guidelines

### Recommended Sections

```markdown
# Skill Name

## When to Use This Skill
[Scenarios where this skill applies]

## Quick Start
[Most common operation, copy-paste ready]

## Prerequisites
[What's needed beyond the metadata requirements]

## Common Operations
[Step-by-step guides for typical tasks]

## Troubleshooting
[Common issues and solutions]

## Reference
[Commands table, links, additional resources]
```

### Markdown Features

Skills support full GitHub-flavored markdown:

- Headers (H1-H6)
- Code blocks with language hints
- Tables
- Lists (ordered and unordered)
- Links
- Blockquotes
- Bold/italic/code spans

### Code Block Best Practices

Always include language hint:
````markdown
```bash
kubectl get pods
```

```yaml
apiVersion: v1
kind: Pod
```

```json
{"key": "value"}
```
````

## CLI Reference

### aofctl skills list

List all loaded skills.

```bash
aofctl skills list [OPTIONS]

Options:
  -o, --output <FORMAT>   Output format: table, json, yaml, wide, name
      --eligible          Show only eligible skills
      --skills-dir <DIR>  Skills directory to load from
```

### aofctl skills check

Check if a skill's requirements are met.

```bash
aofctl skills check <NAME> [OPTIONS]

Options:
      --skills-dir <DIR>  Skills directory
```

### aofctl skills show

Display skill content.

```bash
aofctl skills show <NAME> [OPTIONS]

Options:
      --skills-dir <DIR>  Skills directory
```

### aofctl skills search

Search skills by query.

```bash
aofctl skills search <QUERY> [OPTIONS]

Options:
      --skills-dir <DIR>  Skills directory
```

### aofctl skills prompt

Generate prompt injection for skills.

```bash
aofctl skills prompt [SKILLS] [OPTIONS]

Arguments:
  SKILLS  Skill names (comma-separated) or 'all' for eligible skills

Options:
      --skills-dir <DIR>  Skills directory
```

## API Reference

### Rust API

```rust
use aof_skills::{SkillRegistry, SkillConfig, build_skills_prompt};

// Create registry
let config = SkillConfig {
    workspace_dir: Some(PathBuf::from(".claude/skills")),
    bundled_dirs: vec![PathBuf::from("skills")],
    ..Default::default()
};
let registry = SkillRegistry::new(config);

// Load skills
registry.load().await?;

// Get eligible skills
let skills = registry.eligible().await;

// Build prompt
let prompt = build_skills_prompt(&skills);
```

### Key Types

```rust
// Skill definition
pub struct Skill {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub content: String,
    pub metadata: SkillMetadata,
    pub source: SkillSource,
}

// Metadata
pub struct SkillMetadata {
    pub emoji: Option<String>,
    pub requires: SkillRequirements,
    pub install: Vec<InstallSpec>,
    pub os: Option<Vec<String>>,
    pub always: bool,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub author: Option<String>,
}

// Requirements
pub struct SkillRequirements {
    pub bins: Vec<String>,
    pub any_bins: Vec<String>,
    pub env: Vec<String>,
    pub config: Vec<String>,
}

// Source precedence
pub enum SkillSource {
    Bundled,           // Precedence: 0
    PublicRegistry,    // Precedence: 1
    EnterpriseRegistry,// Precedence: 2
    Workspace,         // Precedence: 3 (highest)
}
```

## Prompt Injection Format

Skills are injected into agent prompts as XML:

```xml
<available-skills>
<skill name="k8s-debug">
<description>Kubernetes pod debugging and troubleshooting</description>
<tags>kubernetes, debugging</tags>
<instructions>
# Kubernetes Debug Skill
...
</instructions>
</skill>
</available-skills>
```

## Hot Reload

Skills support hot-reload via file watching:

1. Edit any `SKILL.md` file
2. Changes are detected automatically
3. Skills are reloaded without restart

Enable in configuration:
```rust
let mut config = SkillConfig::default();
config.watch = true;
```
