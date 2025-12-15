# Documentation Guide - Single Source of Truth

## Overview

This document explains the documentation structure for the Agentic Ops Framework (AOF) and how to maintain a **single source of truth**.

## Documentation Structure

```
my-framework/
├── docs/                          # Internal documentation (architecture, research, etc.)
│   ├── KUBECTL_REFACTOR_COMPLETE.md
│   ├── architecture/
│   ├── research/
│   └── ... (project-level docs)
│
└── docusaurus-site/
    ├── docs/                      # User-facing documentation (SINGLE SOURCE OF TRUTH)
    │   ├── getting-started.md
    │   ├── concepts.md
    │   ├── tutorials/
    │   │   ├── first-agent.md
    │   │   ├── incident-response.md
    │   │   └── slack-bot.md
    │   ├── examples/
    │   │   └── index.md
    │   ├── reference/
    │   │   ├── aofctl-complete.md
    │   │   ├── api-resources.md
    │   │   ├── agent-spec.md
    │   │   └── agentflow-spec.md
    │   └── guides/
    │       ├── migration-guide.md
    │       └── kubernetes-compatibility.md
    └── DOCUMENTATION_GUIDE.md      # This file

```

## Key Principles

### 1. Single Source of Truth
- **All user-facing documentation lives in `/docusaurus-site/docs/`**
- This is the only place where tutorials, guides, and examples are maintained
- No duplicate documentation in `/docs/` folder

### 2. Documentation Organization
- **User Guides**: How-to documentation for users
  - `getting-started.md` - Installation and setup
  - `concepts.md` - Understanding agents, fleets, and flows
  - `tutorials/` - Step-by-step guides
  - `examples/` - Copy-paste ready configurations
  - `guides/` - Specialized guides (migration, compatibility)

- **Reference Docs**: Technical reference
  - `reference/` - Complete API and command reference

- **Internal Docs**: Project-specific documentation (separate from user docs)
  - `/docs/architecture/` - Architecture decisions and design documents
  - `/docs/research/` - Research findings and analysis
  - Project status files (KUBECTL_REFACTOR_COMPLETE.md, etc.)

### 3. Automatic Build & Deploy

The GitHub Actions workflow (`/.github/workflows/deploy-docs.yml`) automatically:

1. **Triggers on changes to:**
   - Any file in `docusaurus-site/docs/`
   - Any file in `docs/` (for awareness)
   - The workflow file itself

2. **On push to `main` or `dev` branch:**
   - Installs dependencies
   - Syncs documentation (if needed)
   - Builds static site
   - Deploys to GitHub Pages

3. **Manual trigger:**
   - Can be triggered manually via GitHub Actions UI

## How to Update Documentation

### Updating User-Facing Docs

1. **Edit files in `/docusaurus-site/docs/`**
   ```bash
   # Example: Update a tutorial
   vim docusaurus-site/docs/tutorials/first-agent.md
   ```

2. **Test locally before pushing**
   ```bash
   cd docusaurus-site
   npm run start  # Starts dev server at http://localhost:3000
   ```

3. **Commit and push to dev or main**
   ```bash
   git add docusaurus-site/docs/tutorials/first-agent.md
   git commit -m "docs: Update first agent tutorial with new patterns"
   git push
   ```

4. **GitHub Actions automatically builds and deploys**
   - Monitor at: https://github.com/gshah/my-framework/actions
   - Deployed to: https://aof.sh (when configured)

### What NOT to Do

❌ **DON'T** maintain separate copies of documentation
❌ **DON'T** update `/docs/` for user-facing content
❌ **DON'T** expect manual builds - GitHub Actions handles this
❌ **DON'T** commit uncommitted changes to documentation files

## Command Pattern Reference

When documenting aofctl commands, use the **verb-noun (kubectl-compatible) pattern**:

### Correct Patterns ✅

```bash
# Running agents and workflows
aofctl run agent config.yaml
aofctl run agentflow my-flow --daemon

# Creating/updating resources
aofctl apply -f agent.yaml
aofctl apply -f agent.yaml --dry-run

# Listing resources
aofctl get agents
aofctl get agent my-agent

# Getting details
aofctl describe agent my-agent
aofctl describe agentflow my-flow

# Viewing logs
aofctl logs agent my-agent -f
aofctl logs agentflow my-flow

# Executing commands
aofctl exec agent my-agent -- "command"

# Deleting resources
aofctl delete agent my-agent
aofctl delete agentflow my-flow

# Discovering resources
aofctl api-resources
```

### Outdated Patterns ❌

```bash
# These are the OLD noun-verb pattern - DO NOT USE
aofctl agent run config.yaml
aofctl agent apply -f config.yaml
aofctl agent get
aofctl flow run my-flow
aofctl flow status my-flow
aofctl flow logs my-flow
```

## Documentation Best Practices

### 1. Clear Command Examples
- Always show the current **verb-noun** pattern
- Use code blocks with bash syntax highlighting
- Include expected output when helpful

### 2. Progressive Disclosure
- Start with simple examples
- Gradually introduce advanced features
- Use step-by-step tutorials

### 3. Link References
- Cross-reference related documentation
- Use relative links: `../reference/agent-spec.md`
- Link to migration guide for deprecated patterns

### 4. Keep Content DRY
- Don't repeat command patterns across files
- Use consistent terminology
- Reference shared concepts from `concepts.md`

## File Structure Template

When creating new documentation files:

```markdown
# Tutorial/Guide Title

Brief description of what this covers.

**What you'll learn:**
- Point 1
- Point 2

**Prerequisites:**
- Item 1
- Item 2

## Step 1: Description

Explanation and code examples.

```bash
# Current pattern only
aofctl run agent ...
```

## Step 2: Description

More content...

## Next Steps

Links to related documentation.
```

## Troubleshooting

### Documentation not updating after push?

1. Check GitHub Actions: https://github.com/gshah/my-framework/actions
2. Verify file path is in `docusaurus-site/docs/`
3. Verify changes are in commit (not staged)

### Build fails locally?

```bash
# Clean and rebuild
cd docusaurus-site
rm -rf node_modules build
npm install
npm run build
```

### How to rollback documentation?

```bash
# Revert specific file
git revert <commit-hash>

# OR revert last commit
git revert HEAD

# Push to trigger redeploy
git push
```

## Future Improvements

- [ ] Automated linting of documentation markdown
- [ ] Link validation
- [ ] Command pattern validation (ensure verb-noun only)
- [ ] Redirect old patterns to new documentation
- [ ] Search functionality across docs
- [ ] Documentation versioning

## Questions?

For documentation improvements or issues:
1. Check existing tutorials and guides
2. Review the migration guide: `guides/migration-guide.md`
3. Submit issues on GitHub: https://github.com/gshah/my-framework/issues
