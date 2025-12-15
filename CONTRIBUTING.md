# Contributing to Agentic Ops Framework

Thank you for your interest in contributing to AOF! We welcome contributions from everyone. This document provides guidelines and instructions for contributing.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and constructive in all interactions.

## Ways to Contribute

- **Report Bugs**: Use GitHub Issues to report bugs with clear descriptions
- **Suggest Features**: Share your ideas for new features via GitHub Discussions
- **Improve Documentation**: Fix typos, add examples, or clarify existing docs
- **Submit Code**: Fix bugs, add features, or improve performance
- **Testing**: Help test new features and report edge cases

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo
- Git
- Make (optional, for convenience)

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/aof.git
   cd aof
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/agenticopsorg/aof.git
   ```

4. Create a development branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. Build the project:
   ```bash
   cargo build
   ```

6. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before You Start

- Check if an issue exists for what you want to work on
- Discuss significant changes in an issue or discussion first
- Keep changes focused and manageable in scope

### While You Work

- Write tests for new functionality
- Follow Rust conventions and use `cargo fmt`
- Keep commits atomic and with clear messages
- Reference related issues in commit messages

### Before You Submit

1. Update documentation if you changed functionality
2. Run tests: `cargo test`
3. Check formatting: `cargo fmt`
4. Lint your code: `cargo clippy`
5. Update CHANGELOG if applicable

## Submitting Changes

### Pull Request Process

1. Push your branch to your fork
2. Create a Pull Request against the `dev` branch (not main)
3. Provide a clear description of changes
4. Reference related issues
5. Ensure CI checks pass
6. Request review from maintainers

### Pull Request Guidelines

- **Title**: Clear, descriptive, follows conventional commits
  - Examples: `feat: add interactive REPL mode`, `fix: correct binary naming`
- **Description**: Explain what changed and why
- **Testing**: Show how you tested the changes
- **Documentation**: Update docs if behavior changed

## Conventional Commits

We follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(cli): add interactive REPL mode`
- `fix(install): correct binary download URL`
- `docs: update getting started guide`

## Code Style

- Run `cargo fmt` to format code
- Run `cargo clippy` to check for common mistakes
- Follow Rust naming conventions
- Add comments for complex logic
- Keep functions focused and modular

## Testing

- Write tests for new features
- Ensure existing tests pass
- Test edge cases
- Include integration tests where appropriate

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Documentation

- Update relevant `.md` files in `docusaurus-site/docs/`
- Add examples where helpful
- Keep documentation up-to-date with code changes
- Use clear, accessible language

## Release Process

Releases follow [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

The maintainers handle the release process, including version bumping and publishing.

## Getting Help

- **Questions**: Ask in GitHub Discussions
- **Bug Reports**: Use GitHub Issues with details
- **Chat**: Join our community discussions
- **Documentation**: Check the docs at https://aof.sh

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

## Questions or Need Help?

Feel free to reach out:
- Open an issue with questions
- Start a discussion in GitHub Discussions
- Check existing documentation

Thank you for contributing! 🎉
