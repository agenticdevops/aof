---
sidebar_position: 2
title: Writing Skills
description: Create your own agentic skills to codify domain knowledge
---

# Writing Skills

This guide shows you how to create effective skills that codify your team's tribal knowledge.

## Skill File Structure

Skills are defined in `SKILL.md` files within a skill directory:

```
.claude/skills/
├── my-skill/
│   └── SKILL.md
├── another-skill/
│   ├── SKILL.md
│   └── references/
│       └── extended-docs.md
```

## SKILL.md Format

Every skill has two parts:

1. **YAML Frontmatter**: Metadata between `---` delimiters
2. **Markdown Content**: Instructions for the agent

```markdown
---
name: skill-name
description: "Brief description of what this skill does"
homepage: "https://docs.example.com/skill"
metadata:
  emoji: "🔧"
  version: "1.0.0"
  author: "Your Team"
  requires:
    bins: ["required-cli"]
    env: ["REQUIRED_VAR"]
    config: ["~/.config/tool"]
  install:
    - id: brew
      kind: brew
      package: tool-name
      bins: ["tool-cli"]
  tags:
    - category
    - subcategory
---

# Skill Title

Instructions for the agent...
```

## Frontmatter Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique identifier (kebab-case recommended) |
| `description` | string | Brief description (1-2 sentences) |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `homepage` | string | URL to additional documentation |
| `metadata.emoji` | string | Display emoji |
| `metadata.version` | string | Semantic version |
| `metadata.author` | string | Author/team name |
| `metadata.tags` | string[] | Categorization tags |
| `metadata.always` | boolean | Always load regardless of requirements |

### Requirements

Requirements determine when a skill is eligible:

```yaml
metadata:
  requires:
    bins:          # ALL must be in PATH
      - kubectl
      - helm
    any_bins:      # At least ONE must be available
      - podman
      - docker
    env:           # ALL must be set
      - KUBECONFIG
      - AWS_PROFILE
    config:        # ALL paths must exist
      - "~/.kube/config"
      - "~/.aws/credentials"
  os:              # Restrict to specific OS
    - darwin
    - linux
```

### Install Specifications

Help users install missing dependencies:

```yaml
metadata:
  install:
    - id: brew-kubectl
      kind: brew
      package: kubernetes-cli
      bins:
        - kubectl
    - id: apt-kubectl
      kind: apt
      package: kubectl
      bins:
        - kubectl
    - id: manual
      kind: manual
      package: kubectl
      url: "https://kubernetes.io/docs/tasks/tools/"
```

Supported installer kinds: `brew`, `apt`, `dnf`, `npm`, `pip`, `cargo`, `manual`

## Writing Effective Content

### Structure Your Instructions

```markdown
# Skill Name

## When to Use This Skill
- Scenario 1
- Scenario 2
- Scenario 3

## Quick Start
[Most common operation, copy-paste ready]

## Common Operations

### Operation 1
```bash
command-here
```

### Operation 2
```bash
another-command
```

## Troubleshooting

### Common Issue 1
**Symptoms:** What the user sees
**Cause:** Why it happens
**Solution:** How to fix it

## Reference
[Tables, links, additional context]
```

### Best Practices

#### Be Specific and Actionable
```markdown
# Good
```bash
kubectl logs <pod-name> --previous --tail=100
```

# Bad
Use kubectl to check the logs
```

#### Include Context
```markdown
# Good
## CrashLoopBackOff
**What it means:** Pod is crashing repeatedly
**Common causes:**
- Application error on startup
- Missing configuration
- Insufficient memory

# Bad
## CrashLoopBackOff
Run: kubectl describe pod
```

#### Provide Copy-Paste Commands
```markdown
# Good
```bash
# Get all pods in error state
kubectl get pods -A | grep -E 'Error|CrashLoopBackOff|ImagePullBackOff'
```

# Bad
Filter pods by error status
```

#### Cover Edge Cases
```markdown
## If Pod Has Multiple Containers
```bash
kubectl logs <pod-name> -c <container-name>
```

## If Previous Container Doesn't Exist
The pod may not have crashed yet. Check current logs:
```bash
kubectl logs <pod-name> --timestamps
```
```

## Real-World Example

Here's a complete skill for PostgreSQL backup operations:

```markdown
---
name: postgres-backup
description: "Backup and restore PostgreSQL databases in Kubernetes"
homepage: "https://wiki.internal/postgres-backup"
metadata:
  emoji: "🐘"
  version: "1.0.0"
  requires:
    bins:
      - kubectl
      - pg_dump
    config:
      - "~/.kube/config"
  install:
    - id: brew
      kind: brew
      package: postgresql
      bins:
        - pg_dump
        - pg_restore
  tags:
    - database
    - postgres
    - backup
    - disaster-recovery
---

# PostgreSQL Backup Skill

Procedures for backing up and restoring PostgreSQL databases running in Kubernetes.

## When to Use
- Creating pre-migration backups
- Disaster recovery preparation
- Data export for analysis
- Environment cloning

## Prerequisites
- `kubectl` with cluster access
- `pg_dump` installed locally
- Database credentials in Kubernetes secret

## Quick Backup

### 1. Port-Forward to Database
```bash
kubectl port-forward svc/postgres 5432:5432 &
```

### 2. Create Backup
```bash
pg_dump -h localhost -U postgres -d mydb -F c -f backup.dump
```

### 3. Verify Backup
```bash
pg_restore --list backup.dump | head -20
```

## Full Backup Script

```bash
#!/bin/bash
set -e

NAMESPACE=${1:-production}
DB_NAME=${2:-appdb}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${DB_NAME}_${TIMESTAMP}.dump"

# Get credentials from secret
DB_USER=$(kubectl get secret postgres-creds -n $NAMESPACE -o jsonpath='{.data.username}' | base64 -d)
DB_PASS=$(kubectl get secret postgres-creds -n $NAMESPACE -o jsonpath='{.data.password}' | base64 -d)

# Port forward
kubectl port-forward svc/postgres 5432:5432 -n $NAMESPACE &
PF_PID=$!
sleep 2

# Backup
PGPASSWORD=$DB_PASS pg_dump -h localhost -U $DB_USER -d $DB_NAME -F c -f $BACKUP_FILE

# Cleanup
kill $PF_PID

echo "Backup created: $BACKUP_FILE"
```

## Restore Procedure

```bash
# Restore to existing database
pg_restore -h localhost -U postgres -d mydb --clean backup.dump

# Restore to new database
createdb -h localhost -U postgres newdb
pg_restore -h localhost -U postgres -d newdb backup.dump
```

## Troubleshooting

### Connection Refused
1. Verify port-forward is running: `lsof -i :5432`
2. Check pod is ready: `kubectl get pods -l app=postgres`

### Permission Denied
Verify you have the correct credentials from the secret:
```bash
kubectl get secret postgres-creds -o yaml
```

### Backup File Corrupted
Always verify backups after creation:
```bash
pg_restore --list backup.dump
```
```

## Testing Your Skill

```bash
# Check if skill is detected
aofctl skills list | grep your-skill

# Check requirements
aofctl skills check your-skill

# View the skill
aofctl skills show your-skill
```

## Next Steps

- [Skill Reference](./skill-reference) - Complete specification
- [Bundled Skills](./bundled-skills) - See more examples
