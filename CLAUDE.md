# OpenAgentiX — Agentic Ops Framework

## OpsFlow Ecosystem Context

OpenAgentiX is an **Apache 2.0 open source** Rust framework for building agentic applications.

**Repository**: https://github.com/agenticdevops/aof
**License**: Apache 2.0
**Type**: Pure Rust library crates + CLI (NO desktop/Tauri)

### Ecosystem Structure
```
/Users/gshah/work/opsflow-sh/
├── aof/          # THIS REPO - Open source framework (OpenAgentiX)
├── kubepilot/    # Closed source K8s desktop (imports agentix-* crates)
└── opspilot/     # Closed source enterprise (imports agentix-* crates)
```

### Agentix Crates (Pure Rust - No Tauri)
- `agentix-core` - Core traits, types, agent definitions
- `agentix-llm` - LLM provider abstraction
- `agentix-mcp` - MCP client
- `agentix-runtime` - Agent execution + gateway
- `agentix-memory` - State management
- `agentix-triggers` - Event triggers (Slack, Telegram, etc.)
- `agentix` - CLI binary

### Cross-Repo Integration
KubePilot and OpsPilot import agentix crates:
```toml
# KubePilot/OpsPilot Cargo.toml
agentix-core = { path = "../../aof/aof/crates/agentix-core" }
agentix-llm = { path = "../../aof/aof/crates/agentix-llm" }
```

NOTE: BREAKING — Update crate names from aof-* to agentix-* in dependent repositories.

## Release Process

**Documentation**: https://docs.aof.sh
**Installation**: `cargo install agentix`

### Creating a Release (Automated)

The release process is fully automated via GitHub Actions. **DO NOT create releases manually.**

```bash
# 1. Create and push a version tag (triggers automated build)
git tag -a v2.0.0-alpha.1 -m "Release v2.0.0-alpha.1: OpenAgentiX rebrand"
git push origin v2.0.0-alpha.1

# 2. Monitor: https://github.com/agenticdevops/aof/actions
# 3. Verify: https://github.com/agenticdevops/aof/releases
```

### Version Numbering

Use semantic versioning: `vMAJOR.MINOR.PATCH`
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

**Full details**: See `RELEASE_PROCESS.md`

---

## Build Commands

```bash
# Quick syntax check
cargo check

# Unit tests only
cargo test --lib

# Full release build
cargo build --release

# Start gateway
./target/release/agentix gateway start --config quickstart/agentix.yaml

# Validate an agent directory
./target/release/agentix validate agents/my-agent

# Create a new agent scaffold
./target/release/agentix init --name my-agent
```

## Agent Format (GitAgent-Compatible)

Agents are now GitAgent-compatible directories (agent.yaml + SOUL.md + optional files).
Flat YAML format (`apiVersion: openagentix.dev/v1`) still loads for backward compatibility.

Primary format:
```
agents/my-agent/
├── agent.yaml    # Minimal manifest (name, model.preferred)
├── SOUL.md       # Agent identity + system prompt
├── RULES.md      # Behavioral rules (optional)
├── skills/       # Reusable skill modules (optional)
└── tools/        # Tool definitions (optional)
```

## Code Style & Best Practices

- **Modular Design**: Files under 500 lines
- **Environment Safety**: Never hardcode secrets
- **Clean Architecture**: Separate concerns
- **Helpful Error Messages**: Use `serde_path_to_error` for YAML/JSON parsing to show exact field paths on errors

### YAML/JSON Parsing Best Practice

Always use `serde_path_to_error` when deserializing user-provided config files:

```rust
let deserializer = serde_yaml::Deserializer::from_str(&content);
let config: Config = serde_path_to_error::deserialize(deserializer)
    .map_err(|e| anyhow!("Field: {}\nError: {}", e.path(), e.inner()))?;
```

## File Organization

**NEVER save working files to the root folder. Use these directories:**
- `crates/` - Rust source code
- `docs/` - Documentation
- `quickstart/` - Configuration files
- `scripts/` - Utility scripts
- `examples/` - Example code

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
Never save working files, text/mds and tests to the root folder.
- Strictly follow kubectl style implementation for agentix CLI. For example use "agentix run agent" instead of "agentix agent run". If you find anything non compliant, correct it.
- for every feature added, add/update docs/ so that we are keeping track of every single feature the product has and how it works.
- When you make changes, first update the internal docs, then implement, verify the implementation matches the docs, then also update the user docs with concepts, examples, resource spec, tutorials etc.
