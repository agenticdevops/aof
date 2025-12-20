---
id: github-integration
title: GitHub Integration
sidebar_label: GitHub Integration
description: Deep GitHub integration with webhooks and triggers for automated code reviews, issue triage, and CI/CD workflows
keywords: [github, integration, webhook, pull request, automation]
---

# GitHub Integration

AOF provides deep GitHub integration through webhooks and triggers, enabling AI agents to automate code reviews, issue triage, CI/CD workflows, and release management.

## Platform Support Status

| Platform | Status | Notes |
|----------|--------|-------|
| **GitHub** | ✅ Stable | Fully tested, production-ready |
| **GitLab** | 🧪 Experimental | Implemented but untested - contributions welcome |
| **Bitbucket** | 🧪 Experimental | Implemented but untested - contributions welcome |

**Notes:**
- GitHub is the primary supported platform for PR review automation and has been thoroughly tested in production environments
- GitLab and Bitbucket adapters follow the same architectural patterns as GitHub but have not been tested with real webhooks or API calls
- The API structure is designed to be platform-agnostic, making it easy to extend support to other Git platforms
- **Community contributions welcome**: If you use GitLab or Bitbucket, please test these integrations and report issues or submit improvements

## Overview

GitHub Integration enables your AOF agents to:

- **Automatically review pull requests** with AI-powered code analysis
- **Triage and label issues** based on content and context
- **Respond to comments** with intelligent suggestions
- **Run CI/CD checks** and update statuses
- **Automate release workflows** with changelogs and notifications
- **Monitor workflow runs** and react to failures

Unlike basic webhook integrations, AOF's GitHub support provides full agent orchestration - your agents can read code, post comments, manage labels, approve PRs, and create check runs with GitHub's official APIs.

## Key Concepts

### 1. GitHub Triggers

GitHub Triggers are AgentFlow event sources that listen to GitHub webhooks and route events to agents.

**How it works:**
1. Configure a webhook in your GitHub repository
2. Create a Trigger with `type: GitHub`
3. Define which events to listen for (push, pull_request, issues, etc.)
4. Route matching events to agents or fleets

```yaml
# flows/github-pr-review.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: github-pr-review-flow

spec:
  trigger:
    type: GitHub
    config:
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}

      # Which GitHub events to process
      events:
        - pull_request
        - pull_request_review_comment

      # Optional: Filter by event actions
      actions:
        - opened
        - synchronize
        - reopened

  agents:
    - name: code-reviewer
```

### 2. GitHub Event Types

AOF supports all major GitHub webhook events:

| Event | Description | Common Actions |
|-------|-------------|----------------|
| `pull_request` | PR opened, updated, closed | `opened`, `synchronize`, `closed`, `reopened` |
| `pull_request_review` | PR reviewed | `submitted`, `edited`, `dismissed` |
| `pull_request_review_comment` | Comment on PR diff | `created`, `edited`, `deleted` |
| `push` | Code pushed to branch | N/A |
| `issues` | Issue opened, edited, closed | `opened`, `edited`, `closed`, `labeled` |
| `issue_comment` | Comment on issue or PR | `created`, `edited`, `deleted` |
| `workflow_run` | GitHub Actions workflow run | `completed`, `requested`, `in_progress` |
| `release` | Release published | `published`, `created`, `edited` |
| `check_run` | Check run updated | `created`, `completed`, `rerequested` |
| `check_suite` | Check suite updated | `completed`, `requested`, `rerequested` |
| `repository` | Repository created/deleted | `created`, `deleted`, `archived` |

### 3. Event Actions

Each event type has specific actions that refine when your agent runs:

**pull_request actions:**
- `opened` - New PR created
- `synchronize` - PR updated with new commits
- `reopened` - Closed PR reopened
- `closed` - PR closed (check `merged` field to distinguish merge vs close)
- `labeled`, `unlabeled` - Labels changed
- `review_requested`, `review_request_removed` - Reviewers changed
- `ready_for_review` - Draft converted to ready

**issues actions:**
- `opened` - New issue created
- `edited` - Issue title/body edited
- `closed`, `reopened` - State changed
- `labeled`, `unlabeled` - Labels changed
- `assigned`, `unassigned` - Assignees changed

**issue_comment / pull_request_review_comment actions:**
- `created` - New comment posted
- `edited` - Comment edited
- `deleted` - Comment deleted

