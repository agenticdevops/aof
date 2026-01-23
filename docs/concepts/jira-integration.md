---
id: jira-integration
title: Jira Integration
sidebar_label: Jira Integration
description: Deep Jira integration with webhooks and triggers for automated issue triage, sprint planning, and workflow automation
keywords: [jira, integration, webhook, issue, automation, sprint, workflow]
---

# Jira Integration

AOF provides deep Jira integration through webhooks and triggers, enabling AI agents to automate issue triage, sprint planning, standup summaries, and workflow management.

## Overview

Jira Integration enables your AOF agents to:

- **Automatically triage issues** with intelligent labeling and prioritization
- **Assist sprint planning** with story estimation and blocker identification
- **Generate standup summaries** from daily issue activity
- **Monitor SLAs** and alert on approaching deadlines
- **Auto-link related issues** based on content analysis
- **Respond to comments** with contextual suggestions
- **Update issue fields** based on transitions and events

Unlike basic webhook integrations, AOF's Jira support provides full agent orchestration - your agents can read issue details, post comments, update fields, manage labels, transition issues, and analyze project metrics through Jira's official APIs.

## How It Works

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Jira Cloud │      │ AOF Daemon  │      │   Agent     │
│             │      │             │      │             │
│  Webhook    │─────▶│  Trigger    │─────▶│  Executes   │
│  Fired      │      │  Matches    │      │  Task       │
│             │      │             │      │             │
└─────────────┘      └─────────────┘      └─────────────┘
                                                  │
                                                  ▼
                                         ┌─────────────────┐
                                         │ Jira MCP Tools  │
                                         │ - Post comment  │
                                         │ - Update issue  │
                                         │ - Add labels    │
                                         │ - Transition    │
                                         └─────────────────┘
```

**Flow:**
1. Event occurs in Jira (issue created, comment added, sprint started)
2. Jira sends webhook to AOF daemon
3. Daemon routes event to matching triggers
4. Agent executes and uses Jira MCP tools
5. Agent posts comments, updates fields, or transitions issues

## Key Concepts

### 1. Jira Triggers

Jira Triggers are AgentFlow event sources that listen to Jira webhooks and route events to agents.

**How it works:**
1. Configure a webhook in your Jira project/workspace
2. Create a Trigger with `type: Jira`
3. Define which events to listen for (issue created, comment added, sprint started)
4. Route matching events to agents or fleets

```yaml
# flows/jira-triage.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: jira-triage-flow

spec:
  trigger:
    type: Jira
    config:
      webhook_secret: ${JIRA_WEBHOOK_SECRET}

      # Which Jira events to process
      events:
        - jira:issue_created
        - jira:issue_updated

      # Optional: Filter by projects
      allowed_projects:
        - PROJ
        - DEV

      # Optional: Filter by issue types
      issue_types:
        - Bug
        - Story

  agents:
    - name: issue-triager
```

### 2. Jira Event Types

AOF supports all major Jira webhook events:

| Event | Description | Typical Use Cases |
|-------|-------------|-------------------|
| `jira:issue_created` | New issue created | Auto-triage, labeling, assignment |
| `jira:issue_updated` | Issue fields updated | Track priority changes, monitor updates |
| `jira:issue_deleted` | Issue deleted | Archive cleanup, notification |
| `jira:issue_transitioned` | Status changed (To Do → In Progress) | Workflow automation, notifications |
| `comment_created` | New comment added | Interactive bot responses, /commands |
| `comment_updated` | Comment edited | Update tracking |
| `comment_deleted` | Comment removed | Audit logging |
| `sprint_started` | Sprint begins | Planning summary, capacity check |
| `sprint_closed` | Sprint ends | Retrospective data, velocity calculation |
| `board_configuration_changed` | Board settings updated | Configuration sync |
| `worklog_created` | Time logged | Time tracking automation |
| `worklog_updated` | Worklog modified | Timesheet validation |

### 3. Event Filtering

Control when agents execute using event filters:

```yaml
# Example: Only bugs in specific projects
spec:
  trigger:
    type: Jira
    config:
      events:
        - jira:issue_created
      allowed_projects:
        - BACKEND
        - API
      issue_types:
        - Bug
```

```yaml
# Example: High-priority issues only
spec:
  trigger:
    type: Jira
    config:
      events:
        - jira:issue_updated
      filter_expression: "issue.priority == 'Highest' || issue.priority == 'High'"
```

## Common Automation Patterns

### 1. Bug Triage Agent

Automatically label, prioritize, and assign bugs:

```yaml
# flows/bug-triage.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: bug-triage-flow

spec:
  trigger:
    type: Jira
    config:
      webhook_secret: ${JIRA_WEBHOOK_SECRET}
      events:
        - jira:issue_created
      issue_types:
        - Bug

  agents:
    - name: bug-triager
