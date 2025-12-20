# PR Review Workflow - Usage Guide

Complete automated PR lifecycle workflow with conditional routing, parallel reviews, and approval gates.

## Overview

The PR review workflow (`pr-review-workflow.yaml`) demonstrates advanced workflow orchestration features:

- **Parallel execution** of security, quality, performance, and test reviewers
- **State reducers** for aggregating results from parallel branches
- **Conditional routing** based on findings severity
- **Approval gates** for human-in-the-loop decisions
- **GitHub integration** for labels, comments, and review status
- **Error handling** with automatic fallback

## Workflow Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PR Review Workflow                           │
└─────────────────────────────────────────────────────────────────────┘

    ┌─────────────────┐
    │  Fetch PR Info  │ ← Entry point
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │     Triage      │ ← Classify PR type, add size labels
    └────────┬────────┘
             │
             ▼
    ┌─────────────────────────────────────────────────────────┐
    │              Parallel Review (4 branches)               │
    ├──────────┬──────────┬────────────┬────────────────────┤
    │ Security │  Quality │ Performance│  Test Coverage      │
    │ Review   │  Review  │  Review    │  Review             │
    └──────────┴──────────┴────────────┴────────────────────┘
             │
             ▼
    ┌─────────────────┐
    │ Evaluate Results│ ← Aggregate findings, determine path
    └────────┬────────┘
             │
             ├─────────────────────────────────────────┐
             │                                         │
             ▼                                         ▼
    ┌────────────────┐                     ┌──────────────────┐
    │ Critical Issues│                     │  High Issues     │
    │ Found?         │                     │  (>3)?           │
    └────┬───────────┘                     └────┬─────────────┘
         │ Yes                                  │ Yes
         ▼                                      ▼
    ┌────────────────┐                     ┌──────────────────┐
    │ Request Changes│                     │ Escalate to Human│
    │                │                     │ (Approval Gate)  │
    └────┬───────────┘                     └────┬─────────────┘
         │                                      │
         └──────────────┬───────────────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │ All Passed & │
                 │ Small PR?    │
                 └──────┬───────┘
                        │ Yes
                        ▼
                 ┌──────────────┐
                 │ Auto-Approve │
                 └──────┬───────┘
                        │
                        ▼
                 ┌──────────────┐
                 │ Add Labels & │
                 │ Comment      │
                 └──────┬───────┘
                        │
                        ▼
                 ┌──────────────┐
                 │   Notify &   │
                 │   Complete   │
                 └──────────────┘
```

## Execution Flow

### 1. Fetch PR Info
- Retrieves PR metadata from GitHub API
- Extracts: number, title, author, files changed, lines added/deleted
- **Agent**: `pr-info-fetcher` (gemini-2.5-flash)

### 2. Triage
- Classifies PR type: feature, bugfix, refactor, docs, chore, hotfix
- Adds size labels: size/XS, S, M, L, XL
- Suggests component labels based on files changed
- **Agent**: `pr-triager` (gemini-2.5-flash)

### 3. Parallel Review
Four specialized reviewers run concurrently:

#### Security Review
- Scans for OWASP Top 10 vulnerabilities
- Checks for secrets in code
- Identifies auth/authz issues
- **Agent**: `security-reviewer` (gemini-2.5-flash)

#### Code Quality Review
- Style and formatting compliance
- Best practices adherence
- Code complexity analysis
- SOLID principles
- Assigns quality score (0-100)
- **Agent**: `code-quality-reviewer` (gemini-2.5-flash)

#### Performance Review
- N+1 query detection
- Inefficient algorithms
- Memory leak patterns
- Missing database indexes
- **Agent**: `performance-reviewer` (gemini-2.5-flash)

#### Test Coverage Review
- Estimates test coverage
- Checks for edge case tests
- Validates test quality
- **Agent**: `test-reviewer` (gemini-2.5-flash)

### 4. Evaluate Results
- Aggregates findings from all reviewers
- Counts critical, high, medium, low severity issues
- Determines routing based on severity

### 5. Conditional Routing

The workflow routes to different paths based on findings:

| Condition | Route | Action |
|-----------|-------|--------|
| `findings.critical > 0` | request-changes | Block PR, request changes |
| `findings.high > 3` | escalate-to-human | Require manual approval |
| `all_passed && lines < 100` | auto-approve | Automatically approve small, clean PRs |
| Otherwise | add-labels-and-comment | Standard review comment |

### 6. Terminal Actions

#### Request Changes
- Posts detailed review comment
- Lists all findings by category
- Sets PR status to "REQUEST_CHANGES"
- Adds `changes-requested` label

#### Escalate to Human
- Triggers approval gate (human-in-the-loop)
- Requires approval from tech-leads or senior-engineers
- Timeout: 2 hours
- If approved: continues to standard path
- If denied: manual review notification

#### Auto-Approve
- Posts congratulatory review comment
- Sets PR status to "APPROVE"
- Adds `auto-approved` and `lgtm` labels
- Only for small PRs (<100 lines) with no issues

#### Add Labels and Comment
- Applies all collected labels
- Posts comprehensive review summary
- Includes statistics and next steps

## State Management

The workflow maintains comprehensive state:

```yaml
state:
  # PR Info
  pr_number: 123
  pr_url: "https://github.com/org/repo/pull/123"
  pr_title: "Add user authentication"
  pr_author: "developer1"
  pr_type: "feature"

  # Changes
  files_changed: ["src/auth.js", "tests/auth.test.js"]
  lines_added: 145
  lines_deleted: 23
  total_lines_changed: 168

  # Review Results (from parallel branches)
  review_results:
    security:
      findings: [...]
      severity: "medium"
      passed: true
    code_quality:
      findings: [...]
      score: 85
      passed: true
    performance:
      findings: []
      issues_found: 0
      passed: true
    tests:
      coverage_percent: 78
      passed: true

  # Aggregated Findings
  findings:
    critical: 0
    high: 1
    medium: 3
    low: 2
    total: 6

  # Labels
  labels_to_add: ["feature", "size/M", "backend", "needs-tests"]

  # Verdict
  final_verdict: "approved"
  all_passed: true