**workflow_run actions:**
- `completed` - Workflow finished (check `conclusion` for success/failure)
- `requested` - Workflow triggered
- `in_progress` - Workflow started

## Agent Capabilities

### What Agents Can Do

When triggered by GitHub events, agents can interact with GitHub through MCP tools:

| Capability | Description | Example Use Case |
|------------|-------------|------------------|
| **Read PR diffs** | Fetch changed files and line-level diffs | Code review, security scanning |
| **Read file contents** | Fetch full file contents from repo | Context for code review |
| **Post comments** | Add comments to PRs and issues | Review feedback, suggestions |
| **Post review** | Submit PR review (approve/request changes/comment) | Automated approval or blocking |
| **Manage labels** | Add/remove labels on PRs and issues | Auto-labeling, triage |
| **Update PR status** | Change PR title, description, milestone | Standardization |
| **Create check runs** | Add custom CI/CD checks | Custom validations |
| **Update check status** | Mark checks as passed/failed | CI/CD integration |
| **Manage assignees** | Assign/unassign users to issues/PRs | Auto-assignment |
| **Close/reopen** | Change issue/PR state | Auto-close duplicates |
| **Create issues** | File new issues programmatically | Bug detection, TODO tracking |
| **React to comments** | Add emoji reactions | Acknowledgment |

### GitHub MCP Tools

Agents use the `github-mcp` tool to interact with GitHub:

```yaml
# agents/code-reviewer.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: code-reviewer

spec:
  model: google:gemini-2.5-flash

  tools:
    - type: MCP
      config:
        server: github-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-github"]
        env:
          GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}

  system_prompt: |
    You are an expert code reviewer. When reviewing pull requests:

    1. Fetch the PR diff to see changed files
    2. Read full file contents for context
    3. Analyze for:
       - Security vulnerabilities
       - Performance issues
       - Code quality and style
       - Missing tests or documentation
    4. Post inline comments on specific lines
    5. Submit a review (APPROVE, REQUEST_CHANGES, or COMMENT)
    6. Add appropriate labels (needs-tests, security, performance)
```

## Supported Workflows

### 1. PR Review Automation

Automatically review code when PRs are opened or updated:

```yaml
# flows/github-pr-review.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: pr-review-flow

spec:
  trigger:
    type: GitHub
    config:
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events:
        - pull_request
      actions:
        - opened
        - synchronize

  agents:
    - name: security-reviewer
      description: "Security-focused code review"

    - name: performance-reviewer
      description: "Performance analysis"

    - name: style-reviewer
      description: "Code quality and style"

  # Run agents in parallel (peer mode)
  coordination:
    mode: peer
    consensus:
      algorithm: majority
```

### 2. Issue Triage

Automatically label and assign issues based on content:

```yaml
# flows/github-issue-triage.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: issue-triage-flow

spec:
  trigger:
    type: GitHub
    config:
      events:
        - issues
      actions:
        - opened

  agents:
    - name: issue-triager

  # Agent instructions
  context:
    prompt: |
      Analyze the issue and:
      1. Add appropriate labels (bug, enhancement, documentation, etc.)
      2. Assign to the right team member based on area
      3. Add helpful comments if info is missing
      4. Close if it's a duplicate or invalid
```

### 3. CI/CD Integration

Run custom checks when code is pushed:

```yaml
# flows/github-ci-checks.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: ci-checks-flow

spec:
  trigger:
    type: GitHub
    config:
      events:
        - pull_request
        - push
      actions:
        - opened
        - synchronize

  agents:
    - name: ci-validator

  context:
    checks:
      - name: "AI Code Quality"
        description: "AI-powered code quality analysis"
      - name: "Security Scan"
        description: "Automated security vulnerability scan"
```

### 4. Release Automation

Automate release workflows when tags are pushed or releases published:

```yaml
# flows/github-release.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: release-automation-flow

spec:
  trigger:
    type: GitHub
    config:
      events:
        - release
      actions:
        - published

  agents:
    - name: release-manager

  context:
    tasks:
      - "Generate changelog from commits"
      - "Post release notes to Slack"
      - "Update documentation versions"
      - "Create GitHub Discussion thread"
```

## Integration Patterns

### Pattern 1: Single Agent Review

Simple setup with one agent reviewing all PRs:

```yaml
# Trigger
spec:
  trigger:
    type: GitHub
    config:
      events: [pull_request]
      actions: [opened, synchronize]

  agents:
    - name: code-reviewer
```

