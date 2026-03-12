# AOF - Agentic Ops Framework

## OpsFlow Ecosystem Context

AOF is an **Apache 2.0 open source** Rust framework for building agentic applications.

**Repository**: https://github.com/agenticdevops/aof
**License**: Apache 2.0
**Type**: Pure Rust library crates + CLI (NO desktop/Tauri)

### Ecosystem Structure
```
/Users/gshah/work/opsflow-sh/
├── aof/          # THIS REPO - Open source framework
├── kubepilot/    # Closed source K8s desktop (imports AOF crates)
└── opspilot/     # Closed source enterprise (imports AOF crates)
```

### AOF Crates (Pure Rust - No Tauri)
- `aof-core` - Core traits, types, interfaces
- `aof-llm` - LLM provider abstraction
- `aof-mcp` - MCP client
- `aof-runtime` - Agent execution
- `aof-memory` - State management
- `aof-triggers` - Event triggers
- `aofctl` - CLI binary

### Cross-Repo Integration
KubePilot and OpsPilot import AOF crates:
```toml
aof-core = { path = "../../aof/aof/crates/aof-core" }
aof-llm = { path = "../../aof/aof/crates/aof-llm" }
```

## Release Process

**Documentation**: https://docs.aof.sh
**Installation**: `curl -sSL https://docs.aof.sh/install.sh | bash`

### Creating a Release (Automated)

The release process is fully automated via GitHub Actions. **DO NOT create releases manually.**

```bash
# 1. Create and push a version tag (triggers automated build)
git tag -a v0.1.14 -m "Release v0.1.14: Brief description"
git push origin v0.1.14

# 2. Monitor: https://github.com/agenticdevops/aof/actions
# 3. Verify: https://github.com/agenticdevops/aof/releases
```

The workflow will automatically:
- Build binaries for Linux, macOS (Intel & Apple Silicon), Windows
- Calculate SHA256 checksums
- Create GitHub Release with formatted release notes
- Include installation instructions

### Release Notes Format (Auto-generated)

The workflow creates consistent release notes with:
- Installation instructions (curl | bash)
- Manual download links
- Checksum verification commands
- Getting started guide

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

# Static analysis
cargo clippy --all-targets

# End-to-end validation
./scripts/test-agent.sh

# Frontend dev server
cd web-app && pnpm dev

# Start backend
./target/release/aofctl serve --config quickstart/serve-config.yaml
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
- `web-app/` - React frontend
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
- Strictly follow kubectl style implementation for aofctl. For example use "aofctl run agent" instead of "aofctl agent run". If you find anything non compliant, correct it.
- for every feature added, add/update  docs/ so that we are keeping track of every single feautre the product has and how it works.
- When you make changes, first update the internal docs, then implement, verify the impleemntation matches the docs, rhen also update the user docs with concepts, examples, resource spec, tutorials etc.