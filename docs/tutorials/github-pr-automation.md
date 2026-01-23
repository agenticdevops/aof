# Tutorial: Automated GitHub PR Review with AOF

This tutorial will guide you through setting up automated pull request reviews using AOF (Agentic Ops Framework).

## What You'll Build

By the end of this tutorial, you'll have:
- ✅ AOF daemon receiving GitHub webhook events
- ✅ Automated PR reviews with AI-powered code analysis
- ✅ Security and quality checks on every PR
- ✅ Automatic comments posted back to GitHub

## Prerequisites

- GitHub account with admin access to a repository
- AOF installed (`cargo build --release`)
- Basic understanding of webhooks
- For local development: [ngrok](https://ngrok.com/) for webhook tunneling

## Time Required

~15 minutes

## Step 1: Set Up Environment Variables

First, create a GitHub Personal Access Token:

1. Visit https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Give it a name: "AOF Bot"
4. Select scopes:
   - `repo` (Full control of private repositories)
   - `write:discussion` (Read and write discussions)
5. Generate and copy the token

Generate a webhook secret:

```bash
openssl rand -hex 32
```

Add these to your `~/.zshrc` (or `~/.bashrc`):

```bash
# GitHub Integration
export GITHUB_TOKEN="ghp_your_token_here"
export GITHUB_WEBHOOK_SECRET="your_webhook_secret_here"

# LLM Provider (using Google Gemini)
export GOOGLE_API_KEY="your_google_api_key"

# Optional: Existing platform tokens
export SLACK_BOT_TOKEN="xoxb-..."
export SLACK_SIGNING_SECRET="..."
export TELEGRAM_BOT_TOKEN="..."
```

**Important**: Source your config before starting AOF:

```bash
source ~/.zshrc  # or source ~/.bashrc
```

## Step 2: Configure AOF Daemon

Your `config/aof/daemon.yaml` should already have GitHub enabled:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: production

spec:
  server:
    port: 8080
    host: 0.0.0.0

  platforms:
    github:
      enabled: true
      token_env: GITHUB_TOKEN
      webhook_secret_env: GITHUB_WEBHOOK_SECRET
      bot_name: "aofbot"

  agents:
    directory: ./agents
    watch: true

  flows:
    directory: ./flows
    watch: true
    enabled: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
    max_tasks_per_user: 5
    default_agent: devops
```

## Step 3: Create PR Review Agent

Create `agents/github-pr-reviewer.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: github-pr-reviewer
  labels:
    category: code-review
    platform: github

spec:
  # Using Google Gemini 2.5 Flash for fast, cost-effective reviews
  model: google:gemini-2.5-flash

  system_prompt: |
    You are an expert code reviewer performing thorough pull request reviews.

    ## Review Focus Areas

    ### 1. Code Quality
    - Readability and maintainability
    - Proper error handling
    - Code organization and structure
    - DRY principle adherence

    ### 2. Security
    - SQL injection vulnerabilities
    - XSS risks
    - Authentication/authorization issues
    - Secrets or API keys in code
    - Insecure dependencies
    - Input validation issues

    ### 3. Performance
    - Inefficient algorithms
    - Memory leaks
    - N+1 query problems
    - Unnecessary API calls

    ### 4. Best Practices
    - Design patterns
    - Language-specific idioms
    - Testing coverage
    - Documentation quality

    ## Output Format

    Provide your review in this markdown format:

    ```markdown
    ## 🔍 Code Review Summary

    **Overall Assessment**: [Approve ✅ / Request Changes ⚠️ / Comment 💬]

    ### ✨ Strengths
    - [List positive aspects of the PR]
    - [Good patterns or approaches used]

    ### ⚠️ Issues Found

    #### Critical (Must Fix)
    - **[File:Line]** - [Issue description]

    #### High Priority
    - **[File:Line]** - [Issue description]

    #### Medium Priority
    - **[File:Line]** - [Suggestion]

    ### 💡 Suggestions for Improvement
    - [Actionable suggestions]
    - [Alternative approaches]

    ### 🔒 Security Analysis
    [Security concerns or "No security issues detected ✅"]

    ### 📝 Additional Notes
    [Any other relevant comments]

    ---
    *Automated review by AOF GitHub Bot*
    ```

    ## Guidelines
    - Be constructive and helpful, not just critical
    - Be specific with file names and line numbers
    - Provide actionable suggestions for fixes
    - Acknowledge both good and bad aspects
    - Prioritize issues by severity
    - If unsure, express uncertainty rather than being wrong

    If the PR looks good overall, say so! Don't manufacture issues.

  tools:
    - shell

  max_iterations: 8
  temperature: 0.3
```

## Step 4: Create PR Review Flow (Optional)

For more control, create `flows/github/pr-review.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: pr-review-flow
  labels:
    platform: github
    event: pull_request

spec:
  trigger:
    type: GitHub
    config:
      events:
        - pull_request.opened
        - pull_request.synchronize
      filters:
        # Don't review draft PRs
        - field: pull_request.draft
          operator: equals
          value: false

  nodes:
    # Step 1: Fetch PR diff using GitHub CLI
    - id: fetch-pr-diff
      type: Action
      action:
        type: shell
        command: |
          # Install gh CLI if not present
          if ! command -v gh &> /dev/null; then
              echo "Installing gh CLI..."
              brew install gh  # or appropriate package manager
          fi

          # Authenticate with token
          echo "$GITHUB_TOKEN" | gh auth login --with-token

          # Fetch PR diff
          gh pr diff ${{ event.pull_request.number }} \
            --repo ${{ event.repository.full_name }} \
            > /tmp/pr-diff.txt

          # Also get file list
          gh pr diff ${{ event.pull_request.number }} \
            --repo ${{ event.repository.full_name }} \
            --name-only \
            > /tmp/pr-files.txt

          cat /tmp/pr-diff.txt

    # Step 2: AI Review
    - id: review
      type: Agent
      agent: github-pr-reviewer
      input: |
        Review this pull request:

        **Repository**: ${{ event.repository.full_name }}
        **PR Number**: #${{ event.pull_request.number }}
        **Title**: ${{ event.pull_request.title }}
        **Author**: @${{ event.pull_request.user.login }}

        **Description**:
        ${{ event.pull_request.body }}

        **Files Changed** (${{ event.pull_request.changed_files }} files):
        - ${{ event.pull_request.additions }} additions
        - ${{ event.pull_request.deletions }} deletions

        **Code Changes**:
        ${{ nodes.fetch-pr-diff.output }}

        Please provide a thorough code review following the guidelines in your system prompt.

    # Step 3: Post review as comment
    - id: post-review
      type: Action
      action:
        type: github_comment
        owner: ${{ event.repository.owner.login }}
        repo: ${{ event.repository.name }}
        issue_number: ${{ event.pull_request.number }}
        body: ${{ nodes.review.output }}

    # Step 4: Add labels based on review
    - id: label-pr
      type: Condition
      condition: ${{ nodes.review.output contains "Critical" }}
      then:
        - type: github_labels
          labels: ["needs-work", "security-review"]
      else:
        - type: github_labels
          labels: ["reviewed", "ready-for-merge"]
```

## Step 5: Start AOF Daemon

```bash
# Make sure to source environment variables first!
source ~/.zshrc  # or source ~/.bashrc

# Build (if not already done)
cargo build --release

# Start the daemon
./target/release/aofctl serve --config config/aof/daemon.yaml
```

You should see:

```
Starting AOF Trigger Server
  Bind address: 0.0.0.0:8080
  Registered platform: github
  Pre-loaded 1 agents from "./agents"
Server starting...
```

## Step 6: Expose Webhook Endpoint

### For Local Development:

```bash
# In a new terminal
ngrok http 8080

# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
```

### For Production:

Deploy to a server with a public IP and domain:

```
https://aof.yourdomain.com/webhook/github
```

## Step 7: Configure GitHub Webhook

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Webhooks** → **Add webhook**

3. Configure the webhook:
   - **Payload URL**: `https://abc123.ngrok.io/webhook/github` (or your production URL)
   - **Content type**: `application/json`
   - **Secret**: Paste your `GITHUB_WEBHOOK_SECRET` value
   - **SSL verification**: Enable
   - **Which events would you like to trigger this webhook?**:
     - Select "Let me select individual events"
     - Check: ☑️ Pull requests
     - Check: ☑️ Pull request reviews
     - Check: ☑️ Pull request review comments
     - Check: ☑️ Issue comments (optional, for commands)
   - **Active**: ☑️ Checked

4. Click "Add webhook"

5. GitHub will immediately send a `ping` event. Check "Recent Deliveries" to verify:
   - Green checkmark = success
   - HTTP 200 response
   - Check AOF logs for "ping event received"

## Step 8: Test It!

### Create a Test PR

```bash
# Create a test branch
git checkout -b test-pr-review

# Make a simple change
echo "# Test PR" >> README.md

# Commit and push
git add README.md
git commit -m "test: trigger automated PR review"
git push origin test-pr-review
```

### Open Pull Request

1. Go to your repository on GitHub
2. Click "Compare & pull request"
3. Fill in title and description
4. Click "Create pull request"

### Watch the Magic! 🎉

Within seconds, you should see:

1. **In ngrok dashboard**: Webhook received
2. **In AOF logs**:
   ```
   INFO aof_triggers::handler: Received GitHub pull_request event
   INFO aof_runtime: Executing agent github-pr-reviewer
   ```
3. **On GitHub PR**: Automated review comment appears!

## What Just Happened?

Let's break down the workflow:

1. **Webhook Event**: GitHub sent a `pull_request.opened` event to your AOF endpoint
2. **Signature Verification**: AOF verified the webhook signature using `GITHUB_WEBHOOK_SECRET`
3. **Event Parsing**: The GitHub platform adapter parsed the webhook payload
4. **Agent Selection**: AOF matched the event to the `github-pr-reviewer` agent
5. **Code Fetch**: The agent fetched the PR diff using GitHub CLI
6. **AI Analysis**: Google Gemini analyzed the code changes
7. **Review Post**: AOF posted the review as a comment on the PR

## Customizing Your Reviews

### Adjust Review Depth

Edit `agents/github-pr-reviewer.yaml`:

```yaml
spec:
  max_iterations: 15  # More iterations = deeper analysis
  temperature: 0.1    # Lower = more focused and deterministic
```

### Focus on Specific Issues

Modify the system prompt to emphasize certain checks:

```yaml
system_prompt: |
  You are a security-focused code reviewer.

  **PRIMARY FOCUS**: Security vulnerabilities
  - SQL injection
  - XSS attacks
  - Authentication bypass
  - Secrets in code

  **SECONDARY**: Performance and best practices
```

### Use Different Models

```yaml
spec:
  # Fast and cheap (Google Gemini Flash)
  model: google:gemini-2.5-flash

  # OR more powerful (OpenAI GPT-4)
  model: openai:gpt-4o

  # OR Claude Sonnet
  model: anthropic:claude-sonnet-4-20250514
```

## Advanced Features

### Auto-Approve Safe PRs

Add a condition to auto-approve PRs that pass all checks:

```yaml
nodes:
  - id: auto-approve
    type: Condition
    condition: |
      ${{ nodes.review.output.assessment == "Approve" &&
          nodes.review.output.critical_issues == 0 }}
    then:
      - type: github_review
        event: APPROVE
        body: "✅ Automated approval: All checks passed!"
```

### Request Changes for Critical Issues

```yaml
nodes:
  - id: request-changes
    type: Condition
    condition: ${{ nodes.review.output.critical_issues > 0 }}
    then:
      - type: github_review
        event: REQUEST_CHANGES
        body: |
          ⚠️ Critical issues found that must be addressed:
          ${{ nodes.review.output.critical_issues_list }}
```

### Add Status Checks

Create GitHub status checks for CI integration:

```yaml
nodes:
  - id: status-check
    type: Action
    action:
      type: github_check_run
      owner: ${{ event.repository.owner.login }}
      repo: ${{ event.repository.name }}
      head_sha: ${{ event.pull_request.head.sha }}
      name: "AOF Code Review"
      status: "completed"
      conclusion: |
        ${{ nodes.review.output.critical_issues > 0 ? "failure" : "success" }}
      output:
        title: "Code Review Complete"
        summary: ${{ nodes.review.output.summary }}
```

## Troubleshooting

### "GitHub enabled but missing webhook_secret"

**Problem**: Environment variable not loaded

**Solution**:
```bash
# Check if variable is set
echo $GITHUB_WEBHOOK_SECRET

# If empty, source your config
source ~/.zshrc  # or ~/.bashrc

# Restart AOF
./target/release/aofctl serve --config config/aof/daemon.yaml
```

### Webhook shows "Invalid signature"

**Problem**: Webhook secret mismatch

**Solution**:
1. Verify secrets match exactly (no extra spaces/newlines)
2. Check `echo $GITHUB_WEBHOOK_SECRET` matches GitHub webhook secret
3. Regenerate secret if needed:
   ```bash
   export GITHUB_WEBHOOK_SECRET=$(openssl rand -hex 32)
   # Update in GitHub webhook configuration
   ```

### Agent not responding

**Check**:
1. **Agent loaded**: Look for "Loaded agent 'github-pr-reviewer'" in logs
2. **Webhook delivered**: Check GitHub webhook Recent Deliveries
3. **Events configured**: Verify webhook listens to "Pull requests"
4. **Logs**: Run with `RUST_LOG=debug` for detailed output

### No comments appearing on PR

**Check**:
1. **GITHUB_TOKEN set**: Verify with `echo $GITHUB_TOKEN`
2. **Token permissions**: Token needs `repo` scope
3. **Repository access**: Token owner has write access to repository
4. **Rate limits**: Check if you've exceeded GitHub API limits

## Next Steps

Now that you have basic PR reviews working, try:

1. **Add Issue Triaging**: Create an agent that auto-labels and assigns issues
2. **Dependency Updates**: Auto-review and approve Dependabot PRs
3. **Release Automation**: Trigger releases when PRs are merged to main
4. **CI/CD Integration**: Link reviews to GitHub Actions workflows
5. **Multi-Repository**: Deploy across all your organization's repositories

## Resources

- [GitHub Platform Documentation](../platforms/github.md)
- [Quick Setup Guide](../../GITHUB_SETUP.md)
- [Agent Configuration Reference](../user-guide/agents/index.md)
- [AgentFlow Documentation](../agentflow/README.md)
- [AOF GitHub Issues](https://github.com/agenticdevops/aof/issues)

## Need Help?

- Check webhook delivery logs in GitHub
- Enable debug logging: `RUST_LOG=debug aofctl serve ...`
- Review AOF documentation: https://docs.aof.sh
- File an issue: https://github.com/agenticdevops/aof/issues

Happy automating! 🚀