```

## State Reducers

The workflow uses reducers to merge parallel results:

```yaml
reducers:
  review_results:
    type: merge
    # Merges {security: {...}, code_quality: {...}, ...}

  findings:
    type: merge
    # Merges {critical: 1, high: 2} + {critical: 0, high: 1} = {critical: 1, high: 3}

  labels_to_add:
    type: append
    # Appends ["size/M"] + ["backend"] + ["needs-tests"] = ["size/M", "backend", "needs-tests"]
```

## Usage

### Prerequisites

1. **GitHub MCP Server** configured:
```yaml
# In your daemon config or agent config
mcp_servers:
  - name: github
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

2. **GitHub Token** with permissions:
   - `repo` - Full repository access
   - `pull_requests:write` - PR reviews and labels
   - `issues:write` - Labels and comments

3. **Export token**:
```bash
export GITHUB_TOKEN=ghp_your_github_token
```

### Running the Workflow

#### Option 1: Trigger from GitHub Webhook

Configure a GitHub webhook to trigger on PR events:

```yaml
# daemon.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: pr-review-bot

spec:
  triggers:
    github:
      enabled: true
      webhook_secret_env: GITHUB_WEBHOOK_SECRET
      events:
        pull_request:
          - opened
          - synchronize  # New commits pushed

      # Route to workflow
      routing:
        - event: pull_request
          workflow: pr-review-workflow
          input:
            pr_number: "{{.pull_request.number}}"
            pr_url: "{{.pull_request.html_url}}"
```

Start the daemon:
```bash
export GITHUB_TOKEN=ghp_...
export GITHUB_WEBHOOK_SECRET=your_webhook_secret
aofctl serve --config daemon.yaml
```

#### Option 2: Manual Trigger

Run the workflow manually for a specific PR:

```bash
# Using aofctl (when workflow execution is implemented)
aofctl run workflow pr-review-workflow.yaml \
  --input '{
    "pr_number": 123,
    "pr_url": "https://github.com/org/repo/pull/123"
  }'
```

#### Option 3: From AgentFlow

Integrate into an AgentFlow for routing:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: github-automation

spec:
  routing:
    - match:
        event: pull_request
        action: [opened, synchronize]
      route:
        workflow: pr-review-workflow
```

## Customization

### Adjust Severity Thresholds

Modify conditional routing in `evaluate-results` step:

```yaml
# Current: Block if any critical issues
- condition: "state.findings.critical > 0"
  target: request-changes

# More lenient: Block only if 2+ critical issues
- condition: "state.findings.critical >= 2"
  target: request-changes
```

### Change Auto-Approve Criteria

```yaml
# Current: auto-approve if <100 lines and all passed
- condition: "state.all_passed && state.total_lines_changed < 100"
  target: auto-approve

# More strict: require tests and <50 lines
- condition: "state.all_passed && state.total_lines_changed < 50 && state.review_results.tests.coverage_percent >= 80"
  target: auto-approve
```

### Add More Reviewers

Add a new branch to `parallel-review`:

```yaml
branches:
  # ... existing reviewers ...

  - name: documentation-review
    steps:
      - agent:
          metadata:
            name: docs-reviewer
          spec:
            model: google:gemini-2.5-flash
            instructions: |
              Review documentation quality:
              - README updates for new features
              - API documentation
              - Code comments
              - Changelog entries
```

### Custom Labels

Modify the `triage` or `evaluate-results` agents to add custom labels:

```yaml
instructions: |
  Add labels based on:
  - Component: "backend", "frontend", "database", "api"
  - Priority: "priority/high", "priority/medium", "priority/low"
  - Type: "feature", "bugfix", "refactor"
  - Status: "needs-review", "approved", "changes-requested"
