---
name: "bugfix"
description: "Automatically fix GitHub issues labeled as 'bug'. Fetches bug issues, analyzes them, implements fixes, and creates PRs against the main branch."
---

# Bug Fix Automation

## What This Skill Does

Automates the bug fixing workflow by:
1. Fetching GitHub issues labeled as "bug"
2. Analyzing the issue and understanding the problem
3. Implementing the fix
4. Running tests to verify the fix
5. Creating a PR against the main branch

## Quick Start

When invoked, this skill will:

1. **Fetch bug issues** from the current repository
2. **Present issues** for selection (or work on the most recent one)
3. **Analyze the bug** by reading related code
4. **Implement the fix** with proper testing
5. **Create a PR** with detailed description

---

## Step-by-Step Process

### Step 1: Fetch Bug Issues

```bash
# List all open bug issues
gh issue list --label "bug" --state open --json number,title,body,labels,createdAt --limit 20
```

### Step 2: Select and Analyze Issue

For each bug issue, analyze:
- Issue title and description
- Steps to reproduce
- Expected vs actual behavior
- Related code files mentioned
- Stack traces or error messages

### Step 3: Create Fix Branch

```bash
# Create a branch for the fix
git checkout -b fix/issue-<number>-<short-description>
```

### Step 4: Implement the Fix

1. Locate the relevant code files
2. Understand the root cause
3. Implement the minimal fix
4. Add or update tests
5. Run the test suite

### Step 5: Create Pull Request

```bash
# Stage and commit changes
git add -A
git commit -m "fix: <description> (fixes #<issue-number>)"

# Push and create PR
git push -u origin fix/issue-<number>-<short-description>

gh pr create \
  --title "fix: <description>" \
  --body "## Summary
Fixes #<issue-number>

## Changes
- <change 1>
- <change 2>

## Testing
- [ ] Unit tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style
- [ ] Tests added/updated
- [ ] Documentation updated if needed"
```

---

## Execution Instructions

When this skill is invoked, follow these steps:

### 1. Get Repository Info
```bash
gh repo view --json owner,name,defaultBranchRef
```

### 2. List Bug Issues
```bash
gh issue list --label "bug" --state open --json number,title,body,createdAt,url --limit 10
```

### 3. For Each Bug (or Selected Bug):

**a) Read the issue details:**
```bash
gh issue view <number> --json title,body,comments
```

**b) Create a fix branch:**
```bash
git checkout main
git pull origin main
git checkout -b fix/issue-<number>-<short-slug>
```

**c) Analyze and implement:**
- Search codebase for related files
- Read the relevant code
- Understand the bug
- Implement the fix
- Write/update tests

**d) Verify the fix:**
```bash
# For Rust projects
cargo test
cargo clippy

# For Node projects
npm test
npm run lint
```

**e) Commit and push:**
```bash
git add -A
git commit -m "fix: <description>

Fixes #<issue-number>"
git push -u origin fix/issue-<number>-<short-slug>
```

**f) Create the PR:**
```bash
gh pr create --base main --title "fix: <title>" --body "<body>"
```

---

## Issue Selection

If multiple bug issues exist, present them to the user:

```
Found N open bug issues:

1. #123 - Login fails with special characters
   Created: 2 days ago

2. #118 - API returns 500 on empty request
   Created: 5 days ago

3. #115 - Memory leak in worker process
   Created: 1 week ago

Which issue would you like to work on? (Enter number or 'all' for batch mode)
```

---

## PR Template

Use this template for bug fix PRs:

```markdown
## Summary

Fixes #<issue-number>

## Problem

<Brief description of the bug>

## Solution

<Description of the fix and why it works>

## Changes

- <File 1>: <What changed>
- <File 2>: <What changed>

## Testing

- [ ] Existing tests pass
- [ ] New tests added for the fix
- [ ] Manually verified the fix

## Screenshots/Logs

<If applicable, add before/after screenshots or log output>
```

---

## Best Practices

1. **Minimal Changes**: Only fix the bug, don't refactor unrelated code
2. **Add Tests**: Every bug fix should include a test that would have caught it
3. **Link Issues**: Always reference the issue number in commits and PR
4. **Verify First**: Run tests before creating the PR
5. **Clear Description**: Explain the root cause and the fix

---

## Troubleshooting

### No bug issues found
```bash
# Check if label exists
gh label list | grep -i bug

# Create bug label if missing
gh label create bug --color d73a4a --description "Something isn't working"
```

### Authentication issues
```bash
# Check gh auth status
gh auth status

# Re-authenticate if needed
gh auth login
```

### Branch already exists
```bash
# Delete local branch and recreate
git branch -D fix/issue-<number>-<slug>
git checkout -b fix/issue-<number>-<slug>
```
