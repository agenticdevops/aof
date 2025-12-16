# Contributing to AOF

Thank you for your interest in contributing to AOF (Agentic Ops Framework)! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and constructive in discussions
- Focus on the technical merits of ideas
- Help others learn and grow
- Report any unacceptable behavior to maintainers

---

## Getting Started

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs))
- Git
- Docker (optional, for integration tests)
- kubectl (optional, for K8s tool testing)

### Fork and Clone

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/YOUR_USERNAME/aof.git
cd aof
git remote add upstream https://github.com/agenticdevops/aof.git
```

### Build and Test

```bash
# Build
cargo build --release

# Run tests
cargo test --lib

# Quick validation
./scripts/test-pre-compile.sh
```

---

## Development Setup

### IDE Setup

**VS Code** (recommended):
```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

**IntelliJ IDEA / CLion**:
- Install Rust plugin
- Enable "cargo clippy" for code analysis

### Environment Variables

For testing with real providers:

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export GOOGLE_API_KEY=...
```

---

## Making Changes

### Branching Strategy

```
main        → Production releases
  └── dev   → Development branch (PR target)
       └── feat/feature-name    → Feature branches
       └── fix/bug-description  → Bug fixes
       └── docs/doc-changes     → Documentation
```

### Creating a Branch

```bash
# Update from upstream
git fetch upstream
git checkout dev
git merge upstream/dev

# Create feature branch
git checkout -b feat/my-feature
```

### Commit Messages

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Maintenance

**Examples:**
```
feat(tools): Add AWS Lambda tool support

- Implement aws_lambda tool for function invocation
- Add list-functions and get-function operations
- Include comprehensive tests

Closes #123
```

```
fix(mcp): Handle connection timeout gracefully

Previously, MCP connections would hang indefinitely.
Now properly enforces timeout and returns error.
```

---

## Pull Request Process

### Before Submitting

1. **Update from upstream:**
   ```bash
   git fetch upstream
   git rebase upstream/dev
   ```

2. **Run tests:**
   ```bash
   cargo test --lib
   cargo clippy --all-targets
   cargo fmt --check
   ```

3. **Update documentation** if needed

4. **Add tests** for new functionality

### PR Template

```markdown
## Summary
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed
- [ ] Documentation updated
- [ ] No new warnings
```

### Review Process

1. Create PR against `dev` branch
2. Automated CI runs tests
3. Maintainer reviews code
4. Address feedback
5. Squash and merge

---

## Code Style

### Rust Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// Good: Clear naming, proper documentation
/// Executes a kubectl command and returns the result.
///
/// # Arguments
/// * `resource` - Kubernetes resource type
/// * `namespace` - Optional namespace
///
/// # Returns
/// Command output on success, error message on failure
pub async fn kubectl_get(
    resource: &str,
    namespace: Option<&str>,
) -> AofResult<String> {
    // Implementation
}

// Good: Use Result types for fallible operations
pub fn parse_config(yaml: &str) -> AofResult<AgentConfig> {
    serde_yaml::from_str(yaml).map_err(|e| {
        AofError::config(format!("Invalid YAML: {}", e))
    })
}

// Good: Prefer Option over sentinel values
pub struct ToolConfig {
    pub timeout_secs: Option<u64>,  // None = use default
}
```

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Address warnings
cargo clippy --fix
```

---

## Testing

### Unit Tests

Place in same file as implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
            name: test
            model: openai:gpt-4
        "#;
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.name, "test");
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let tool = MyTool::new();
        let input = ToolInput::new(json!({"param": "value"}));
        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_test.rs
use aof_runtime::Runtime;

#[tokio::test]
async fn test_agent_execution() {
    let mut runtime = Runtime::new();
    runtime.load_agent_from_file("tests/fixtures/test-agent.yaml")
        .await
        .unwrap();

    let result = runtime.execute("test-agent", "hello").await;
    assert!(result.is_ok());
}
```

### Test Commands

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Specific crate
cargo test -p aof-tools

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

---

## Documentation

### Code Documentation

```rust
/// Brief one-line description.
///
/// Longer description with details about behavior,
/// edge cases, and usage patterns.
///
/// # Arguments
///
/// * `name` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// * `AofError::Config` - When configuration is invalid
///
/// # Examples
///
/// ```rust
/// let result = function_name("arg");
/// assert!(result.is_ok());
/// ```
pub fn function_name(name: &str) -> AofResult<()> {
    // ...
}
```

### User Documentation

Place in `docs/user/`:
- `CLI_REFERENCE.md` - Command-line usage
- `tools/index.md` - Tool reference
- `FEATURES.md` - Feature overview
- `MCP_CONFIGURATION.md` - MCP setup

### Developer Documentation

Place in `docs/dev/`:
- `ARCHITECTURE.md` - System design
- `CONTRIBUTING.md` - This guide
- `TOOLS_DEVELOPMENT.md` - Tool development
- `AGENTFLOW_DESIGN.md` - AgentFleet design

---

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** - Breaking changes
- **MINOR** - New features (backward compatible)
- **PATCH** - Bug fixes

### Release Checklist

1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create release branch from `dev`
4. Final testing
5. Merge to `main`
6. Tag release: `git tag v0.2.0`
7. Push tag: `git push origin v0.2.0`
8. Publish to crates.io

---

## Areas for Contribution

### Good First Issues

Look for issues labeled `good-first-issue`:
- Documentation improvements
- Test coverage
- Small bug fixes

### Feature Development

- New tool implementations
- LLM provider support
- Memory backends
- Trigger platforms

### Testing

- Increase test coverage
- Integration tests
- Performance benchmarks

### Documentation

- User guides
- API documentation
- Example agents

---

## Getting Help

- **GitHub Issues** - Bug reports, feature requests
- **Discussions** - Questions, ideas
- **Discord** - Real-time chat (coming soon)

---

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md`
- Release notes
- Project README

Thank you for contributing to AOF!