### Pattern 2: Fleet-Based Multi-Reviewer

Multiple specialized agents review in parallel:

```yaml
# Use AgentFleet for parallel reviews
spec:
  trigger:
    type: GitHub
    config:
      events: [pull_request]

  fleet: code-review-fleet  # References an AgentFleet
```

```yaml
# fleets/code-review-fleet.yaml
apiVersion: aof.dev/v1alpha1
kind: AgentFleet
metadata:
  name: code-review-fleet

spec:
  agents:
    - ref: agents/security-reviewer.yaml
      role: specialist

    - ref: agents/performance-reviewer.yaml
      role: specialist

    - ref: agents/quality-reviewer.yaml
      role: specialist

  coordination:
    mode: peer
    consensus:
      algorithm: majority
```

### Pattern 3: Workflow-Based Review

Multi-step workflow with approval gates:

```yaml
# flows/pr-workflow.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: pr-workflow

spec:
  trigger:
    type: GitHub
    config:
      events: [pull_request]
      actions: [opened]

  nodes:
    - id: initial-review
      type: Agent
      config:
        agent: code-reviewer

    - id: approval-gate
      type: Conditional
      config:
        condition: "review.status == 'approved'"

    - id: security-check
      type: Agent
      config:
        agent: security-scanner

    - id: approve-pr
      type: GitHub
      config:
        action: approve

  connections:
    - from: start
      to: initial-review
    - from: initial-review
      to: approval-gate
    - from: approval-gate
      to: security-check
      condition: approved
    - from: security-check
      to: approve-pr
```

## Quick Example

Here's a complete PR review bot in under 50 lines:

```yaml
# flows/simple-pr-review.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: simple-pr-review

spec:
  trigger:
    type: GitHub
    config:
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events:
        - pull_request
      actions:
        - opened
        - synchronize

  agents:
    - name: code-reviewer
---
# agents/code-reviewer.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: code-reviewer

spec:
  model: google:gemini-2.5-flash

  tools:
    - type: MCP
      config:
        server: github-mcp
        command: ["npx", "-y", "@modelcontextprotocol/server-github"]
        env:
          GITHUB_PERSONAL_ACCESS_TOKEN: ${GITHUB_TOKEN}

  system_prompt: |
    You are a code reviewer. For each PR:
    1. Fetch the diff
    2. Review for security, performance, and quality issues
    3. Post inline comments on problematic lines
    4. Submit a review (APPROVE or REQUEST_CHANGES)
```

**Setup:**

1. Create webhook in GitHub repo:
   - URL: `https://your-domain.com/webhook/github`
   - Secret: Set as `GITHUB_WEBHOOK_SECRET`
   - Events: Pull requests, Pull request reviews

2. Set environment variables:
   ```bash
   export GITHUB_WEBHOOK_SECRET="your-webhook-secret"
   export GITHUB_TOKEN="ghp_your_personal_access_token"
   export GOOGLE_API_KEY="your-gemini-api-key"
   ```

3. Start the daemon:
   ```bash
   aofctl serve --config daemon.yaml
   ```

4. Open a PR - your agent will automatically review it!

## GitHub Webhook Setup

### Step 1: Create Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Select scopes:
   - `repo` - Full repository access
   - `write:discussion` - Create discussions (optional)
4. Copy the token (starts with `ghp_`)

### Step 2: Configure Repository Webhook

1. Go to your repository → Settings → Webhooks → Add webhook
2. Configure:
   - **Payload URL**: `https://your-domain.com/webhook/github`
   - **Content type**: `application/json`
   - **Secret**: Generate a strong random string (this is `GITHUB_WEBHOOK_SECRET`)
   - **SSL verification**: Enable
   - **Events**:
     - Pull requests
     - Pull request reviews
     - Pull request review comments
     - Issues
     - Issue comments
     - Pushes
     - Workflow runs
     - (Select based on your needs)
3. Click "Add webhook"

### Step 3: Expose Webhook Endpoint

For local development, use a tunnel:

```bash
# Option 1: Cloudflared (no signup)
brew install cloudflared
cloudflared tunnel --url http://localhost:3000

# Option 2: ngrok (free account required)
ngrok http 3000
```

Use the tunnel URL as your webhook payload URL.

For production, deploy to a server with a public domain and HTTPS.

## Environment Variables

