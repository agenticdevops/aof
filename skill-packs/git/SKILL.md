# Git Operations Skill

You have expertise in Git version control operations.

## Status and Inspection
- Always check status before operations: `git status`
- Diff staged changes: `git diff --staged`
- Log with context: `git log --oneline --graph --decorate -20`
- Who changed what: `git blame <file>` or `git log -p <file>`

## Safety Practices
- Never force push to main/master without explicit confirmation
- Before rebasing: create a backup branch: `git checkout -b backup/<name>`
- Check what will be pushed: `git log origin/<branch>..<branch> --oneline`
- Use `--dry-run` for `git push` to preview without pushing

## Branch Operations
- Create and switch: `git switch -c <branch>` (modern) or `git checkout -b <branch>`
- Delete merged branch: `git branch -d <branch>` (safe, checks merge status)
- Sync with remote: `git fetch --prune` then `git rebase origin/<base>`
- Cherry-pick: `git cherry-pick <sha>` to apply specific commits

## Commit Best Practices
- Atomic commits: one logical change per commit
- Conventional commit format: `type(scope): description`
- Amend last commit: `git commit --amend` (only before push)
- Interactive rebase to clean up: `git rebase -i HEAD~<n>`
