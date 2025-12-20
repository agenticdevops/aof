# PR Review Workflow - Quick Start

Get started with automated PR reviews in 5 minutes.

## TL;DR

```bash
# 1. Set up GitHub token
export GITHUB_TOKEN=ghp_your_token_here

# 2. Run the workflow (when implemented)
aofctl run workflow pr-review-workflow.yaml \
  --input '{"pr_number": 123, "pr_url": "https://github.com/org/repo/pull/123"}'
```

## What It Does

Automatically reviews pull requests with:
- ✅ Security scanning (OWASP Top 10, secrets)
- ✅ Code quality analysis (SOLID, best practices, score 0-100)
- ✅ Performance review (N+1 queries, inefficient algorithms)
- ✅ Test coverage estimation
- ✅ Auto-labeling by type and size
- ✅ Conditional routing (approve, request changes, or escalate)

## Workflow Outcomes

| Scenario | Action |
|----------|--------|
| **Critical issues found** | Request changes, block PR |
| **3+ high issues** | Escalate to human approval |
| **All passed + <100 lines** | Auto-approve ✨ |
| **Otherwise** | Add labels & detailed review comment |

## Setup

### 1. GitHub Token

Create a GitHub token with these permissions:
- `repo` - Full repository access
- `pull_requests:write` - Reviews and labels
- `issues:write` - Comments and labels

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

### 2. MCP Server Configuration

Add to your daemon config:

```yaml
mcp_servers:
  - name: github
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

### 3. GitHub Webhook (Optional)

For automatic triggering on PR events:

```yaml
# daemon.yaml
spec:
  triggers:
    github:
      enabled: true
      webhook_secret_env: GITHUB_WEBHOOK_SECRET
      events:
        pull_request: [opened, synchronize]
      routing:
        - event: pull_request
          workflow: pr-review-workflow
```

Configure webhook in GitHub:
- URL: `https://your-server.com/webhooks/github`
- Events: Pull requests
- Secret: Same as `GITHUB_WEBHOOK_SECRET`

## Quick Customization

### Change Auto-Approve Threshold

Edit line ~280 in `pr-review-workflow.yaml`:

```yaml
# Current: <100 lines
- condition: "state.all_passed && state.total_lines_changed < 100"

# More strict: <50 lines AND 80% test coverage
- condition: "state.all_passed && state.total_lines_changed < 50 && state.review_results.tests.coverage_percent >= 80"

# More lenient: <500 lines
- condition: "state.all_passed && state.total_lines_changed < 500"
```

### Change Escalation Threshold

Edit line ~270:

```yaml
# Current: 3+ high issues
- condition: "state.findings.high > 3"

# More strict: 1+ high issue
- condition: "state.findings.high > 0"

# More lenient: 5+ high issues
- condition: "state.findings.high > 5"
```

### Add Custom Labels

Edit the `triage` agent (line ~90):

```yaml
instructions: |
  # Add your custom labels:
  - "team/backend" if files in src/backend/
  - "team/frontend" if files in src/frontend/
  - "priority/high" if hotfix or critical
```

## State Flow

```
Input → Fetch PR Info → Triage → Parallel Review → Evaluate
                            ↓
                        pr_type
                        labels
                            ↓
        ┌───────────────────┼───────────────────┐
        │                   │                   │
    Security          Code Quality        Performance
    Review              Review              Review
        │                   │                   │
        └───────────────────┴───────────────────┘
                            ↓
                    Aggregate Findings
                            ↓
        ┌───────────────────┼───────────────────┐
        │                   │                   │
    Critical?          High Issues?         All Passed?
        ↓                   ↓                   ↓
    Request            Escalate            Auto-Approve
    Changes             Human
```

## Example Outputs

### ✅ Auto-Approved

```
✅ Code Review Summary

PR #123: Add user authentication
Review Results:
✅ Security: No issues
✅ Code Quality: 87/100
✅ Performance: No issues
✅ Test Coverage: 82%

Verdict: AUTO-APPROVED ✨
```

### ⚠️ Changes Requested

```
⚠️ Code Review: Changes Requested

Critical Issues:
1. SQL injection in migration (line 23)
2. Hardcoded credentials (line 5)

Verdict: CHANGES REQUESTED
```

### 🚨 Escalated

```
🚨 Manual Review Required

5 high severity issues found
Approval needed from @tech-leads

[Approve] [Deny]
```

## Key Features Demonstrated

1. **Parallel Execution**: 4 reviewers run simultaneously
   ```yaml
   type: parallel
   branches: [security, quality, performance, tests]
   join: {strategy: all, timeout: 15m}
   ```

2. **State Reducers**: Merge results from parallel branches
   ```yaml
   reducers:
     review_results: {type: merge}
     findings: {type: merge}
     labels_to_add: {type: append}
   ```

3. **Conditional Routing**: Branch based on findings
   ```yaml
   next:
     - condition: "state.findings.critical > 0"
       target: request-changes
     - condition: "state.findings.high > 3"
       target: escalate-to-human
   ```

4. **Approval Gates**: Human-in-the-loop for critical decisions
   ```yaml
   type: approval
   config:
     approvers: [{role: tech-leads}]
     timeout: 2h
   ```

5. **Error Handling**: Graceful failure with notifications
   ```yaml
   errorHandler: error-handler
   retry:
     maxAttempts: 3
     backoff: exponential
   ```

## Testing

### Test with Sample PR

```bash
# 1. Create a test PR in your repo
# 2. Get the PR number (e.g., 123)
# 3. Run workflow

export GITHUB_TOKEN=ghp_...
aofctl run workflow pr-review-workflow.yaml \
  --input '{
    "pr_number": 123,
    "pr_url": "https://github.com/your-org/your-repo/pull/123"
  }'
```

### Monitor Execution

```bash
# Follow logs (when implemented)
aofctl logs workflow pr-review-workflow --follow

# Check status
aofctl get workflow pr-review-workflow
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| GitHub API 403 | Check token permissions: `repo`, `pull_requests:write` |
| Workflow timeout | Increase `join.timeout` or reduce parallel branches |
| No labels added | Verify token has `issues:write` permission |
| Reviewers fail | Check MCP server: `npx @modelcontextprotocol/server-github --version` |

## What's Next?

1. **Read the full guide**: `README-pr-review.md`
2. **Customize reviewers**: Add domain-specific checks
3. **Tune thresholds**: Adjust auto-approve criteria
4. **Add integrations**: Slack/Teams notifications
5. **Track metrics**: Monitor review quality over time

## Resources

- Full Documentation: `README-pr-review.md`
- Workflow YAML: `pr-review-workflow.yaml`
- GitHub MCP Server: https://github.com/modelcontextprotocol/servers/tree/main/src/github
- AOF Docs: https://docs.aof.sh

---

**Ready to use?** Copy `pr-review-workflow.yaml` to your project and run!