```bash
# GitHub Integration
export GITHUB_TOKEN="ghp_xxxxx"              # Personal access token
export GITHUB_WEBHOOK_SECRET="random-string"  # Webhook secret

# LLM Provider
export GOOGLE_API_KEY="xxxxx"                # For Gemini models
# OR
export OPENAI_API_KEY="xxxxx"                # For GPT models
# OR
export ANTHROPIC_API_KEY="xxxxx"             # For Claude models
```

## Best Practices

### 1. Use Repository-Scoped Tokens

For production, use fine-grained personal access tokens with repository-specific permissions:

- Metadata: Read
- Contents: Read
- Pull requests: Read & Write
- Issues: Read & Write

### 2. Validate Webhook Signatures

AOF automatically validates webhook signatures using `webhook_secret`. Always configure this in production.

### 3. Rate Limiting

GitHub API has rate limits (5,000 requests/hour for authenticated users). AOF handles retries automatically, but:

- Avoid reviewing every tiny commit (use `synchronize` wisely)
- Cache file contents when possible
- Use conditional requests

### 4. Filter Events Carefully

Only subscribe to events you need:

```yaml
# ❌ Too broad - triggers on everything
events:
  - pull_request
  - issues
  - push

# ✅ Specific - only new PRs and updates
events:
  - pull_request
actions:
  - opened
  - synchronize
```

### 5. Use Fleet Mode for Complex Reviews

For production code review, use an AgentFleet with multiple specialized reviewers:

```yaml
fleet: code-review-fleet

# In fleets/code-review-fleet.yaml:
agents:
  - security-reviewer (model: gemini-2.5-flash)
  - performance-reviewer (model: gemini-2.5-flash)
  - quality-reviewer (model: gemini-2.5-flash)

coordination:
  mode: peer
  consensus:
    algorithm: majority  # 2/3 must agree
```

### 6. Use Check Runs for CI/CD

For CI/CD validation, use GitHub Check Runs API instead of PR comments:

```yaml
system_prompt: |
  After reviewing code:
  1. Create a check run named "AI Code Review"
  2. Set status to "in_progress" at start
  3. Set status to "completed" when done
  4. Use conclusion "success" or "failure"
  5. Include detailed output in summary
```

## Security Considerations

### Token Security

- Never commit tokens to git
- Use environment variables or secrets management
- Rotate tokens regularly
- Use minimum required permissions

### Webhook Security

- Always validate webhook signatures (enabled by default)
- Use HTTPS for webhook endpoints
- Restrict webhook IP addresses if possible
- Monitor webhook delivery logs

### Agent Permissions

Agents can perform destructive actions. For production:

```yaml
# Require approval for dangerous actions
approval:
  enabled: true
  require_for:
    - "close issue"
    - "delete comment"
    - "force push"
  allowed_users:
    - "U12345ADMIN"
```

## Troubleshooting

### Webhook Not Triggering

1. Check webhook delivery logs in GitHub repo settings
2. Verify webhook URL is publicly accessible
3. Check webhook secret matches `GITHUB_WEBHOOK_SECRET`
4. Ensure daemon is running: `aofctl serve --config daemon.yaml`
5. Check daemon logs: `RUST_LOG=debug aofctl serve`

### Agent Not Posting Comments

1. Verify `GITHUB_TOKEN` has correct permissions
2. Check MCP server is running: Look for "MCP server started" in logs
3. Verify agent has github-mcp tool configured
4. Test token manually:
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/user
   ```

### Rate Limiting Issues

```
Error: API rate limit exceeded
```

Solutions:
- Reduce event frequency (avoid `synchronize` on every commit)
- Cache file contents
- Use conditional requests
- Upgrade to GitHub Enterprise for higher limits

## Related Documentation

- [AgentFlow Routing Guide](../guides/agentflow-routing.md) - How message routing works
- [Agent Reference](../reference/agent-spec.md) - Complete agent specification
- [AgentFlow Reference](../reference/agentflow-spec.md) - Flow specification
- [MCP Integration](../tools/mcp-integration.md) - Using MCP tools
- [Approval Workflows](../guides/approval-workflow.md) - Human approval gates

## Examples

See the `examples/` directory for complete working examples:

- `examples/flows/github-pr-review.yaml` - PR review automation
- `examples/flows/github-issue-triage.yaml` - Issue labeling
- `examples/agents/code-reviewer.yaml` - Code review agent
- `examples/fleets/code-review-fleet.yaml` - Multi-reviewer fleet
