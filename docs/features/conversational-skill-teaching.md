# Conversational Skill Teaching

Convert tribal knowledge and runbooks into executable SKILL.md files through natural language.

## What Is Skill Teaching?

Skill teaching transforms operational knowledge into structured, version-controlled skill files that agents can execute. Instead of writing SKILL.md by hand, describe what you want and the system generates a complete template.

## Teaching a Skill

### Basic Skill Teaching

```
You: Learn how to debug Postgres connections
```

The system will:
1. Derive skill name: `debug-postgres-connections`
2. Generate SKILL.md with steps, examples, validation criteria
3. Create the file at `skills/debug-postgres-connections/SKILL.md`
4. Prompt you to customize the steps

### Example Conversations

**Example 1: Database Debugging**

```
You: Learn how to debug slow Postgres queries

System: Created skill: debug-slow-postgres-queries

Description: Learn how to debug slow Postgres queries

Location: skills/debug-slow-postgres-queries/SKILL.md

Review the generated SKILL.md and customize the steps, commands, and validation criteria.
```

**Example 2: Kubernetes Operations**

```
You: Learn how to safely restart a crashlooping pod

System: Created skill: restart-crashlooping-pod

Description: Learn how to safely restart a crashlooping pod

Location: skills/restart-crashlooping-pod/SKILL.md

Review the generated SKILL.md and customize the steps, commands, and validation criteria.
```

**Example 3: Cost Optimization**

```
You: Learn how to identify idle EC2 instances

System: Created skill: identify-idle-ec2-instances

Description: Learn how to identify idle EC2 instances

Location: skills/identify-idle-ec2-instances/SKILL.md

Review the generated SKILL.md and customize the steps, commands, and validation criteria.
```

## What Gets Generated

### SKILL.md Structure

A complete skill file with:
1. YAML frontmatter (metadata)
2. Overview description
3. Step-by-step instructions with code examples
4. Common issues and solutions
5. Validation criteria

### Example SKILL.md

```markdown
---
name: debug-postgres-connections
description: Learn how to debug Postgres connections
metadata:
  emoji: 🎯
  version: "1.0.0"
  tags: []
  requires: []
---

# debug-postgres-connections

Learn how to debug Postgres connections

## Steps

### 1. Preparation

Prepare your environment and gather necessary information.

\```bash
# Check Postgres is running
systemctl status postgresql

# Get connection count
psql -c "SELECT count(*) FROM pg_stat_activity;"
\```

### 2. Execution

Execute the main task.

\```bash
# Check for connection errors
tail -f /var/log/postgresql/postgresql.log | grep -i "connection"

# List active connections
psql -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"
\```

### 3. Verification

Verify the results.

\```bash
# Confirm connection pool is healthy
psql -c "SELECT count(*) FROM pg_stat_activity WHERE state != 'idle';"
\```

## Common Issues

- **Issue 1**: Too many connections
  - **Solution**: Check max_connections setting, close idle connections

- **Issue 2**: Connection timeout
  - **Solution**: Check network, verify credentials, check firewall

## Validation

To verify this skill works correctly:

1. Run the preparation steps
2. Execute the main task
3. Verify expected outcomes match actual results
4. Check for any error messages or warnings

## Notes

This is a generated skill template. Customize the steps, commands, and validation criteria based on your specific use case.
```

## Skill Name Derivation

The system automatically derives kebab-case skill names from your description:

| Description | Generated Name |
|-------------|----------------|
| "debug Postgres connections" | debug-postgres-connections |
| "Monitor Kubernetes pods health" | monitor-kubernetes-pods |
| "analyze AWS cost anomalies" | analyze-aws-cost |

You can see the name in the confirmation message and customize it if needed.

## Updating Skills

### When Skill Already Exists

```
You: Learn how to debug Postgres connections

System: Skill 'debug-postgres-connections' already exists at skills/debug-postgres-connections/SKILL.md.

Would you like to:
1. Update the existing skill
2. Create a new variant with a different name
```

Choose option 1 to replace, or option 2 to create a variant (e.g., `debug-postgres-connections-v2`).

## Skill Format Reference

### YAML Frontmatter

```yaml
---
name: skill-name                # Unique identifier
description: Brief description  # One-line summary
metadata:
  emoji: 🎯                    # Visual identifier
  version: "1.0.0"             # Semantic version
  tags: []                     # Optional tags for categorization
  requires: []                 # Prerequisites (other skills)
---
```

### Required Sections

1. **Steps** (`## Steps`): Numbered step-by-step instructions
2. **Validation** (`## Validation`): How to verify the skill worked

### Optional Sections

1. **Common Issues** (`## Common Issues`): Troubleshooting guide
2. **Notes** (`## Notes`): Additional context

### Code Blocks

All skills must include at least one code block with actual commands:

\```bash
# Command description
command --with-flags
\```

## Best Practices

### 1. Be Specific in Descriptions

**Good:**
```
You: Learn how to debug Postgres connection pool exhaustion
```

**Better:**
```
You: Learn how to debug Postgres connection pool exhaustion in production environments
```

### 2. Include Context

**Good:**
```
You: Learn how to restart a pod
```

**Better:**
```
You: Learn how to safely restart a crashlooping pod without affecting other services
```

### 3. Review and Customize

Always review generated SKILL.md files and:
- Replace placeholder commands with real ones
- Add domain-specific steps
- Update validation criteria
- Add common issues you've encountered

## Validation Errors

If skill generation fails, you'll see specific errors:

| Error | Meaning | Fix |
|-------|---------|-----|
| MissingName | No `name:` in frontmatter | Ensure frontmatter has `name:` field |
| MissingDescription | No `description:` in frontmatter | Add `description:` field |
| TooFewSteps | Less than 2 `##` sections | Add more sections (Steps, Validation) |
| NoCodeExamples | No code blocks | Include at least one ``` code block |
| NoValidationCriteria | No Validation section | Add `## Validation` section |

## Examples by Domain

### Infrastructure

```
- Learn how to diagnose high CPU usage on Kubernetes nodes
- Learn how to troubleshoot DNS resolution failures
- Learn how to investigate network latency spikes
```

### Database

```
- Learn how to identify slow Postgres queries
- Learn how to debug Redis memory issues
- Learn how to optimize MySQL query performance
```

### Cloud Cost

```
- Learn how to find underutilized AWS EC2 instances
- Learn how to identify orphaned EBS volumes
- Learn how to analyze S3 storage costs
```

### Security

```
- Learn how to audit IAM permissions
- Learn how to investigate suspicious login attempts
- Learn how to rotate API keys safely
```

## Next Steps

1. Teach skills for common operational tasks
2. Review and customize generated SKILL.md files
3. Add skills to agent configurations (AGENTS.md)
4. Test skills in isolated environments first
5. Build a library of validated skills for your team

