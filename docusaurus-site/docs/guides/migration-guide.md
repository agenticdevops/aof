---
sidebar_position: 5
---

# Migration Guide: aofctl Kubectl-Style Refactoring

This guide helps you migrate from the old noun-verb command pattern to the new **kubectl-compatible verb-noun pattern**.

## What's Changing?

aofctl is being refactored to match Kubernetes CLI conventions for improved consistency and usability.

### Old Pattern (Deprecated)
```bash
aofctl <noun> <verb> [options]
aofctl agent run config.yaml
aofctl agent get
aofctl workflow list
```

### New Pattern (Current)
```bash
aofctl <verb> <noun> [name] [options]
aofctl run agent config.yaml
aofctl get agents
aofctl get workflows
```

---

## Command Mapping Reference

Quick lookup table for converting your existing commands:

### Agent Commands

| Old Command | New Command | Description |
|------------|-------------|-------------|
| `aofctl agent run config.yaml --input "q"` | `aofctl run agent config.yaml --input "q"` | Execute an agent |
| `aofctl agent get` | `aofctl get agents` | List all agents |
| `aofctl agent get my-agent` | `aofctl get agent my-agent` | Get specific agent |
| `aofctl agent apply -f file.yaml` | `aofctl apply -f file.yaml` | Create/update agent |
| `aofctl agent delete my-agent` | `aofctl delete agent my-agent` | Delete an agent |
| `aofctl agent validate -f file.yaml` | `aofctl apply -f file.yaml --dry-run` | Validate configuration |
| `aofctl agent describe my-agent` | `aofctl describe agent my-agent` | Show agent details |

### Workflow Commands

| Old Command | New Command | Description |
|------------|-------------|-------------|
| `aofctl workflow run config.yaml` | `aofctl run workflow config.yaml` | Execute workflow |
| `aofctl workflow get` | `aofctl get workflows` | List all workflows |
| `aofctl workflow get my-workflow` | `aofctl get workflow my-workflow` | Get specific workflow |
| `aofctl workflow delete my-workflow` | `aofctl delete workflow my-workflow` | Delete workflow |

### Tool Commands

| Old Command | New Command | Description |
|------------|-------------|-------------|
| `aofctl tools --server npx --args claude-flow` | `aofctl get tools` | List available tools |

### Other Commands

| Old Command | New Command | Description |
|------------|-------------|-------------|
| `aofctl version` | `aofctl version` | Show version (unchanged) |

---

## Step-by-Step Migration

### 1. Update Your Scripts and Automation

Find all references to aofctl in your scripts and update them:

```bash
# Find references (bash/shell scripts)
grep -r "aofctl" .

# Find references (Makefiles)
grep -r "aofctl" Makefile*

# Find references (CI/CD configs)
grep -r "aofctl" .github/ .gitlab-ci.yml
```

**Examples of common migrations:**

Before:
```bash
#!/bin/bash
aofctl agent run config.yaml --input "process"
aofctl agent get
aofctl agent delete my-agent
```

After:
```bash
#!/bin/bash
aofctl run agent config.yaml --input "process"
aofctl get agents
aofctl delete agent my-agent
```

### 2. Update Your YAML Configurations

Most YAML configurations don't change, but if you have documentation or examples, update references:

**Note:** The YAML structure itself remains unchanged. Only the CLI commands change.

### 3. Update Documentation and READMEs

Search your documentation for aofctl command examples and update them:

```bash
# Find in markdown files
grep -r "aofctl" docs/ README.md *.md
```

Update patterns like:
- `aofctl agent run` → `aofctl run agent`
- `aofctl agent get` → `aofctl get agents`
- `aofctl ... agent` → `aofctl ... agent` (when noun comes after verb)

### 4. Train Your Team

Share these key changes with your team:

**Quick Reference Card:**

```
OLD → NEW Pattern
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Noun-Verb → Verb-Noun

aofctl <noun> <verb> → aofctl <verb> <noun>

EXAMPLES:
  agent run   → run agent
  agent get   → get agents
  agent apply → apply (same)
  agent delete → delete agent
```

---

## Detailed Command Changes

### Running Agents

**Old:**
```bash
aofctl agent run --config my-config.yaml --input "What is 2+2?" --output json
```

**New:**
```bash
aofctl run agent my-config.yaml --input "What is 2+2?" --output json
```

---

### Listing Resources

**Old:**
```bash
aofctl agent get
aofctl workflow list
aofctl tools
```

**New:**
```bash
aofctl get agents              # or: aofctl get agent (singular works too)
aofctl get workflows           # or: aofctl get workflow
aofctl get tools              # or: aofctl get tool
```

---

### Applying Configurations

**Old:**
```bash
aofctl agent apply -f agent.yaml
```

**New:**
```bash
aofctl apply -f agent.yaml
```

**Note:** The `apply` command is unchanged! It already follows the new pattern.

---

### Deleting Resources

**Old:**
```bash
aofctl agent delete my-agent
aofctl workflow delete my-workflow
```

**New:**
```bash
aofctl delete agent my-agent
aofctl delete workflow my-workflow
```

---

### Getting Specific Resources

**Old:**
```bash
aofctl agent get my-agent
aofctl workflow get my-workflow
```

**New:**
```bash
aofctl get agent my-agent
aofctl get workflow my-workflow
```

Or use the shorthand:
```bash
aofctl get ag my-agent        # using shorthand 'ag' for agent
aofctl get wf my-workflow     # using shorthand 'wf' for workflow
```

