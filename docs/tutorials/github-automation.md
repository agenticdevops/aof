# GitHub Automation with AOF

This tutorial shows you how to build powerful GitHub automation for DevOps, SRE, and Platform Engineering workflows using AOF's trigger system.

## What You'll Build

GitHub-triggered automation that can:
- Auto-review PRs for security, performance, and best practices
- Run automated tests and deployments on PR events
- Triage and label issues automatically
- Trigger deployment pipelines on merge
- Manage release workflows
- Sync across multiple repositories

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- GitHub account with repository access
- Personal Access Token or GitHub App
- A server with public HTTPS endpoint (or ngrok for testing)

## Step 1: Set Up GitHub Authentication

### Option A: Personal Access Token (Quick Start)

1. Go to GitHub Settings > Developer Settings > Personal Access Tokens > Fine-grained tokens
2. Create token with permissions:
   - **Repository**: Read & Write for Contents, Pull requests, Issues, Workflows
   - **Organization**: Read for Members (if using org repos)
3. Save the token securely

### Option B: GitHub App (Recommended for Production)

1. Go to GitHub Settings > Developer Settings > GitHub Apps > New GitHub App
2. Configure:
   - **Name**: `AOF Automation`
   - **Homepage URL**: Your documentation URL
   - **Webhook URL**: `https://your-domain.com/webhooks/github`
   - **Webhook Secret**: Generate a secure random string
   - **Permissions**:
     - Contents: Read & Write
     - Pull requests: Read & Write
     - Issues: Read & Write
     - Checks: Read & Write
     - Workflows: Read & Write
     - Metadata: Read-only
   - **Subscribe to events**:
     - Pull request
     - Push
     - Issues
     - Check run
     - Workflow run
     - Release

3. After creating, generate a Private Key and note the App ID
4. Install the app on your repositories

## Step 2: Configure AOF Trigger Server

Create `config/github-automation.yaml`:

```yaml
version: v1
kind: TriggerConfig

server:
  host: "0.0.0.0"
  port: 8080
  base_path: "/webhooks"

platforms:
  github:
    type: github

    # Authentication (choose one)
    # Option A: Personal Access Token
    token: "${GITHUB_TOKEN}"

    # Option B: GitHub App (preferred)
    # app_id: "${GITHUB_APP_ID}"
    # private_key_path: "/etc/aof/github-app-private-key.pem"
    # installation_id: "${GITHUB_INSTALLATION_ID}"

    # Webhook verification
    webhook_secret: "${GITHUB_WEBHOOK_SECRET}"

    # Bot identity for comments
    bot_name: "aof-bot"

    # Repository filters (empty = all repos app has access to)
    allowed_repos:
      - "myorg/api"
      - "myorg/web"
      - "myorg/infrastructure"

    # Event filters
    allowed_events:
      - "pull_request"
      - "push"
      - "issues"
      - "workflow_run"
      - "check_run"
      - "release"

    # User filters for sensitive operations
    allowed_users:
      - "alice"
      - "bob"
      - "sre-team"

# Event routing
routing:
  default_flow: "github-event-logger"

  # Route by event type and action
  events:
    pull_request:
      opened: "pr-review-flow"
      synchronize: "pr-review-flow"
      closed: "pr-cleanup-flow"

    push:
      # Route by branch
      branches:
        main: "production-deploy-flow"
        develop: "staging-deploy-flow"
        "release/*": "release-deploy-flow"

    issues:
      opened: "issue-triage-flow"
      labeled: "issue-handler-flow"

    workflow_run:
      completed: "workflow-result-handler"

    release:
      published: "release-announce-flow"

flows:
  directory: "./flows/github"
  watch: true
```

### Environment Variables

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
export GITHUB_WEBHOOK_SECRET="$(openssl rand -hex 32)"
# For GitHub App:
export GITHUB_APP_ID="123456"
export GITHUB_INSTALLATION_ID="12345678"
```

## Step 3: Create PR Review Automation

### 3.1 Automated Code Review Flow

Create `flows/github/pr-review-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: pr-review-flow
  description: Automated PR review for security, performance, and best practices