```

## Error Handling

The workflow includes comprehensive error handling:

1. **Retry Configuration**: Exponential backoff for GitHub API failures
2. **Error Handler Agent**: Graceful failure with user notification
3. **Timeout Protection**: 15-minute timeout on parallel reviews

If any step fails, the `error-handler` agent:
- Logs detailed error information
- Posts a comment on the PR about the failure
- Adds "workflow-error" label
- Notifies maintainers

## Best Practices

### Model Selection
- Use **gemini-2.5-flash** for all reviewers (fast, cost-effective)
- Use lower temperature (0.1-0.2) for consistency
- Increase max_tokens for complex reviews (2000-3000)

### GitHub API Rate Limits
- Parallel reviewers share the rate limit
- Add retry logic (already configured)
- Consider caching PR data between steps

### State Size
- Keep state focused on essential data
- Don't store entire file contents in state
- Use references (file paths) instead of values

### Testing
1. Test with small PRs first
2. Verify GitHub token permissions
3. Check MCP server connectivity
4. Monitor workflow execution logs

## Monitoring

### Check Workflow Status

```bash
# When aofctl workflow commands are implemented
aofctl get workflow pr-review-workflow
aofctl logs workflow pr-review-workflow --follow
```

### Review Metrics

Track workflow performance:
- Average execution time
- Success vs. failure rate
- Auto-approval rate
- Escalation frequency

## Security Considerations

1. **GitHub Token**: Use fine-grained tokens with minimal permissions
2. **Secrets**: Never log or store tokens in state
3. **Approval Gate**: Configure approvers list carefully
4. **Command Execution**: Review agents use `git` and `shell` - ensure sandboxing

## Future Enhancements

- [ ] Integrate with CI/CD status checks
- [ ] Add code formatting auto-fix
- [ ] Suggest code improvements via inline comments
- [ ] Track review metrics over time
- [ ] Multi-repository support
- [ ] Custom review rule configurations
- [ ] Integration with JIRA/Linear for ticket linking

## Troubleshooting

### Workflow doesn't trigger
- Check GitHub webhook configuration
- Verify `GITHUB_WEBHOOK_SECRET` matches
- Check daemon logs: `aofctl logs --follow`

### GitHub API errors
- Verify `GITHUB_TOKEN` has correct permissions
- Check rate limit: `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit`
- Review retry configuration

### Parallel reviews timeout
- Increase timeout in `join.timeout`
- Check individual agent performance
- Consider running reviews sequentially for large PRs

### Incorrect routing
- Review state after `evaluate-results` step
- Check conditional expressions syntax
- Verify state field names match schema

## Related Documentation

- [Workflow Spec](../../docs/reference/workflow-spec.md)
- [GitHub Integration](../../docs/concepts/github-integration.md)
- [MCP Integration](../../docs/tools/mcp-integration.md)
- [Agent Spec](../../docs/reference/agent-spec.md)

## Example Output

### Auto-Approved PR

```
✅ Code Review Summary

PR #123: Add user authentication
Author: @developer1
Type: feature | Size: M

Review Results:
✅ Security: No issues found
✅ Code Quality: Score 87/100 (Good)
✅ Performance: No issues found
✅ Test Coverage: 82% (Good)

Findings: 0 critical, 0 high, 2 medium, 1 low

Labels Applied:
- feature
- size/M
- backend
- auto-approved
- lgtm

Verdict: AUTO-APPROVED ✨
This PR meets all quality standards and has been automatically approved.
```

### Changes Requested PR

```
⚠️ Code Review: Changes Requested

PR #456: Database migration script
Author: @developer2
Type: refactor | Size: L

Review Results:
❌ Security: 2 critical issues found
⚠️ Code Quality: Score 68/100 (Needs improvement)
✅ Performance: No issues found
⚠️ Test Coverage: 45% (Below threshold)

Critical Issues:
1. [security] SQL injection vulnerability in migration script
   - File: migrations/001_users.sql
   - Line: 23
   - Issue: Unsanitized user input in SQL query
   - Fix: Use parameterized queries

2. [security] Hardcoded database credentials
   - File: config/database.js
   - Line: 5
   - Issue: Database password in source code
   - Fix: Use environment variables

Medium Issues:
1. [quality] High cyclomatic complexity (15)
   - File: src/auth/validator.js
   - Recommendation: Extract validation logic into separate functions

2. [tests] Missing test coverage for error paths
   - Coverage: 45% (threshold: 70%)
   - Recommendation: Add tests for edge cases

Labels Applied:
- refactor
- size/L
- changes-requested
- needs-security-review
- needs-tests

Verdict: CHANGES REQUESTED ⛔
Please address the critical security issues before resubmitting.
```

## Contributing

To improve this workflow:
1. Test with various PR scenarios
2. Gather feedback from team
3. Tune thresholds and conditions
4. Add project-specific reviewers
5. Document customizations

## License

Apache 2.0 - Same as AOF framework
