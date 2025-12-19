---
name: "Checkpoint Commit"
description: "Create quick checkpoint commits to save work-in-progress without detailed commit messages. Use when you want to save current progress, create a restore point, checkpoint before risky changes, or snapshot code mid-task."
---

# Checkpoint Commit

## What This Skill Does

Creates quick work-in-progress (WIP) checkpoint commits with auto-generated messages. Perfect for:
1. Saving progress before risky operations
2. Creating restore points mid-development
3. Quick snapshots without crafting commit messages
4. Preserving work before context switches

## Quick Start

```bash
# Basic checkpoint - stages all changes and commits
git add -A && git commit -m "checkpoint: WIP $(date +%Y-%m-%d_%H:%M:%S)"
```

## Checkpoint Patterns

### Pattern 1: Simple Timestamp Checkpoint
```bash
git add -A && git commit -m "checkpoint: $(date +%Y-%m-%d_%H:%M:%S)"
```

### Pattern 2: Checkpoint with Context
```bash
git add -A && git commit -m "checkpoint: WIP on <feature-name>"
```

### Pattern 3: Checkpoint Before Risky Change
```bash
git add -A && git commit -m "checkpoint: before <risky-operation>"
```

### Pattern 4: Checkpoint with File Summary
```bash
# Shows what's being checkpointed
git add -A && git commit -m "checkpoint: $(git diff --cached --stat | tail -1 | xargs)"
```

---

## Step-by-Step Guide

### Creating a Checkpoint

1. **Stage all changes**:
   ```bash
   git add -A
   ```

2. **Create checkpoint commit**:
   ```bash
   git commit -m "checkpoint: WIP $(date +%Y-%m-%d_%H:%M:%S)"
   ```

3. **Verify checkpoint created**:
   ```bash
   git log --oneline -1
   ```

### Restoring from Checkpoint

If you need to go back to a checkpoint:

```bash
# View recent checkpoints
git log --oneline --grep="checkpoint:" -10

# Reset to a specific checkpoint (keeps changes staged)
git reset --soft <checkpoint-hash>

# Hard reset to checkpoint (discards all changes after)
git reset --hard <checkpoint-hash>
```

### Cleaning Up Checkpoints

Before pushing, squash checkpoint commits into meaningful commits:

```bash
# Interactive rebase to squash checkpoints
git rebase -i HEAD~<number-of-commits>

# In the editor, change 'pick' to 'squash' for checkpoint commits
# Then write a proper commit message
```

---

## Best Practices

### When to Checkpoint
- Before refactoring code
- Before deleting files
- Before merge operations
- When switching tasks mid-work
- Before running automated fixes (linting, formatting)
- After completing a logical unit of work

### Checkpoint Message Conventions
```
checkpoint: WIP                    # Generic work-in-progress
checkpoint: before refactor        # Before risky operation
checkpoint: auth flow progress     # With feature context
checkpoint: 2024-01-15_14:30:00   # Timestamp-based
```

### Never Push Checkpoints
Checkpoints are local-only. Always squash or rewrite before pushing:
```bash
# Check for checkpoint commits before pushing
git log origin/main..HEAD --oneline --grep="checkpoint:"

# If found, rebase to clean up
git rebase -i origin/main
```

---

## Troubleshooting

### Issue: "Nothing to commit"
**Cause**: No changes since last commit
**Solution**: Only checkpoint when there are actual changes
```bash
# Check for changes first
git status --porcelain | grep -q . && git add -A && git commit -m "checkpoint: WIP"
```

### Issue: Accidentally pushed checkpoints
**Solution**: If not shared, rewrite history:
```bash
git rebase -i origin/main~<n>
# Squash checkpoint commits
git push --force-with-lease
```

### Issue: Too many checkpoints cluttering history
**Solution**: Squash before continuing work:
```bash
# Squash last N commits into one
git reset --soft HEAD~<n>
git commit -m "feat: <proper-message>"
```

---

## Advanced Usage

### Checkpoint with Branch Backup
For extra safety, create a backup branch:
```bash
git branch backup/$(date +%Y%m%d_%H%M%S) && git add -A && git commit -m "checkpoint: WIP"
```

### Auto-Checkpoint Hook (Optional)
Create a pre-commit alias for quick checkpoints:
```bash
# Add to ~/.gitconfig
[alias]
    cp = "!git add -A && git commit -m \"checkpoint: WIP $(date +%Y-%m-%d_%H:%M:%S)\""

# Usage
git cp
```

### View Checkpoint History
```bash
# List all checkpoints
git log --oneline --all --grep="checkpoint:"

# Show checkpoint diff
git show <checkpoint-hash>
```

---

## Related Commands

| Command | Purpose |
|---------|---------|
| `git stash` | Temporary storage (not a commit) |
| `git commit --amend` | Modify last commit |
| `git reset --soft` | Undo commit, keep changes staged |
| `git reflog` | View all HEAD changes (recovery) |