triggers:
  - platform: github
    events:
      - pull_request.opened
      - pull_request.synchronize

input:
  from_event:
    pr_number: "{{ event.pull_request.number }}"
    repo: "{{ event.repository.full_name }}"
    head_sha: "{{ event.pull_request.head.sha }}"
    base_branch: "{{ event.pull_request.base.ref }}"
    author: "{{ event.pull_request.user.login }}"
    files_changed: "{{ event.pull_request.changed_files }}"

# Skip if PR is from bot or draft
conditions:
  - "{{ not event.pull_request.draft }}"
  - "{{ event.pull_request.user.type != 'Bot' }}"

steps:
  # Create initial check run
  - name: create-check
    agent: github
    action: create_check_run
    input:
      repo: "{{ input.repo }}"
      head_sha: "{{ input.head_sha }}"
      name: "AOF Code Review"
      status: "in_progress"
      output:
        title: "Reviewing PR..."
        summary: "Automated code review in progress"

  # Get changed files
  - name: get-files
    agent: github
    action: get_pr_files
    input:
      repo: "{{ input.repo }}"
      pr_number: "{{ input.pr_number }}"

  # Parallel analysis
  - name: analyze
    parallel: true
    steps:
      # Security scan
      - name: security-scan
        agent: security-scanner
        action: scan
        input:
          repo: "{{ input.repo }}"
          ref: "{{ input.head_sha }}"
          files: "{{ steps.get-files.output.files }}"
        checks:
          - type: secrets
            severity: critical
          - type: sql_injection
            severity: high
          - type: xss
            severity: high
          - type: dependencies
            severity: medium

      # Performance analysis
      - name: perf-analysis
        agent: perf-analyzer
        action: analyze
        input:
          files: "{{ steps.get-files.output.files }}"
        checks:
          - type: n_plus_one
          - type: missing_indexes
          - type: large_payloads
          - type: inefficient_loops

      # Code quality
      - name: quality-check
        agent: code-quality
        action: check
        input:
          files: "{{ steps.get-files.output.files }}"
          config:
            max_complexity: 10
            max_file_length: 500
            require_tests: true
            coverage_threshold: 80

      # Kubernetes manifest validation (if applicable)
      - name: k8s-validation
        agent: kubernetes-validator
        condition: "{{ steps.get-files.output.files | selectattr('filename', 'match', '.*\\.ya?ml$') | list | length > 0 }}"
        action: validate
        input:
          files: "{{ steps.get-files.output.files | selectattr('filename', 'match', '.*\\.ya?ml$') | list }}"
        checks:
          - type: schema
          - type: security_context
          - type: resource_limits
          - type: best_practices

  # Aggregate results
  - name: aggregate-results
    agent: review-aggregator
    action: aggregate
    input:
      security: "{{ steps.analyze.security-scan.output }}"
      performance: "{{ steps.analyze.perf-analysis.output }}"
      quality: "{{ steps.analyze.quality-check.output }}"
      k8s: "{{ steps.analyze.k8s-validation.output | default({}) }}"

  # Determine approval status
  - name: determine-status
    agent: decision-maker
    action: evaluate
    input:
      results: "{{ steps.aggregate-results.output }}"
    rules:
      - condition: "{{ results.security.critical_count > 0 }}"
        status: "failure"
        message: "Critical security issues found"
      - condition: "{{ results.security.high_count > 0 }}"
        status: "failure"
        message: "High severity security issues found"
      - condition: "{{ results.quality.coverage < 80 }}"
        status: "failure"
        message: "Test coverage below 80%"
      - condition: "{{ results.performance.issues | length > 5 }}"
        status: "warning"
        message: "Multiple performance issues detected"
      - default:
        status: "success"
        message: "All checks passed"

  # Post review comment
  - name: post-review
    agent: github
    action: post_review
    input:
      repo: "{{ input.repo }}"
      pr_number: "{{ input.pr_number }}"
      commit_id: "{{ input.head_sha }}"
      event: "{{ 'APPROVE' if steps.determine-status.output.status == 'success' else 'REQUEST_CHANGES' }}"
      body: |
        ## 🤖 Automated Code Review

        {{ '✅' if steps.determine-status.output.status == 'success' else '❌' }} **{{ steps.determine-status.output.message }}**

        ### Security Scan
        {{ '✅' if steps.aggregate-results.output.security.passed else '❌' }} {{ steps.aggregate-results.output.security.summary }}
        {% if steps.aggregate-results.output.security.issues %}
        <details>
        <summary>Security Issues ({{ steps.aggregate-results.output.security.issues | length }})</summary>

        {% for issue in steps.aggregate-results.output.security.issues %}
        - **{{ issue.severity }}**: {{ issue.message }} (`{{ issue.file }}:{{ issue.line }}`)
        {% endfor %}
        </details>
        {% endif %}

        ### Performance Analysis
        {{ '✅' if steps.aggregate-results.output.performance.passed else '⚠️' }} {{ steps.aggregate-results.output.performance.summary }}
        {% if steps.aggregate-results.output.performance.issues %}
        <details>
        <summary>Performance Issues ({{ steps.aggregate-results.output.performance.issues | length }})</summary>

        {% for issue in steps.aggregate-results.output.performance.issues %}
        - **{{ issue.type }}**: {{ issue.message }} (`{{ issue.file }}:{{ issue.line }}`)
        {% endfor %}
        </details>
        {% endif %}

        ### Code Quality
        - Complexity: {{ '✅' if steps.aggregate-results.output.quality.complexity_ok else '❌' }} (max: {{ steps.aggregate-results.output.quality.max_complexity }})
        - Test Coverage: {{ '✅' if steps.aggregate-results.output.quality.coverage >= 80 else '❌' }} {{ steps.aggregate-results.output.quality.coverage }}%
        - Lint: {{ '✅' if steps.aggregate-results.output.quality.lint_passed else '❌' }}

        {% if steps.aggregate-results.output.k8s %}
        ### Kubernetes Validation
        {{ '✅' if steps.aggregate-results.output.k8s.passed else '❌' }} {{ steps.aggregate-results.output.k8s.summary }}
        {% endif %}

        ---
        <sub>🤖 Review by [AOF](https://docs.aof.sh) | [Re-run review]({{ trigger.event.pull_request.html_url }}/checks)</sub>
      comments: "{{ steps.aggregate-results.output.inline_comments }}"

  # Update check run
  - name: update-check
    agent: github
    action: update_check_run
    input:
      repo: "{{ input.repo }}"
      check_run_id: "{{ steps.create-check.output.id }}"
      conclusion: "{{ steps.determine-status.output.status }}"
      output:
        title: "{{ steps.determine-status.output.message }}"
        summary: |
          **Security**: {{ steps.aggregate-results.output.security.summary }}
          **Performance**: {{ steps.aggregate-results.output.performance.summary }}
          **Quality**: Coverage {{ steps.aggregate-results.output.quality.coverage }}%

  # Add labels based on content
  - name: add-labels
    agent: github
    action: add_labels
    input:
      repo: "{{ input.repo }}"
      issue_number: "{{ input.pr_number }}"
      labels: "{{ steps.aggregate-results.output.suggested_labels }}"
```

### 3.2 PR Size & Risk Labels

Create `flows/github/pr-labeler-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: pr-labeler
  description: Auto-label PRs by size and risk

triggers:
  - platform: github
    events:
      - pull_request.opened
      - pull_request.synchronize

steps:
  - name: analyze-size
    agent: pr-analyzer
    action: analyze_size
    input:
      additions: "{{ event.pull_request.additions }}"
      deletions: "{{ event.pull_request.deletions }}"
      files_changed: "{{ event.pull_request.changed_files }}"
    rules:
      - condition: "{{ additions + deletions < 50 }}"
        label: "size/XS"
      - condition: "{{ additions + deletions < 200 }}"
        label: "size/S"
      - condition: "{{ additions + deletions < 500 }}"
        label: "size/M"
      - condition: "{{ additions + deletions < 1000 }}"
        label: "size/L"
      - default:
        label: "size/XL"

  - name: analyze-risk
    agent: pr-analyzer
    action: analyze_risk
    input:
      files: "{{ event.pull_request.files }}"
    rules:
      # High risk areas
      - pattern: "^.*/(auth|security|crypto)/"
        label: "risk/high"
        requires_review_from: ["security-team"]
      - pattern: "^.*/migrations/"
        label: "risk/high"
        requires_review_from: ["dba-team"]
      - pattern: "^Dockerfile|docker-compose|k8s/"
        label: "infrastructure"
        requires_review_from: ["platform-team"]
      # Documentation
      - pattern: "^docs/|README|CHANGELOG"
        label: "documentation"
      # Tests
      - pattern: "^.*_test\\.go|^.*\\.test\\.(js|ts)|^test/"
        label: "tests"

  - name: apply-labels
    agent: github
    action: add_labels
    input:
      repo: "{{ event.repository.full_name }}"
      issue_number: "{{ event.pull_request.number }}"
      labels:
        - "{{ steps.analyze-size.output.label }}"
        - "{{ steps.analyze-risk.output.labels }}"

  - name: request-reviewers
    condition: "{{ steps.analyze-risk.output.required_reviewers | length > 0 }}"
    agent: github
    action: request_reviewers
    input:
      repo: "{{ event.repository.full_name }}"
      pr_number: "{{ event.pull_request.number }}"
      teams: "{{ steps.analyze-risk.output.required_reviewers }}"
```

## Step 4: Deployment Automation

### 4.1 Production Deploy on Merge

Create `flows/github/production-deploy-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: production-deploy
  description: Deploy to production on merge to main

triggers:
  - platform: github
    events:
      - push
    branches:
      - main

conditions:
  # Only run if this is a merge (not a direct push)
  - "{{ event.head_commit.message | regex_search('Merge pull request') }}"

input:
  from_event:
    repo: "{{ event.repository.full_name }}"
    commit_sha: "{{ event.head_commit.id }}"
    commit_message: "{{ event.head_commit.message }}"
    author: "{{ event.head_commit.author.username }}"

steps:
  # Extract PR number from merge commit
  - name: get-pr-info
    agent: parser
    action: extract
    input:
      text: "{{ input.commit_message }}"
      pattern: "Merge pull request #(\\d+)"
    output:
      pr_number: "{{ match.1 }}"

  # Get deployment info from PR labels
  - name: get-pr
    agent: github
    action: get_pull_request
    input:
      repo: "{{ input.repo }}"
      pr_number: "{{ steps.get-pr-info.output.pr_number }}"

  # Check if deployment is allowed
  - name: check-deployment-gate
    agent: deployment-gate
    action: check
    input:
      repo: "{{ input.repo }}"
      environment: "production"
      commit_sha: "{{ input.commit_sha }}"
    checks:
      - type: all_checks_passed
        required: true
      - type: required_reviewers
        count: 2
      - type: no_blocking_labels
        labels: ["do-not-merge", "wip", "blocked"]
      - type: deployment_window
        allowed_hours: "06:00-18:00"
        allowed_days: ["monday", "tuesday", "wednesday", "thursday"]
        timezone: "America/New_York"

  # Create deployment
  - name: create-deployment
    agent: github
    action: create_deployment
    input:
      repo: "{{ input.repo }}"
      ref: "{{ input.commit_sha }}"
      environment: "production"
      auto_merge: false
      required_contexts: []
      description: "Deploying from PR #{{ steps.get-pr-info.output.pr_number }}"

  # Update deployment status
  - name: start-deployment
    agent: github
    action: create_deployment_status
    input:
      repo: "{{ input.repo }}"
      deployment_id: "{{ steps.create-deployment.output.id }}"
      state: "in_progress"
      description: "Deployment started"

  # Run actual deployment
  - name: deploy
    agent: kubernetes-deployer
    action: deploy
    input:
      cluster: "production"
      namespace: "{{ input.repo | split('/') | last }}"
      image: "ghcr.io/{{ input.repo }}:{{ input.commit_sha | truncate(7) }}"
      strategy: "rolling"
      health_check:
        path: "/health"
        timeout: 300
    progress:
      callback: "deployment-progress-flow"
      interval: 30

  # Update deployment status
  - name: complete-deployment
    agent: github
    action: create_deployment_status
    input:
      repo: "{{ input.repo }}"
      deployment_id: "{{ steps.create-deployment.output.id }}"
      state: "success"
      environment_url: "https://{{ input.repo | split('/') | last }}.example.com"
      description: "Deployed successfully"

  # Post deployment comment
  - name: post-comment
    agent: github
    action: post_comment
    input:
      repo: "{{ input.repo }}"
      issue_number: "{{ steps.get-pr-info.output.pr_number }}"
      body: |
        ## 🚀 Deployed to Production

        | | |
        |---|---|
        | **Commit** | [`{{ input.commit_sha | truncate(7) }}`](https://github.com/{{ input.repo }}/commit/{{ input.commit_sha }}) |
        | **Environment** | [production](https://{{ input.repo | split('/') | last }}.example.com) |
        | **Deployed by** | @{{ input.author }} |
        | **Time** | {{ now() | format_time }} |

        ### Verification Links
        - [Application](https://{{ input.repo | split('/') | last }}.example.com)
        - [Logs](https://logs.example.com/{{ input.repo | split('/') | last }}/production)
        - [Metrics](https://grafana.example.com/d/{{ input.repo | split('/') | last }})

  # Notify Slack
  - name: notify-slack
    agent: slack
    action: send
    input:
      channel: "#deployments"
      text: |
        :rocket: *{{ input.repo }}* deployed to production
        Commit: {{ input.commit_sha | truncate(7) }}
        Author: {{ input.author }}
        <https://{{ input.repo | split('/') | last }}.example.com|View Application>

on_error:
  - name: fail-deployment
    agent: github
    action: create_deployment_status
    input:
      repo: "{{ input.repo }}"
      deployment_id: "{{ steps.create-deployment.output.id }}"
      state: "failure"
      description: "{{ error.message }}"

  - name: notify-failure
    agent: multi-channel
    action: notify
    input:
      channels: ["slack:#deployments", "pagerduty"]
      message: |
        ❌ Production deployment failed
        Repo: {{ input.repo }}
        Commit: {{ input.commit_sha }}
        Error: {{ error.message }}
```

### 4.2 Staging Deploy on PR

Create `flows/github/staging-deploy-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: staging-deploy
  description: Deploy preview environment for PRs

triggers:
  - platform: github
    events:
      - pull_request.opened
      - pull_request.synchronize

conditions:
  - "{{ not event.pull_request.draft }}"
  - "{{ 'no-preview' not in event.pull_request.labels | map(attribute='name') }}"

steps:
  # Create deployment
  - name: create-deployment
    agent: github
    action: create_deployment
    input:
      repo: "{{ event.repository.full_name }}"
      ref: "{{ event.pull_request.head.sha }}"
      environment: "preview-pr-{{ event.pull_request.number }}"
      transient_environment: true
      production_environment: false

  # Deploy to preview namespace
  - name: deploy-preview
    agent: kubernetes-deployer
    action: deploy
    input:
      cluster: "staging"
      namespace: "preview-{{ event.pull_request.number }}"
      create_namespace: true
      image: "ghcr.io/{{ event.repository.full_name }}:pr-{{ event.pull_request.number }}"
      resources:
        requests:
          cpu: "100m"
          memory: "256Mi"
        limits:
          cpu: "500m"
          memory: "512Mi"
      ttl: "72h"  # Auto-delete after 72 hours

  # Set up preview URL
  - name: create-ingress
    agent: kubernetes-deployer
    action: create_ingress
    input:
      cluster: "staging"
      namespace: "preview-{{ event.pull_request.number }}"
      host: "pr-{{ event.pull_request.number }}.preview.example.com"

  # Update deployment status
  - name: update-status
    agent: github
    action: create_deployment_status
    input:
      repo: "{{ event.repository.full_name }}"
      deployment_id: "{{ steps.create-deployment.output.id }}"
      state: "success"
      environment_url: "https://pr-{{ event.pull_request.number }}.preview.example.com"

  # Post comment with preview URL
  - name: post-comment
    agent: github
    action: post_comment
    input:
      repo: "{{ event.repository.full_name }}"
      issue_number: "{{ event.pull_request.number }}"
      body: |
        ## 🔍 Preview Environment Ready

        | Environment | URL |
        |-------------|-----|
        | Preview | https://pr-{{ event.pull_request.number }}.preview.example.com |

        This preview will be automatically deleted 72 hours after the PR is closed.

        ---
        <sub>🤖 [AOF Preview Deployments](https://docs.aof.sh)</sub>
```

## Step 5: Issue Management

### 5.1 Auto-Triage Flow

Create `flows/github/issue-triage-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: issue-triage
  description: Automatically triage new issues

triggers:
  - platform: github
    events:
      - issues.opened

steps:
  # Analyze issue content
  - name: analyze-issue
    agent: issue-analyzer
    action: analyze
    input:
      title: "{{ event.issue.title }}"
      body: "{{ event.issue.body }}"
    rules:
      # Bug detection
      - patterns: ["bug", "error", "crash", "not working", "broken"]
        label: "bug"
        priority: "high"
      # Feature request
      - patterns: ["feature", "enhancement", "request", "would be nice", "suggestion"]
        label: "enhancement"
        priority: "medium"
      # Question
      - patterns: ["how to", "question", "help", "?"]
        label: "question"
        priority: "low"
      # Security
      - patterns: ["security", "vulnerability", "cve", "exploit"]
        label: "security"
        priority: "critical"
        notify: "security-team"

  # Determine component
  - name: determine-component
    agent: classifier
    action: classify
    input:
      text: "{{ event.issue.title }} {{ event.issue.body }}"
    categories:
      - name: "api"
        patterns: ["api", "endpoint", "rest", "graphql"]
      - name: "frontend"
        patterns: ["ui", "frontend", "css", "react", "button", "page"]
      - name: "database"
        patterns: ["database", "db", "postgres", "migration", "query"]
      - name: "infrastructure"
        patterns: ["kubernetes", "k8s", "docker", "deployment", "ci/cd"]
      - name: "documentation"
        patterns: ["docs", "readme", "documentation", "example"]

  # Apply labels
  - name: apply-labels
    agent: github
    action: add_labels
    input:
      repo: "{{ event.repository.full_name }}"
      issue_number: "{{ event.issue.number }}"
      labels:
        - "{{ steps.analyze-issue.output.label }}"
        - "priority/{{ steps.analyze-issue.output.priority }}"
        - "component/{{ steps.determine-component.output.category }}"
        - "needs-triage"

  # Assign to team
  - name: assign-team
    agent: github
    action: add_assignees
    input:
      repo: "{{ event.repository.full_name }}"
      issue_number: "{{ event.issue.number }}"
      assignees: "{{ team_mapping[steps.determine-component.output.category] }}"
    variables:
      team_mapping:
        api: ["backend-team"]
        frontend: ["frontend-team"]
        database: ["dba-team"]
        infrastructure: ["platform-team"]
        documentation: ["docs-team"]

  # Post welcome comment
  - name: welcome-comment
    agent: github
    action: post_comment
    input:
      repo: "{{ event.repository.full_name }}"
      issue_number: "{{ event.issue.number }}"
      body: |
        Thanks for opening this issue, @{{ event.issue.user.login }}! 👋

        I've automatically classified this as:
        - **Type**: {{ steps.analyze-issue.output.label }}
        - **Priority**: {{ steps.analyze-issue.output.priority }}
        - **Component**: {{ steps.determine-component.output.category }}

        {% if steps.analyze-issue.output.label == 'bug' %}
        To help us investigate, please ensure you've provided:
        - [ ] Steps to reproduce
        - [ ] Expected behavior
        - [ ] Actual behavior
        - [ ] Environment details (OS, version, etc.)
        {% elif steps.analyze-issue.output.label == 'enhancement' %}
        To help us understand your request:
        - [ ] Use case / problem being solved
        - [ ] Proposed solution
        - [ ] Alternatives considered
        {% endif %}

        A team member will review this shortly.

  # Notify on critical issues
  - name: notify-critical
    condition: "{{ steps.analyze-issue.output.priority == 'critical' }}"
    agent: multi-channel
    action: notify
    input:
      channels:
        - "slack:#security-alerts"
        - "pagerduty:security-team"
      message: |
        🚨 Critical security issue opened

        Repo: {{ event.repository.full_name }}
        Issue: #{{ event.issue.number }} - {{ event.issue.title }}
        Author: @{{ event.issue.user.login }}

        {{ event.issue.html_url }}
```

## Step 6: Release Management

### 6.1 Automated Release Flow

Create `flows/github/release-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: release-automation
  description: Automate release creation and announcements

triggers:
  - platform: github
    events:
      - release.published

steps:
  # Parse release info
  - name: parse-release
    agent: parser
    action: parse
    input:
      tag: "{{ event.release.tag_name }}"
      body: "{{ event.release.body }}"
    extract:
      version: "{{ tag | regex_replace('^v', '') }}"
      is_prerelease: "{{ event.release.prerelease }}"
      changelog: "{{ body }}"

  # Build and push container images
  - name: build-images
    agent: docker-builder
    action: build_and_push
    input:
      repo: "{{ event.repository.full_name }}"
      tag: "{{ event.release.tag_name }}"
      platforms: ["linux/amd64", "linux/arm64"]
      registries:
        - "ghcr.io/{{ event.repository.full_name }}"
        - "docker.io/{{ event.repository.name }}"

  # Update Helm chart
  - name: update-helm
    agent: helm-manager
    action: update_chart
    input:
      repo: "{{ event.repository.owner.login }}/helm-charts"
      chart: "{{ event.repository.name }}"
      app_version: "{{ steps.parse-release.output.version }}"
      values:
        image:
          tag: "{{ event.release.tag_name }}"

  # Update documentation
  - name: update-docs
    agent: docs-manager
    action: update_version
    input:
      repo: "{{ event.repository.owner.login }}/docs"
      product: "{{ event.repository.name }}"
      version: "{{ steps.parse-release.output.version }}"
      changelog: "{{ steps.parse-release.output.changelog }}"

  # Notify channels
  - name: announce-release
    agent: multi-channel
    action: announce
    parallel: true
    input:
      channels:
        - type: slack
          channel: "#releases"
          message:
            blocks:
              - type: header
                text: "🎉 {{ event.repository.name }} {{ event.release.tag_name }} Released"
              - type: section
                text: "{{ steps.parse-release.output.changelog | truncate(500) }}"
              - type: actions
                elements:
                  - type: button
                    text: "View Release"
                    url: "{{ event.release.html_url }}"
                  - type: button
                    text: "Changelog"
                    url: "{{ event.repository.html_url }}/blob/main/CHANGELOG.md"

        - type: twitter
          condition: "{{ not steps.parse-release.output.is_prerelease }}"
          message: |
            🚀 {{ event.repository.name }} {{ event.release.tag_name }} is out!

            {{ steps.parse-release.output.changelog | summarize(200) }}

            {{ event.release.html_url }}

        - type: discord
          channel: "releases"
          embed:
            title: "{{ event.repository.name }} {{ event.release.tag_name }}"
            description: "{{ steps.parse-release.output.changelog }}"
            url: "{{ event.release.html_url }}"
            color: 0x00ff00

  # Auto-deploy to staging
  - name: deploy-staging
    condition: "{{ not steps.parse-release.output.is_prerelease }}"
    agent: kubernetes-deployer
    action: deploy
    input:
      cluster: "staging"
      namespace: "{{ event.repository.name }}"
      image: "ghcr.io/{{ event.repository.full_name }}:{{ event.release.tag_name }}"
```

## Step 7: Multi-Repo Synchronization

### 7.1 Dependency Update Flow

Create `flows/github/dependency-sync-flow.yaml`:

```yaml
apiVersion: aof.sh/v1alpha1
kind: AgentFlow
metadata:
  name: dependency-sync
  description: Sync dependency updates across repos

triggers:
  - platform: github
    events:
      - release.published
    repos:
      - "myorg/shared-lib"  # When shared lib releases

steps:
  # Find dependent repos
  - name: find-dependents
    agent: dependency-scanner
    action: find_dependents
    input:
      package: "{{ event.repository.name }}"
      version: "{{ event.release.tag_name }}"
      registries:
        - type: npm
          scope: "@myorg"
        - type: go
          module: "github.com/{{ event.repository.full_name }}"

  # Create PRs in each repo
  - name: update-dependents
    agent: pr-creator
    action: create_update_pr
    foreach: "{{ steps.find-dependents.output.repos }}"
    input:
      repo: "{{ item.repo }}"
      branch: "deps/update-{{ event.repository.name }}-{{ event.release.tag_name }}"
      updates:
        - file: "{{ item.manifest_file }}"
          package: "{{ event.repository.name }}"
          from_version: "{{ item.current_version }}"
          to_version: "{{ event.release.tag_name }}"
      pr:
        title: "chore(deps): update {{ event.repository.name }} to {{ event.release.tag_name }}"
        body: |
          ## Dependency Update

          Updates `{{ event.repository.name }}` from `{{ item.current_version }}` to `{{ event.release.tag_name }}`.

          ### Changelog
          {{ event.release.body }}

          ### Release
          {{ event.release.html_url }}

          ---
          <sub>🤖 Automated by [AOF Dependency Sync](https://docs.aof.sh)</sub>
        labels:
          - "dependencies"
          - "automated"
        auto_merge: "{{ item.auto_merge_allowed }}"
```

## Step 8: Set Up Webhook

### 8.1 Start Server

```bash
aofctl trigger serve --config config/github-automation.yaml
```

### 8.2 Register Webhook

**For repository webhooks:**
1. Go to Repository Settings > Webhooks > Add webhook
2. Payload URL: `https://your-domain.com/webhooks/github`
3. Content type: `application/json`
4. Secret: Same as `GITHUB_WEBHOOK_SECRET`
5. Events: Select specific events or "Send me everything"

**For organization webhooks:**
1. Go to Organization Settings > Webhooks
2. Same configuration as above

### 8.3 Verify Setup

```bash
# Check webhook deliveries in GitHub
# Settings > Webhooks > Recent Deliveries

# Check server logs
tail -f /var/log/aof/trigger-server.log
```

## Best Practices

### Security

```yaml
platforms:
  github:
    security:
      # Always verify webhook signatures
      verify_signatures: true

      # Require specific user permissions for sensitive actions
      required_permissions:
        deploy: ["admin", "maintain"]
        approve: ["write"]
        merge: ["write"]

      # IP allowlist (GitHub webhook IPs)
      allowed_ips:
        - "140.82.112.0/20"
        - "185.199.108.0/22"
```

### Rate Limiting

```yaml
platforms:
  github:
    rate_limits:
      # GitHub API limits
      requests_per_hour: 5000

      # Retry on rate limit
      retry:
        enabled: true
        max_attempts: 3
        backoff: exponential
```

### Error Handling

```yaml
on_error:
  # Retry transient failures
  retry:
    conditions:
      - "{{ error.status in [502, 503, 504] }}"
      - "{{ 'rate limit' in error.message }}"
    max_attempts: 3
    backoff: exponential

  # Notify on persistent failures
  notify:
    channels: ["slack:#github-automation-alerts"]

  # Log for debugging
  log:
    level: error
    include_context: true
```

## Troubleshooting

### Webhook Not Receiving Events

1. Check webhook delivery status in GitHub
2. Verify webhook secret matches
3. Check firewall/network allows GitHub IPs
4. Verify server is running and accessible

### Check Run Not Updating

1. Verify GitHub App has `checks:write` permission
2. Check check run ID is correct
3. Verify conclusion is valid value

### Deployment Failing

1. Check Kubernetes credentials
2. Verify image exists in registry
3. Check namespace permissions
4. Review deployment logs

## Next Steps

- [Telegram Bot Tutorial](./telegram-ops-bot.md)
- [WhatsApp Bot Tutorial](./whatsapp-ops-bot.md)
- [Multi-Platform Routing](./multi-platform-routing.md)
- [Building Custom Triggers](../developer/BUILDING_TRIGGERS.md)