---
# agents/bug-triager.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: bug-triager

spec:
  model: google:gemini-2.5-flash

  tools:
    - type: MCP
      config:
        server: jira-mcp
        command: ["npx", "-y", "@jira/mcp-server"]
        env:
          JIRA_CLOUD_INSTANCE_URL: ${JIRA_CLOUD_INSTANCE_URL}
          JIRA_API_TOKEN: ${JIRA_API_TOKEN}
          JIRA_USER_EMAIL: ${JIRA_USER_EMAIL}

  system_prompt: |
    You are a bug triage specialist. For each new bug:

    1. Analyze description and steps to reproduce
    2. Determine severity:
       - Critical: System down, data loss
       - High: Major feature broken
       - Medium: Feature partially broken
       - Low: Minor issue or cosmetic
    3. Add appropriate labels (backend, frontend, database, api)
    4. Assign to the relevant team based on component
    5. If missing info, comment with questions
    6. Set priority field based on severity assessment
```

**What it does:**
- Reads bug description
- Analyzes severity and impact
- Adds labels (`backend`, `needs-reproduction`, `security`)
- Sets priority (Highest, High, Medium, Low)
- Assigns to appropriate team member
- Requests missing information via comments

### 2. Sprint Planning Assistant

Help estimate stories and identify blockers:

```yaml
# flows/sprint-planning.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: sprint-planning-flow

spec:
  trigger:
    type: Jira
    config:
      events:
        - sprint_started
        - jira:issue_created
      issue_types:
        - Story
        - Task

  agents:
    - name: planning-assistant

  context:
    prompt: |
      You are a sprint planning assistant. For each story:

      1. Estimate story points (1, 2, 3, 5, 8, 13) based on:
         - Complexity of acceptance criteria
         - Number of dependencies
         - Technical unknowns

      2. Flag blockers:
         - Missing requirements
         - Unclear acceptance criteria
         - Dependencies on other teams
         - Technical debt that must be addressed first

      3. Suggest breakdown if > 8 points

      4. Link to related issues

      5. Comment with estimation rationale
```

**What it does:**
- Estimates story points based on complexity
- Flags stories missing acceptance criteria
- Identifies dependencies and blockers
- Suggests breaking down large stories
- Links related issues automatically

### 3. Standup Summary Agent

Generate daily standup summaries:

```yaml
# flows/standup-summary.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: standup-summary-flow

spec:
  trigger:
    type: Schedule
    config:
      cron: "0 9 * * MON-FRI"  # Every weekday at 9 AM

  agents:
    - name: standup-bot

  context:
    prompt: |
      Generate a standup summary for the team:

      1. Query issues updated in last 24 hours
      2. Group by assignee
      3. Summarize:
         - Issues completed (Done)
         - Issues in progress (In Progress)
         - Issues blocked (needs-help label)
      4. Post summary as comment on today's standup ticket
      5. Highlight any blockers or high-priority items needing attention
```

**What it does:**
- Queries yesterday's issue activity
- Groups updates by team member
- Identifies completed work
- Flags blockers
- Posts formatted summary

### 4. SLA Monitor

Alert on approaching SLA deadlines:

```yaml
# flows/sla-monitor.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: sla-monitor-flow

spec:
  trigger:
    type: Schedule
    config:
      cron: "0 */4 * * *"  # Every 4 hours

  agents:
    - name: sla-monitor

  context:
    prompt: |
      Monitor SLA compliance:

      1. Query all open customer issues (label: customer)
      2. Check SLA timers:
         - P0/Critical: 4 hours first response, 24 hours resolution
         - P1/High: 8 hours first response, 48 hours resolution
         - P2/Medium: 24 hours first response, 5 days resolution
      3. For issues approaching deadline:
         - Add "sla-warning" label
         - Comment with time remaining
         - Mention assignee
      4. For overdue issues:
         - Add "sla-breach" label
         - Comment and tag team lead
         - Increase priority if not already Highest
```

**What it does:**
- Monitors SLA timers on customer issues
- Alerts assignees when approaching deadlines
- Escalates overdue issues
- Tracks first response and resolution times

### 5. Issue Linker

Auto-link related issues based on content:

```yaml
# flows/issue-linker.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: issue-linker-flow

spec:
  trigger:
    type: Jira
    config:
      events:
        - jira:issue_created

  agents:
    - name: issue-linker

  context:
    prompt: |
      Find and link related issues:

      1. Extract key terms from issue summary and description
      2. Search for similar issues using JQL:
         - Same component
         - Similar keywords
         - Same epic
      3. For each similar issue found:
         - Determine relationship (duplicates, relates to, blocks)
         - Create issue link
         - Comment explaining the connection
      4. If exact duplicate found:
         - Comment suggesting to close as duplicate
         - Link to original issue
