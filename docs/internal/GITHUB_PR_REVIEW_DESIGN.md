# GitHub PR Review Integration - Internal Design Document

**Status**: Design Proposal
**Version**: 1.0
**Last Updated**: 2025-01-20
**Owner**: AOF Core Team

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Design](#component-design)
4. [Data Flow](#data-flow)
5. [Configuration Examples](#configuration-examples)
6. [Extension Points](#extension-points)
7. [Security Considerations](#security-considerations)
8. [Implementation Phases](#implementation-phases)
9. [Testing Strategy](#testing-strategy)
10. [Future Enhancements](#future-enhancements)

---

## Overview

### Purpose

This document describes the internal architecture for GitHub PR Review Integration in AOF. The feature enables automated, multi-agent code review workflows triggered by GitHub pull request events, with consensus-based decision making and automated PR feedback through GitHub's API.

### Goals

1. **Automated PR Review**: Trigger specialized review agents on PR events (opened, synchronized, review_requested)
2. **Multi-Agent Analysis**: Coordinate security, performance, and quality reviewers in parallel (Fleet coordination)
3. **Consensus-Based Decisions**: Aggregate agent feedback using weighted consensus algorithms
4. **GitHub API Integration**: Post reviews, comments, labels, and status checks back to GitHub
5. **Flexible Workflows**: Support both simple agent-based reviews and complex multi-step workflows

### Non-Goals

- Full GitHub Actions replacement (focus on PR review only)
- Branch protection enforcement (use GitHub's native features)
- Merge automation (use GitHub auto-merge or external tools)

---

## Architecture

### High-Level Flow

```
GitHub Webhook → AOF Daemon → Trigger Router → Fleet/Workflow → GitHub API
                    ↓                             ↓
              Signature Verify              Agent Execution
                    ↓                             ↓
              Parse PR Event              Consensus + Results
                    ↓                             ↓
              Match Trigger               Post Review/Labels
```

### Key Components

| Component | Module | Responsibility |
|-----------|--------|----------------|
| **GitHub Platform** | `aof-triggers::platforms::github` | Webhook parsing, signature verification, API methods |
| **Trigger System** | `aof-triggers` | Event routing, command matching, trigger lifecycle |
| **Fleet Coordinator** | `aof-runtime::fleet` | Multi-agent orchestration, consensus, result aggregation |
| **Workflow Engine** | `aof-runtime::workflow` | Multi-step PR review workflows with approval gates |
| **GitHub API Client** | `aof-triggers::platforms::github` | REST API calls (reviews, comments, checks, labels) |

### Integration Points

```rust
// GitHub Platform → Trigger Router
TriggerMessage {
    platform: "github",
    event_type: "pull_request.opened",
    metadata: {
        pr_number, pr_title, pr_diff, pr_files, pr_head_sha, ...
    },
    ...
}

// Trigger Router → Fleet/Workflow
ExecutionContext {
    trigger_message: TriggerMessage,
    state: {
        pr_number, repo, pr_diff, changed_files, ...
    },
    ...
}

// Fleet/Workflow → GitHub API
GitHubPlatform::post_review(owner, repo, pr_number, body, event)
GitHubPlatform::add_labels(owner, repo, pr_number, labels)
GitHubPlatform::create_check_run(owner, repo, head_sha, ...)
```

---

## Component Design

### 1. GitHub Trigger Configuration

Trigger resources define GitHub webhook endpoints and command routing.

**Location**: `examples/triggers/github-pr-review.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-pr-review
  labels:
    platform: github
    purpose: pr-review

spec:
  type: GitHub
  config:
    # Authentication & Security
    token_env: GITHUB_TOKEN                 # GitHub PAT or App token
    webhook_secret_env: GITHUB_WEBHOOK_SECRET

    # Event Filters
    events:
      - pull_request.opened
      - pull_request.synchronize
      - pull_request.review_requested

    repositories:
      - agenticdevops/aof                   # Specific repos
      - agenticdevops/*                     # Org wildcard

    # Optional: Branch filters
    branches:
      - main
      - "release/*"

    # Optional: User filters (ignore bot PRs)
    allowed_users:
      - "*"
    excluded_users:
      - dependabot[bot]
      - renovate[bot]

    # Feature Flags
    enable_reviews: true
    enable_comments: true
    enable_status_checks: true

  # Command Routing (for /commands in PR comments)
  commands:
    /review:
      fleet: pr-review-fleet
      description: "AI code review with consensus"

    /review-security:
      agent: security-reviewer
      description: "Security-focused review"

    /review-full:
      workflow: pr-review-workflow
      description: "Full workflow with approval gate"

  # Default handler for PR events without commands
  default_fleet: pr-review-fleet
```

**Key Features**:
- **Event Filtering**: Only respond to specific PR events
- **Repository Scoping**: Limit to specific repos/orgs
- **Security**: Webhook signature verification (HMAC-SHA256)
- **Command Routing**: Support both fleets and workflows
- **Default Handler**: Automatic review on PR open/sync

### 2. PR Review Fleet

Multi-agent fleet with specialized reviewers and weighted consensus.

**Location**: `examples/fleets/pr-review-fleet.yaml`

```yaml
apiVersion: aof.dev/v1
kind: AgentFleet
metadata:
  name: pr-review-fleet
  labels:
    purpose: code-review
    platform: github

spec:
  agents:
    # Security Specialist (higher weight)
    - name: security-reviewer
      role: specialist
      tier: 1
      weight: 2.0                    # Security concerns are critical
      spec:
        model: anthropic:claude-sonnet-4-20250514
        instructions: |
          You are a security specialist reviewing code for vulnerabilities.
          Focus on:
          - SQL injection, XSS, CSRF
          - Authentication/authorization flaws
          - Secrets in code
          - Dependency vulnerabilities

          Analyze PR diff and provide:
          1. List of security issues (if any)
          2. Severity rating (CRITICAL/HIGH/MEDIUM/LOW)
          3. Recommended fixes
          4. Overall approval decision: APPROVE or REQUEST_CHANGES

        tools:
          - read_file        # Read changed files
          - shell            # Run security scanners

        max_iterations: 3
        temperature: 0.3

    # Performance Specialist
    - name: performance-reviewer
      role: specialist
      tier: 1
      weight: 1.0
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a performance specialist reviewing code for efficiency.
          Focus on:
          - Algorithm complexity (O(n²) → O(n log n))
          - Database query optimization (N+1 queries)
          - Memory leaks
          - Inefficient loops

          Analyze PR diff and provide:
          1. List of performance issues (if any)
          2. Impact rating (HIGH/MEDIUM/LOW)
          3. Optimization suggestions
          4. Overall approval decision: APPROVE or REQUEST_CHANGES

        tools:
          - read_file
          - shell

        max_iterations: 3
        temperature: 0.4

    # Code Quality Specialist
    - name: quality-reviewer
      role: specialist
      tier: 1
      weight: 1.0
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a code quality specialist reviewing for maintainability.
          Focus on:
          - SOLID principles
          - Code readability
          - Error handling
          - Test coverage
          - Documentation

          Analyze PR diff and provide:
          1. List of quality issues (if any)
          2. Priority rating (HIGH/MEDIUM/LOW)
          3. Improvement suggestions
          4. Overall approval decision: APPROVE or REQUEST_CHANGES

        tools:
          - read_file
          - shell

        max_iterations: 3
        temperature: 0.5

  coordination:
    mode: peer                       # All agents run in parallel
    distribution: round-robin

    consensus:
      algorithm: weighted            # Use agent weights
      min_votes: 2                   # At least 2 agents must respond
      timeout_ms: 120000             # 2 minute timeout
      allow_partial: true            # Accept if some agents fail
      min_confidence: 0.6            # Flag for human review if below 60%

      # Per-agent weights (can also be set in agent.weight)
      weights:
        security-reviewer: 2.0
        performance-reviewer: 1.0
        quality-reviewer: 1.0

  shared:
    memory:
      type: inmemory
      namespace: pr-review
      ttl: 3600                      # 1 hour cache

  communication:
    pattern: broadcast               # All agents see each other's findings
    broadcast:
      channel: review-findings
      include_sender: false
```

**Key Features**:
- **Parallel Execution**: All reviewers run simultaneously
- **Weighted Consensus**: Security reviewer has 2x voting power
- **Confidence Threshold**: Low-confidence results flagged for human review
- **Shared Context**: Agents can see each other's findings via broadcast

### 3. PR Review Workflow

Multi-step workflow with conditional routing and approval gates.

**Location**: `examples/workflows/pr-review-workflow.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Workflow
metadata:
  name: pr-review-workflow
  labels:
    platform: github
    purpose: pr-review-lifecycle

spec:
  entrypoint: review

  # State schema
  state:
    type: object
    properties:
      pr_number:
        type: integer
      repo:
        type: string
      pr_diff:
        type: string
      review_results:
        type: array
      consensus_decision:
        type: string
        enum: [APPROVE, REQUEST_CHANGES, COMMENT]
      confidence_score:
        type: number
    required: [pr_number, repo]

  steps:
    # Step 1: Run multi-agent review
    - name: review
      type: agent
      agent: pr-review-fleet          # Execute entire fleet

      next:
        # Route based on consensus decision
        - condition: "state.consensus_decision == 'APPROVE' && state.confidence_score >= 0.8"
          target: auto-approve

        - condition: "state.consensus_decision == 'REQUEST_CHANGES'"
          target: post-feedback

        - condition: "state.confidence_score < 0.6"
          target: human-review-gate

        - target: post-comment           # Default: post as comment

    # Step 2: Auto-approve high-confidence approvals
    - name: auto-approve
      type: agent
      agent: github-ops-agent          # Posts approval review
      config:
        prompt: |
          Post GitHub review with event=APPROVE.
          Summary: {state.review_results}

      next: add-approved-label

    # Step 3: Post requested changes
    - name: post-feedback
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Post GitHub review with event=REQUEST_CHANGES.
          Include all issues from {state.review_results}.

      next: add-changes-label

    # Step 4: Human review gate for low confidence
    - name: human-review-gate
      type: approval
      config:
        approvers:
          - role: senior-engineer
          - role: tech-lead

        timeout: 24h
        required_approvals: 1

        auto_approve:
          condition: "state.confidence_score >= 0.7"  # Skip if borderline

      next:
        - condition: approved
          target: post-comment
        - condition: rejected
          target: post-feedback

    # Step 5: Post as informational comment
    - name: post-comment
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Post GitHub review with event=COMMENT.
          Provide suggestions from {state.review_results}.

      next: add-reviewed-label

    # Step 6: Label management
    - name: add-approved-label
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Add labels: ["approved", "ready-to-merge"]
          Remove labels: ["needs-review"]
      next: create-success-check

    - name: add-changes-label
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Add labels: ["changes-requested"]
          Remove labels: ["needs-review", "approved"]
      next: create-failure-check

    - name: add-reviewed-label
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Add labels: ["reviewed"]
          Remove labels: ["needs-review"]
      next: success

    # Step 7: Create GitHub check runs
    - name: create-success-check
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Create check run:
          - name: "AI Code Review"
          - status: completed
          - conclusion: success
          - output: {state.review_results}
      next: success

    - name: create-failure-check
      type: agent
      agent: github-ops-agent
      config:
        prompt: |
          Create check run:
          - name: "AI Code Review"
          - status: completed
          - conclusion: failure
          - output: {state.review_results}
      next: success

    # Terminal states
    - name: success
      type: terminal
      status: completed

  # Global error handling
  error_handler: handle-error

  # Retry configuration
  retry:
    max_attempts: 3
    backoff: exponential
    initial_delay: 1s
    max_delay: 30s
```

**Key Features**:
- **Conditional Routing**: Different paths based on consensus decision and confidence
- **Human-in-the-Loop**: Approval gate for low-confidence reviews
- **Label Management**: Automated PR labeling based on review outcome
- **Status Checks**: Create GitHub check runs visible in PR UI
- **Error Handling**: Global retry policy with exponential backoff

---

## Data Flow

### 1. Webhook Ingestion

```
GitHub → POST /webhook/github → AOF Daemon
                                    ↓
                            Signature Verification
                                    ↓
                        Parse Webhook Payload
                                    ↓
                        Build TriggerMessage
```

**Implementation**: `GitHubPlatform::parse_message()`

```rust
// Signature verification
fn verify_github_signature(&self, payload: &[u8], signature: &str) -> bool {
    // HMAC-SHA256 verification
    let mut mac = HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes())?;
    mac.update(payload);
    let computed = hex::encode(mac.finalize().into_bytes());

    signature.strip_prefix("sha256=") == Some(&computed)
}

// Parse PR event
async fn parse_message(&self, raw: &[u8], headers: &HashMap<String, String>)
    -> Result<TriggerMessage, PlatformError>
{
    // 1. Verify signature
    let signature = headers.get("x-hub-signature-256")?;
    if !self.verify_github_signature(raw, signature) {
        return Err(PlatformError::InvalidSignature);
    }

    // 2. Check event type
    let event_type = headers.get("x-github-event")?;

    // 3. Parse payload
    let payload: GitHubWebhookPayload = serde_json::from_slice(raw)?;

    // 4. Build trigger message
    self.build_trigger_message(event_type, payload.action, &payload)
}
```

**Output**: `TriggerMessage`

```json
{
  "id": "gh-789-pull_request-pr-123",
  "platform": "github",
  "channel_id": "agenticdevops/aof",
  "user": {
    "id": "456",
    "username": "contributor"
  },
  "text": "pr:opened:main:feature-branch #42 Add GitHub PR review - Initial implementation",
  "timestamp": "2025-01-20T10:00:00Z",
  "metadata": {
    "event_type": "pull_request",
    "action": "opened",
    "pr_number": 42,
    "pr_title": "Add GitHub PR review",
    "pr_base_ref": "main",
    "pr_head_ref": "feature-branch",
    "pr_head_sha": "abc123def456",
    "pr_additions": 250,
    "pr_deletions": 50,
    "pr_changed_files": 8,
    "pr_html_url": "https://github.com/agenticdevops/aof/pull/42",
    "repo_full_name": "agenticdevops/aof"
  },
  "thread_id": "pr-42"
}
```

### 2. Trigger Matching & Routing

```
TriggerMessage → FlowRouter::find_match()
                      ↓
              Evaluate Trigger Filters
                      ↓
              Match Command Binding
                      ↓
              Route to Fleet/Workflow
```

**Implementation**: `FlowRouter` uses `FlowRegistry`

```rust
// Match trigger to handler
pub async fn find_match(&self, message: &TriggerMessage) -> Option<FlowMatch> {
    // 1. Filter triggers by platform
    let candidates = self.registry.triggers
        .iter()
        .filter(|t| t.platform == message.platform);

    // 2. Apply filters (events, repos, branches)
    let filtered = candidates.filter(|t| {
        self.matches_event_filter(t, &message.metadata)
            && self.matches_repo_filter(t, &message.metadata)
            && self.matches_branch_filter(t, &message.metadata)
    });

    // 3. Extract command from message (if any)
    let command = self.extract_command(&message.text);

    // 4. Route to command binding or default handler
    if let Some(cmd) = command {
        trigger.commands.get(&cmd).map(|binding| FlowMatch {
            trigger: trigger.clone(),
            command: Some(cmd),
            target: binding.clone(),
            confidence: 1.0,
        })
    } else {
        // Use default_agent/fleet for automatic PR events
        trigger.default_handler.map(|target| FlowMatch {
            trigger: trigger.clone(),
            command: None,
            target,
            confidence: 0.8,
        })
    }
}
```

### 3. Fleet Execution (Parallel Review)

```
FleetMatch → FleetRuntime::execute()
                  ↓
          Spawn Agent Instances
                  ↓
      ┌──────────┼──────────┐
      ↓          ↓          ↓
  Security   Performance  Quality
  Reviewer    Reviewer    Reviewer
      ↓          ↓          ↓
  Review PR   Review PR   Review PR
      ↓          ↓          ↓
      └──────────┼──────────┘
                 ↓
        Consensus Algorithm
                 ↓
        Aggregate Results
                 ↓
        Return FleetResult
```

**Implementation**: `FleetRuntime::execute_peer_mode()`

```rust
// Execute fleet in peer mode (parallel agents with consensus)
async fn execute_peer_mode(&self, fleet: &AgentFleet, input: serde_json::Value)
    -> Result<FleetResult>
{
    // 1. Spawn all agents in parallel
    let mut tasks = vec![];
    for agent_def in &fleet.spec.agents {
        let agent = self.load_agent(&agent_def).await?;
        let input = input.clone();

        tasks.push(tokio::spawn(async move {
            agent.execute(input).await
        }));
    }

    // 2. Wait for all agents (with timeout)
    let timeout = fleet.spec.coordination.consensus
        .as_ref()
        .and_then(|c| c.timeout_ms)
        .unwrap_or(60000);

    let results = timeout(Duration::from_millis(timeout),
        futures::future::join_all(tasks)
    ).await?;

    // 3. Apply consensus algorithm
    let consensus = self.apply_consensus(
        &fleet.spec.coordination.consensus,
        &results,
        &fleet
    ).await?;

    // 4. Return aggregated result
    Ok(FleetResult {
        consensus_decision: consensus.decision,
        confidence_score: consensus.confidence,
        agent_results: results,
        metadata: consensus.metadata,
    })
}

// Weighted consensus algorithm
async fn apply_weighted_consensus(
    &self,
    results: &[AgentResult],
    fleet: &AgentFleet
) -> ConsensusResult {
    let mut approve_weight = 0.0;
    let mut reject_weight = 0.0;

    for (agent_name, result) in results {
        let weight = fleet.get_agent_weight(agent_name);

        match result.decision {
            Decision::Approve => approve_weight += weight,
            Decision::RequestChanges => reject_weight += weight,
            _ => {}
        }
    }

    let total_weight = approve_weight + reject_weight;
    let confidence = approve_weight / total_weight;

    ConsensusResult {
        decision: if approve_weight > reject_weight {
            Decision::Approve
        } else {
            Decision::RequestChanges
        },
        confidence,
        metadata: serde_json::json!({
            "approve_weight": approve_weight,
            "reject_weight": reject_weight,
            "total_weight": total_weight,
        }),
    }
}
```

### 4. GitHub API Response

```
FleetResult → GitHub API Client
                  ↓
      Format Review Comment
                  ↓
      POST /repos/{owner}/{repo}/pulls/{pr}/reviews
                  ↓
      POST /repos/{owner}/{repo}/issues/{pr}/labels
                  ↓
      POST /repos/{owner}/{repo}/check-runs
```

**Implementation**: `GitHubPlatform` API methods

```rust
// Post PR review
pub async fn post_review(
    &self,
    owner: &str,
    repo: &str,
    pr_number: i64,
    body: &str,
    event: &str  // APPROVE, REQUEST_CHANGES, COMMENT
) -> Result<i64, PlatformError> {
    let url = format!("{}/repos/{}/{}/pulls/{}/reviews",
        self.config.api_url, owner, repo, pr_number);

    let payload = json!({
        "body": body,
        "event": event.to_uppercase()
    });

    let response = self.client
        .post(&url)
        .header("Authorization", format!("Bearer {}", self.config.token))
        .header("Accept", "application/vnd.github+json")
        .json(&payload)
        .send()
        .await?;

    let review: ReviewResponse = response.json().await?;
    Ok(review.id)
}

// Add labels
pub async fn add_labels(
    &self,
    owner: &str,
    repo: &str,
    issue_number: i64,
    labels: &[String]
) -> Result<(), PlatformError> {
    let url = format!("{}/repos/{}/{}/issues/{}/labels",
        self.config.api_url, owner, repo, issue_number);

    self.client
        .post(&url)
        .header("Authorization", format!("Bearer {}", self.config.token))
        .json(&json!({"labels": labels}))
        .send()
        .await?;

    Ok(())
}

// Create check run
pub async fn create_check_run(
    &self,
    owner: &str,
    repo: &str,
    head_sha: &str,
    name: &str,
    status: &str,  // queued, in_progress, completed
    conclusion: Option<&str>,  // success, failure, neutral, etc.
    output: Option<CheckRunOutput>
) -> Result<i64, PlatformError> {
    let url = format!("{}/repos/{}/{}/check-runs",
        self.config.api_url, owner, repo);

    let mut payload = json!({
        "name": name,
        "head_sha": head_sha,
        "status": status
    });

    if let Some(c) = conclusion {
        payload["conclusion"] = json!(c);
    }

    if let Some(o) = output {
        payload["output"] = json!({
            "title": o.title,
            "summary": o.summary,
            "text": o.text
        });
    }

    let response = self.client
        .post(&url)
        .header("Authorization", format!("Bearer {}", self.config.token))
        .json(&payload)
        .send()
        .await?;

    let check_run: CheckRunResponse = response.json().await?;
    Ok(check_run.id)
}
```

---

## Configuration Examples

### Example 1: Simple Agent-Based Review

**Trigger**: `examples/triggers/github-simple-review.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-simple-review

spec:
  type: GitHub
  config:
    token_env: GITHUB_TOKEN
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
    events:
      - pull_request.opened
    repositories:
      - myorg/myrepo

  commands:
    /review:
      agent: code-reviewer
      description: "AI code review"

  default_agent: code-reviewer
```

**Agent**: `examples/agents/code-reviewer.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: code-reviewer

spec:
  model: anthropic:claude-sonnet-4-20250514
  instructions: |
    Review the PR for:
    - Code quality
    - Security issues
    - Performance problems

    Respond with:
    1. Issues found
    2. Recommendations
    3. Decision: APPROVE or REQUEST_CHANGES

  tools:
    - read_file
    - shell

  max_iterations: 5
```

### Example 2: Fleet-Based Multi-Reviewer

**Trigger**: `examples/triggers/github-fleet-review.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-fleet-review

spec:
  type: GitHub
  config:
    token_env: GITHUB_TOKEN
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
    events:
      - pull_request.opened
      - pull_request.synchronize

  default_fleet: pr-review-fleet
```

**Fleet**: See [PR Review Fleet](#2-pr-review-fleet) above.

### Example 3: Workflow-Based Review with Approval

**Trigger**: `examples/triggers/github-workflow-review.yaml`

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: github-workflow-review

spec:
  type: GitHub
  config:
    token_env: GITHUB_TOKEN
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
    events:
      - pull_request.opened
      - pull_request.synchronize

  default_workflow: pr-review-workflow
```

**Workflow**: See [PR Review Workflow](#3-pr-review-workflow) above.

---

## Extension Points

### 1. Custom Review Agents

Users can add custom reviewers to fleets:

```yaml
spec:
  agents:
    # Built-in reviewers
    - name: security-reviewer
      config: ./agents/security-reviewer.yaml

    # Custom reviewer
    - name: domain-reviewer
      spec:
        model: google:gemini-2.5-flash
        instructions: |
          You are a domain expert in fintech regulations.
          Review code changes for PCI-DSS compliance.
```

### 2. Custom Consensus Algorithms

Fleet coordination supports pluggable consensus:

```rust
pub trait ConsensusAlgorithm: Send + Sync {
    async fn aggregate(
        &self,
        results: &[AgentResult],
        config: &ConsensusConfig
    ) -> ConsensusResult;
}

// Register custom algorithm
registry.register_consensus_algorithm(
    "custom_weighted",
    Box::new(CustomWeightedConsensus::new())
);
```

### 3. Custom GitHub Actions

Workflow steps can execute custom GitHub API calls:

```yaml
steps:
  - name: custom-action
    type: agent
    agent: github-ops-agent
    config:
      prompt: |
        Call GitHub API:
        POST /repos/{owner}/{repo}/custom-endpoint
        Body: {custom data}
```

### 4. MCP Server Integration

Reviewers can use MCP servers for specialized tools:

```yaml
spec:
  agents:
    - name: security-reviewer
      spec:
        mcp_servers:
          - name: semgrep
            command: npx
            args: [-y, @modelcontextprotocol/server-semgrep]
            env:
              SEMGREP_API_KEY: ${SEMGREP_API_KEY}

        tools:
          - semgrep.scan_code    # Use Semgrep MCP tool
```

---

## Security Considerations

### 1. Webhook Signature Verification

**Implementation**: HMAC-SHA256 signature verification

```rust
// Verify X-Hub-Signature-256 header
fn verify_github_signature(&self, payload: &[u8], signature: &str) -> bool {
    if !signature.starts_with("sha256=") {
        return false;
    }

    let mut mac = HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes())?;
    mac.update(payload);
    let computed = hex::encode(mac.finalize().into_bytes());

    signature[7..] == computed
}
```

**Configuration**:
```yaml
spec:
  config:
    webhook_secret_env: GITHUB_WEBHOOK_SECRET  # Never hardcode
```

### 2. Token Scoping

**Required GitHub Token Permissions**:

For **Personal Access Tokens (PAT)**:
- `repo` - Full repository access
- `write:discussion` - For PR comments

For **GitHub Apps** (recommended):
- **Repository permissions**:
  - Pull requests: Read & Write
  - Contents: Read
  - Checks: Write (for check runs)
- **Subscribe to events**:
  - Pull request
  - Pull request review
  - Issue comment

**Configuration**:
```bash
export GITHUB_TOKEN="ghp_your_personal_access_token"
# OR for GitHub App:
export GITHUB_TOKEN="ghs_your_github_app_installation_token"
```

### 3. Repository Filtering

Limit triggers to specific repositories:

```yaml
spec:
  config:
    repositories:
      - agenticdevops/aof           # Specific repo
      - agenticdevops/*             # Org wildcard

    # Optional: Exclude repos
    excluded_repos:
      - agenticdevops/private-repo
```

### 4. User Filtering

Ignore bot PRs or restrict to specific users:

```yaml
spec:
  config:
    excluded_users:
      - dependabot[bot]
      - renovate[bot]

    # Or whitelist specific users
    allowed_users:
      - trusted-contributor
      - core-team-member
```

### 5. Rate Limiting

**GitHub API Rate Limits**:
- Authenticated requests: 5,000/hour
- GitHub Apps: 15,000/hour (higher limit)

**Mitigation**:
- Use GitHub Apps over PATs
- Implement exponential backoff in retry logic
- Cache review results in shared memory

```yaml
spec:
  retry:
    max_attempts: 3
    backoff: exponential
    initial_delay: 1s
    max_delay: 30s
```

### 6. Secrets Management

**Best Practices**:
1. Never commit tokens to YAML files
2. Use environment variable references: `${VAR_NAME}`
3. Use secret management tools (Vault, AWS Secrets Manager, etc.)
4. Rotate tokens regularly

```yaml
spec:
  config:
    token_env: GITHUB_TOKEN              # Reference, not value
    webhook_secret_env: GITHUB_WEBHOOK_SECRET
```

---

## Implementation Phases

### Phase 1: Core GitHub Platform (DONE)

**Status**: ✅ Complete

- [x] GitHub webhook parsing (`platforms::github`)
- [x] Signature verification (HMAC-SHA256)
- [x] Event type handling (push, pull_request, issues, etc.)
- [x] API methods: `post_comment`, `post_review`, `create_check_run`, `add_labels`
- [x] TriggerMessage building with PR metadata

**Files**:
- `crates/aof-triggers/src/platforms/github.rs`

### Phase 2: Trigger Integration

**Status**: 🔨 In Progress

**Tasks**:
- [ ] GitHub trigger YAML parsing
- [ ] Trigger router integration
- [ ] Event filtering (events, repos, branches)
- [ ] Command routing to agents/fleets/workflows
- [ ] Default handler for automatic PR events

**Files**:
- `crates/aof-triggers/src/flow.rs` (FlowRouter)
- `examples/triggers/github-pr-review.yaml`

### Phase 3: Fleet Execution

**Status**: 📋 Planned

**Tasks**:
- [ ] Fleet runtime PR review mode
- [ ] Parallel agent execution
- [ ] Weighted consensus algorithm
- [ ] Confidence scoring
- [ ] Result aggregation

**Files**:
- `crates/aof-runtime/src/fleet.rs`
- `examples/fleets/pr-review-fleet.yaml`

### Phase 4: Workflow Execution

**Status**: 📋 Planned

**Tasks**:
- [ ] Workflow PR review mode
- [ ] Conditional routing based on consensus
- [ ] Human-in-the-loop approval gate
- [ ] Label management steps
- [ ] Check run creation steps

**Files**:
- `crates/aof-runtime/src/workflow.rs`
- `examples/workflows/pr-review-workflow.yaml`

### Phase 5: GitHub API Integration

**Status**: 📋 Planned

**Tasks**:
- [ ] Format review comments from agent results
- [ ] Post reviews with APPROVE/REQUEST_CHANGES/COMMENT
- [ ] Add/remove labels based on review outcome
- [ ] Create check runs with detailed output
- [ ] Error handling and retry logic

**Files**:
- `crates/aof-triggers/src/platforms/github.rs` (API methods)

### Phase 6: Documentation & Examples

**Status**: 📋 Planned

**Tasks**:
- [ ] Tutorial: Setting up GitHub PR review
- [ ] Concepts: GitHub integration architecture
- [ ] Reference: GitHub trigger configuration
- [ ] Example: Simple agent-based review
- [ ] Example: Fleet-based multi-reviewer
- [ ] Example: Workflow-based review with approval

**Files**:
- `docs/tutorials/github-pr-review.md`
- `docs/concepts/github-integration.md`
- `docs/reference/github-trigger.md`
- `examples/github/README.md`

---

## Testing Strategy

### 1. Unit Tests

**GitHub Platform Tests** (`platforms/github.rs`):
```rust
#[tokio::test]
async fn test_signature_verification() {
    let config = create_test_config();
    let platform = GitHubPlatform::new(config).unwrap();

    let payload = b"test payload";
    let secret = "test_secret";

    // Generate valid signature
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    assert!(platform.verify_signature(payload, &signature).await);
}

#[tokio::test]
async fn test_parse_pr_webhook() {
    let webhook_json = include_str!("../../../tests/fixtures/github_pr_opened.json");
    let platform = create_test_platform();

    let message = platform.parse_message(
        webhook_json.as_bytes(),
        &test_headers()
    ).await.unwrap();

    assert_eq!(message.metadata.get("pr_number"), Some(&json!(42)));
    assert_eq!(message.metadata.get("event_type"), Some(&json!("pull_request")));
}
```

**Fleet Consensus Tests** (`fleet.rs`):
```rust
#[tokio::test]
async fn test_weighted_consensus() {
    let fleet = load_test_fleet("pr-review-fleet.yaml").await;
    let results = vec![
        ("security-reviewer", Decision::Approve, 2.0),
        ("performance-reviewer", Decision::RequestChanges, 1.0),
        ("quality-reviewer", Decision::Approve, 1.0),
    ];

    let consensus = apply_weighted_consensus(&results, &fleet).await;

    // 2.0 + 1.0 > 1.0 → APPROVE
    assert_eq!(consensus.decision, Decision::Approve);
    assert!(consensus.confidence > 0.6);
}
```

### 2. Integration Tests

**End-to-End PR Review Flow**:
```rust
#[tokio::test]
async fn test_pr_review_e2e() {
    // 1. Setup mock GitHub server
    let mock_server = MockServer::start().await;
    mock_server.register(
        Mock::given(method("POST"))
            .and(path("/repos/test/repo/pulls/42/reviews"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 123,
                "state": "APPROVED"
            })))
    ).await;

    // 2. Send webhook
    let webhook = include_bytes!("../../../tests/fixtures/github_pr_opened.json");
    let response = test_daemon
        .post("/webhook/github")
        .header("X-Hub-Signature-256", test_signature(webhook))
        .body(webhook)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    // 3. Verify review was posted
    let review_posted = mock_server.received_requests()
        .await
        .unwrap()
        .iter()
        .any(|r| r.url.path() == "/repos/test/repo/pulls/42/reviews");

    assert!(review_posted);
}
```

### 3. Manual Testing

**Setup Test Repository**:
```bash
# 1. Create test repo
gh repo create aof-test-pr-review --private

# 2. Configure webhook
gh api repos/agenticdevops/aof-test-pr-review/hooks \
  -f name=web \
  -f active=true \
  -f config[url]=https://your-aof-server.com/webhook/github \
  -f config[content_type]=json \
  -f config[secret]=$GITHUB_WEBHOOK_SECRET \
  -F events[]=pull_request

# 3. Start AOF daemon
export GITHUB_TOKEN="ghp_your_token"
export GITHUB_WEBHOOK_SECRET="your_secret"
aofctl daemon start --config daemon.yaml

# 4. Create test PR
gh pr create --title "Test PR" --body "Test AI review"

# 5. Verify review posted
gh pr view 1 --json reviews
```

---

## Future Enhancements

### 1. PR Comment Commands

Support commands in PR comments to trigger specific actions:

```
/review-security → Run security-only review
/review-performance → Run performance-only review
/review-full → Run complete review workflow
/approve → Force approve (with authorization check)
```

**Implementation**: Parse issue_comment events for `/commands`

### 2. File-Specific Review

Review only changed files matching patterns:

```yaml
spec:
  config:
    file_patterns:
      security-review:
        - "**/*.go"
        - "**/*.rs"
        - "!**/test/**"

      frontend-review:
        - "**/*.tsx"
        - "**/*.jsx"
```

### 3. Incremental Review

Review only new commits since last review:

```yaml
spec:
  config:
    incremental: true
    track_reviewed_commits: true
```

### 4. Review Templates

Pre-configured review templates for common scenarios:

```yaml
spec:
  templates:
    - name: security-audit
      agents: [security-reviewer]
      consensus:
        algorithm: unanimous
        min_confidence: 0.9

    - name: quick-review
      agents: [quality-reviewer]
      consensus:
        algorithm: first_wins
```

### 5. Review Metrics Dashboard

Track review effectiveness:
- Average review time
- Approval rate
- Issues found per review
- False positive rate
- Reviewer accuracy

### 6. Integration with External Tools

- **Semgrep**: Static analysis via MCP server
- **SonarQube**: Code quality metrics
- **Snyk**: Dependency vulnerability scanning
- **CodeClimate**: Maintainability scoring

---

## Appendix

### A. GitHub Webhook Event Schema

**pull_request.opened**:
```json
{
  "action": "opened",
  "number": 42,
  "pull_request": {
    "id": 123,
    "number": 42,
    "title": "Add feature X",
    "body": "Description...",
    "state": "open",
    "draft": false,
    "base": {"ref": "main", "sha": "abc123"},
    "head": {"ref": "feature-x", "sha": "def456"},
    "additions": 250,
    "deletions": 50,
    "changed_files": 8,
    "html_url": "https://github.com/owner/repo/pull/42",
    "user": {"id": 456, "login": "contributor"}
  },
  "repository": {
    "id": 789,
    "name": "repo",
    "full_name": "owner/repo"
  },
  "sender": {"id": 456, "login": "contributor"}
}
```

### B. GitHub API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/repos/{owner}/{repo}/pulls/{pr}/reviews` | POST | Create PR review |
| `/repos/{owner}/{repo}/issues/{issue}/comments` | POST | Post comment |
| `/repos/{owner}/{repo}/issues/{issue}/labels` | POST | Add labels |
| `/repos/{owner}/{repo}/issues/{issue}/labels/{label}` | DELETE | Remove label |
| `/repos/{owner}/{repo}/check-runs` | POST | Create check run |
| `/repos/{owner}/{repo}/pulls/{pr}/files` | GET | Get changed files |

### C. Example Review Comment Format

**Fleet Review Output**:
```markdown
## 🤖 AI Code Review

### Summary
3 agents reviewed this PR with weighted consensus.

**Decision**: ✅ APPROVED
**Confidence**: 85%

---

### 🔒 Security Review (Weight: 2.0)
**Decision**: ✅ APPROVE

No critical security issues found.

**Suggestions**:
- Consider adding input validation for user-supplied data (line 42)
- Use parameterized queries to prevent SQL injection (line 156)

---

### ⚡ Performance Review (Weight: 1.0)
**Decision**: ✅ APPROVE

Code changes look good from a performance perspective.

**Suggestions**:
- Consider caching the result of expensive computation (line 78)

---

### 📋 Quality Review (Weight: 1.0)
**Decision**: ✅ APPROVE

Code follows best practices.

**Suggestions**:
- Add unit tests for new helper functions
- Update documentation for API changes

---

*Generated by AOF v0.1.0 - [Learn More](https://docs.aof.sh)*
```

---

**End of Document**
