---
name: git-operations
description: "Git repository operations and troubleshooting"
homepage: "https://docs.aof.sh/skills/git-operations"
metadata:
  emoji: "🌳"
  version: "1.0.0"
  requires:
    bins: ["git"]
    env: []
    config: []
  tags: ["git", "version-control", "operations"]
---

# Git Operations Skill

Perform git operations for code management, debugging, and repository maintenance.

## When to Use This Skill

- Need to check commit history
- Investigating code changes
- Managing branches and tags
- Resolving merge conflicts
- Checking repository status

## Steps

1. **Check status** — `git status`
2. **View history** — `git log --oneline -20`
3. **Find commits** — `git log --grep="pattern"`
4. **Check differences** — `git diff HEAD~1`
5. **List branches** — `git branch -a`