```

**What it does:**
- Analyzes new issue content
- Searches for similar existing issues
- Creates issue links (duplicates, relates, blocks)
- Suggests closing duplicates

## Configuration Example

### Complete DaemonConfig with Jira

```yaml
# daemon.yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-daemon

spec:
  server:
    host: "0.0.0.0"
    port: 3000
    cors: true
    timeout_secs: 60

  platforms:
    jira:
      enabled: true
      # Use base_url for direct Atlassian URL (recommended)
      base_url: https://your-domain.atlassian.net
      # Authentication credentials via environment variables
      user_email_env: JIRA_USER_EMAIL
      api_token_env: JIRA_API_TOKEN
      webhook_secret_env: JIRA_WEBHOOK_SECRET
      bot_name: aofbot  # Optional: name displayed in comments

      # Optional: Restrict to specific projects
      allowed_projects:
        - PROJ
        - DEV

  # Resource directories
  triggers:
    directory: "./triggers"
    watch: true

  agents:
    directory: "./agents"

  flows:
    directory: "./flows"
    enabled: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

**Webhook endpoint**: `https://your-domain.com/webhook/jira`

### Trigger with Interactive Commands

Enable `/analyze` style commands in Jira comments:

```yaml
# flows/jira-commands.yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: jira-commands-flow

spec:
  trigger:
    type: Jira
    config:
      events:
        - comment_created

      # Command-based routing
      commands:
        - command: /analyze
          description: "Analyze issue and suggest next steps"
          agent: analyzer

        - command: /estimate
          description: "Estimate story points"
          agent: estimator

        - command: /blockers
          description: "Identify potential blockers"
          agent: blocker-detector

  agents:
    - name: analyzer
    - name: estimator
    - name: blocker-detector
```

**Usage in Jira:**
```
User comments: /analyze

Agent responds:
Based on the description and acceptance criteria, I recommend:
1. Breaking this into 2 stories (current scope is too large)
2. Clarifying the API contract with the backend team
3. Creating a tech spike for the authentication flow
Estimated effort: 8 story points (5 points if scope reduced)
```

## Authentication Options

### 1. API Token (Jira Cloud - Recommended)

**Setup:**
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Create API token
3. Configure in agent:

```yaml
tools:
  - type: MCP
    config:
      server: jira-mcp
      command: ["npx", "-y", "@jira/mcp-server"]
      env:
        JIRA_CLOUD_INSTANCE_URL: "https://your-domain.atlassian.net"
        JIRA_USER_EMAIL: "bot@yourcompany.com"
        JIRA_API_TOKEN: ${JIRA_API_TOKEN}
```

**Permissions needed:**
- Read issues and projects
- Write issues and comments
- Update issue fields
- Manage sprints (for sprint automation)

### 2. Personal Access Token (Jira Server/Data Center)

For self-hosted Jira:

```yaml
env:
  JIRA_SERVER_URL: "https://jira.yourcompany.com"
  JIRA_PERSONAL_ACCESS_TOKEN: ${JIRA_PAT}
```

### 3. OAuth 2.0 (Jira Apps)

For marketplace apps:

```yaml
env:
  JIRA_OAUTH_CLIENT_ID: ${JIRA_OAUTH_CLIENT_ID}
  JIRA_OAUTH_CLIENT_SECRET: ${JIRA_OAUTH_CLIENT_SECRET}
  JIRA_OAUTH_REDIRECT_URL: "https://your-app.com/oauth/callback"
```

## Jira Webhook Setup

### Step 1: Create Webhook

**Jira Cloud:**
1. Go to Settings → System → WebHooks
2. Create webhook:
   - **Name**: AOF Integration
   - **URL**: `https://your-domain.com/webhook/jira`
   - **Secret**: Generate random string (this is `JIRA_WEBHOOK_SECRET`)
   - **Events**: Select events you need

**Jira Server/Data Center:**
1. Go to Settings → System → Advanced → WebHooks
2. Same configuration as Cloud

### Step 2: Configure Events

Select events based on your use cases:

- ✅ **Issue Created** - For triage, auto-labeling
- ✅ **Issue Updated** - For monitoring changes
- ✅ **Issue Commented** - For interactive commands
- ✅ **Issue Transitioned** - For workflow automation
- ⚠️ **Worklog Created** - Only if tracking time
- ⚠️ **Sprint Started/Closed** - Only for sprint automation

**Tip:** Don't select all events - only what you need to reduce noise.

### Step 3: Test Webhook