---

## New Features Available

With the new kubectl-compatible structure, you now have access to:

### New Commands

1. **`aofctl describe`** - Show detailed resource information
   ```bash
   aofctl describe agent my-agent
   ```

2. **`aofctl logs`** - View resource logs (coming soon)
   ```bash
   aofctl logs agent my-agent --follow
   ```

3. **`aofctl exec`** - Execute commands in resources (coming soon)
   ```bash
   aofctl exec agent my-agent -- python script.py
   ```

4. **`aofctl api-resources`** - List all available resource types
   ```bash
   aofctl api-resources
   ```

### Enhanced Flags

1. **Output Format Support**
   ```bash
   aofctl get agents -o json      # JSON output
   aofctl get agents -o yaml      # YAML output
   aofctl get agents -o wide      # Wide table
   aofctl get agents -o name      # Names only
   ```

2. **Namespace Support**
   ```bash
   aofctl get agents -n production         # Specific namespace
   aofctl get agents --all-namespaces      # All namespaces
   ```

3. **Label Selectors** (coming soon)
   ```bash
   aofctl get agents -l env=production     # Filter by label
   ```

---

## Backward Compatibility

### Deprecation Timeline

- **Phase 1 (Current):** Old commands work with deprecation warnings
- **Phase 2 (Next Release):** Old commands disabled by default, can be enabled with `--legacy` flag
- **Phase 3 (Future):** Old commands removed completely

### Checking Compatibility

To ensure your setup is compatible:

```bash
# List your aofctl version
aofctl version

# Check for deprecated command usage
aofctl agent get 2>&1 | grep -i "deprecated"
```

---

## Troubleshooting Migration

### Common Issues

#### Issue: "Unknown command"

**Problem:** Using old command pattern
```bash
$ aofctl agent run config.yaml
# Error: Unknown command 'agent'
```

**Solution:** Use new verb-noun pattern
```bash
aofctl run agent config.yaml
```

#### Issue: "Unknown resource type"

**Problem:** Typo in resource type
```bash
$ aofctl get agnet
# Error: Unknown resource type: agnet
```

**Solution:** Verify correct resource type
```bash
aofctl api-resources  # See all available resources
aofctl get agents     # Correct spelling
```

#### Issue: Scripts failing after update

**Problem:** Old scripts using deprecated patterns

**Solution:** Update script to use new commands
```bash
# Find all aofctl invocations
grep -n "aofctl" myscript.sh

# Update each one manually or use sed
sed -i 's/aofctl agent run/aofctl run agent/g' myscript.sh
```

---

## Quick Lookup: Resource Type Names

### Resource Type Variations

Each resource type supports singular, plural, and short names:

| Long Name | Short Name | Description |
|-----------|-----------|-------------|
| agents / agent | ag | AI agents |
| workflows / workflow | wf | Multi-step workflows |
| tools / tool | tl | MCP tools |
| jobs / job | - | Job executions |
| cronjobs / cronjob | cj | Scheduled jobs |
| configs / config | cfg | Configuration objects |
| secrets / secret | sec | Sensitive data |
| deployments / deployment | deploy | Managed deployments |

### All Forms Work

```bash
# All of these are equivalent:
aofctl get agents
aofctl get agent
aofctl get ag

# Same for other resources:
aofctl get workflows
aofctl get workflow
aofctl get wf
```

---

## Getting Help

If you need help with specific commands:

```bash
# Get help for a specific command
aofctl get --help
aofctl run --help
aofctl delete --help

# See all available commands
aofctl --help

# List all resource types
aofctl api-resources
```

---

## Examples

### Migration Examples

**Example 1: CI/CD Pipeline Update**

Old pipeline:
```yaml
script:
  - aofctl agent apply -f my-agent.yaml
  - aofctl agent run --config my-agent.yaml --input "data.json"
  - aofctl agent get
```

New pipeline (no changes needed for `apply`, but update `run` and `get`):
```yaml
script:
  - aofctl apply -f my-agent.yaml          # apply already uses new pattern
  - aofctl run agent my-agent.yaml --input "data.json"
  - aofctl get agents
```

**Example 2: Local Script Update**

Old script:
```bash
#!/bin/bash
set -e

echo "Creating agents..."
aofctl agent apply -f agents/researcher.yaml
aofctl agent apply -f agents/coder.yaml

echo "Running workflow..."
aofctl workflow run --config workflow.yaml --input "start"

echo "Checking status..."
aofctl agent get
aofctl workflow get
```

New script:
```bash
#!/bin/bash
set -e

echo "Creating agents..."
aofctl apply -f agents/researcher.yaml
aofctl apply -f agents/coder.yaml

echo "Running workflow..."
aofctl run workflow workflow.yaml --input "start"

echo "Checking status..."
aofctl get agents
aofctl get workflows
```

---

## Support and Questions

For migration questions or issues:

1. Check this guide's Troubleshooting section
2. Review the [Command Reference](../reference/aofctl-complete.md)
3. Run `aofctl --help` for latest command documentation
4. Open an issue on GitHub

---

## What's Next?

After migration, explore new features:

- **Labels and Selectors:** Filter resources by labels
- **Dry-run mode:** Test configurations before applying
- **Custom output:** Format output with custom columns
- **Watching:** Monitor resources in real-time

See the [Complete Reference](../reference/aofctl-complete.md) for all available features.