```bash
# Start daemon
aofctl serve --config daemon.yaml

# In Jira, create a test issue
# Check daemon logs:
RUST_LOG=debug aofctl serve

# Should see:
# [INFO] Received Jira webhook: issue_created
# [INFO] Routing to flow: jira-triage-flow
# [INFO] Agent executed successfully
```

## Best Practices

### 1. Use `allowed_projects` for Scoping

Don't process events from all projects - scope to relevant ones:

```yaml
# ❌ Too broad - processes all Jira activity
spec:
  trigger:
    type: Jira
    config:
      events:
        - jira:issue_created

# ✅ Scoped - only relevant projects
spec:
  trigger:
    type: Jira
    config:
      events:
        - jira:issue_created
      allowed_projects:
        - BACKEND
        - FRONTEND
```

### 2. Filter by `issue_types` for Targeted Automation

Different workflows for bugs vs. stories:

```yaml
# Bug triage flow
allowed_projects: [BACKEND]
issue_types: [Bug]

# Story planning flow (separate)
allowed_projects: [BACKEND]
issue_types: [Story, Task]
```

### 3. Use Commands for Interactive Workflows

Enable users to trigger analysis on demand:

```yaml
commands:
  - command: /analyze
    agent: analyzer

  - command: /estimate
    agent: estimator
```

Users comment `/analyze` → Agent responds with analysis.

### 4. Rate Limiting

Jira Cloud API limits: 10 requests/second, 100,000/day.

**Tips:**
- Batch operations when possible
- Cache issue data
- Use JQL queries instead of fetching issues one-by-one
- Avoid processing every minor field update

### 5. Use JQL for Bulk Operations

Instead of processing issues one-by-one:

```yaml
system_prompt: |
  Use JQL to find issues efficiently:

  # Find all high-priority bugs
  project = BACKEND AND type = Bug AND priority = High

  # Find overdue issues
  project = BACKEND AND duedate < now()

  # Find unassigned stories in current sprint
  project = BACKEND AND type = Story AND assignee is EMPTY AND sprint in openSprints()
```

### 6. Secure Webhook Endpoints

```yaml
# Always validate webhook signatures
webhook_secret: ${JIRA_WEBHOOK_SECRET}  # Required!

# Use HTTPS in production
# Restrict by IP if possible (Jira Cloud IPs: https://support.atlassian.com/organization-administration/docs/ip-addresses-and-domains-for-atlassian-cloud-products/)
```

## Environment Variables

```bash
# Jira Webhook (for receiving events)
export JIRA_WEBHOOK_SECRET="random-secret-string"

# Jira API (for agent actions)
export JIRA_CLOUD_INSTANCE_URL="https://yourcompany.atlassian.net"
export JIRA_USER_EMAIL="bot@yourcompany.com"
export JIRA_API_TOKEN="your-api-token"

# LLM Provider
export GOOGLE_API_KEY="your-gemini-api-key"
```

## Troubleshooting

### Webhook Not Triggering

1. Check webhook delivery logs in Jira (Settings → System → WebHooks → View details)
2. Verify URL is publicly accessible
3. Check `JIRA_WEBHOOK_SECRET` matches
4. Ensure daemon is running: `aofctl serve --config daemon.yaml`
5. Check logs: `RUST_LOG=debug aofctl serve`

### Agent Not Posting Comments

1. Verify API token permissions (needs write access)
2. Check `JIRA_CLOUD_INSTANCE_URL` is correct (include `https://`)
3. Verify agent has `jira-mcp` tool configured
4. Test token manually:
   ```bash
   curl -u your-email@example.com:$JIRA_API_TOKEN \
     https://yourcompany.atlassian.net/rest/api/3/myself
   ```

### Rate Limiting Issues

```
Error: Rate limit exceeded (429)
```

**Solutions:**
- Use JQL queries instead of individual fetches
- Cache issue data in agent memory
- Reduce event frequency (filter by projects/issue types)
- Implement exponential backoff (AOF handles this automatically)

## Related Documentation

- [GitHub Integration](./github-integration.md) - Similar webhook-based integration
- [AgentFlow Routing Guide](../guides/agentflow-routing.md) - How message routing works
- [Agent Reference](../reference/agent-spec.md) - Complete agent specification
- [MCP Integration](../tools/mcp-integration.md) - Using MCP tools
- [Approval Workflows](../guides/approval-workflow.md) - Human approval gates

## Examples

See the `examples/` directory for complete working examples:

- `examples/flows/jira-bug-triage.yaml` - Automated bug triage
- `examples/flows/jira-sprint-planning.yaml` - Sprint planning assistant
- `examples/agents/issue-triager.yaml` - Issue triage agent
- `examples/agents/standup-bot.yaml` - Daily standup summaries
