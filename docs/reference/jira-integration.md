---
id: jira-integration
title: Jira Integration Reference
sidebar_label: Jira Integration
description: Complete API reference for Jira platform integration with webhooks, events, and actions
keywords: [jira, reference, api, webhook, events, actions, atlassian]
---

# Jira Integration Reference

Complete reference for Jira platform integration with AOF.

## Platform Support Status

| Platform | Status | Notes |
|----------|--------|-------|
| **Jira Cloud** | ✅ Stable | Fully tested with Atlassian Cloud |
| **Jira Server** | 🧪 Experimental | Compatible API, needs validation |
| **Jira Data Center** | 🧪 Experimental | Compatible API, needs validation |

**Important:**
- This documentation focuses on **Jira Cloud**, which is the primary tested platform
- Jira Server and Data Center use compatible REST APIs but may have version-specific differences
- Self-hosted deployments require additional network configuration (webhooks must reach AOF daemon)

## Overview

AOF provides deep Jira integration through webhook-based triggers and the Jira REST API, enabling agents to automate issue triage, sprint management, work logging, and cross-platform workflows with GitHub.

**Key Capabilities:**
- Automatic issue triage and categorization
- AI-powered bug analysis and estimation
- Sprint planning and backlog refinement
- Work logging and time tracking automation
- Multi-project coordination
- GitHub/Jira cross-reference sync
- Custom workflow automation

## Architecture

Jira integration consists of three components:

1. **Jira Trigger** - Receives webhook events from Jira
2. **Jira Platform Adapter** - Parses events and manages API calls
3. **Agents/Flows** - Process events and take actions

```
Jira Webhook → AOF Daemon → Trigger → Agent/Flow → Jira API
```

---

## Configuration Reference

### Daemon Configuration

The DaemonConfig enables Jira as a platform and configures authentication. **Event filtering and command routing are defined in Triggers**, not here.

Configure Jira platform in `daemon.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-daemon

spec:
  server:
    port: 3000
    host: "0.0.0.0"
    cors: true
    timeout_secs: 60

  # Enable platforms
  platforms:
    jira:
      enabled: true
      # Use base_url for direct Atlassian URL (recommended)
      base_url: https://yourcompany.atlassian.net
      # Or use cloud_id_env for Cloud ID based URL construction
      # cloud_id_env: JIRA_CLOUD_ID
      user_email_env: JIRA_USER_EMAIL
      api_token_env: JIRA_API_TOKEN
      webhook_secret_env: JIRA_WEBHOOK_SECRET
      bot_name: aofbot  # Optional: name for comments

      # Optional: Restrict to specific projects
      allowed_projects:
        - PROJ
        - DEV

      # Optional: Filter by event types
      allowed_events:
        - jira:issue_created
        - jira:issue_updated
        - comment_created

  # Resource discovery
  triggers:
    directory: ./triggers/
    watch: true

  agents:
    directory: ./agents/

  fleets:
    directory: ./fleets/

  flows:
    directory: ./flows/
    enabled: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

**Webhook endpoint**: `https://your-domain.com/webhook/jira`

> **Important**: When configuring Jira automation rules, use the full URL with `/webhook/jira` path, not just the base domain.

### Platform Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable Jira webhook endpoint (`/webhook/jira`) |
| `base_url` | string | Yes* | Jira instance URL (e.g., `https://your-domain.atlassian.net`) |
| `cloud_id_env` | string | Yes* | Environment variable for Jira Cloud ID (alternative to base_url) |
| `user_email_env` | string | Yes | Environment variable name for user email |
| `api_token_env` | string | Yes | Environment variable name for API token |
| `webhook_secret_env` | string | Yes | Environment variable name for webhook secret |
| `bot_name` | string | No | Bot name for comments (default: "aofbot") |
| `allowed_projects` | array | No | Project keys allowed to trigger (whitelist) |
| `allowed_events` | array | No | Event types to handle (whitelist) |

*Either `base_url` or `cloud_id_env` must be provided.

### Self-Hosted Jira Configuration

For Jira Server or Data Center deployments, use `base_url` pointing to your internal instance:

```yaml
platforms:
  jira:
    enabled: true
    base_url: https://jira.yourcompany.com  # Self-hosted URL
    user_email_env: JIRA_USER_EMAIL
    api_token_env: JIRA_API_TOKEN  # Use PAT for Server/DC
    webhook_secret_env: JIRA_WEBHOOK_SECRET
```

> **Note**: For Jira Server/Data Center, create a Personal Access Token (PAT) instead of an API token. The configuration is the same - just store the PAT in `JIRA_API_TOKEN`.

> **Note**: Event filtering, project filtering, and command routing can also be configured in **Trigger** files for per-trigger customization.

### Trigger Configuration

Create trigger in `triggers/jira-bug-triage.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-bug-triage

spec:
  type: Jira

  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET

    # Event filters
    events:
      - issue_created
      - issue_updated
      - comment_created

    # Project filters
    filters:
      projects:
        - "PROJ"
        - "TEAM"

      # Issue type filters
      issue_types:
        - "Bug"
        - "Task"
        - "Story"

      # Status filters
      statuses:
        - "To Do"
        - "In Progress"
        - "Blocked"

      # Priority filters (optional)
      priorities:
        - "High"
        - "Critical"

      # Label filters (optional)
      labels:
        - "needs-triage"
        - "security"

    # User filtering
    allowed_users:
      - "alice.developer"
      - "bob.manager"

  # Command routing
  commands:
    /triage:
      agent: bug-triager
      description: "Analyze and categorize this bug"

    /estimate:
      agent: story-estimator
      description: "Estimate complexity and effort"

    /analyze:
      fleet: analysis-fleet
      description: "Deep analysis of issue"

  default_agent: jira-assistant
```

### Trigger Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Must be `Jira` |
| `config.webhook_secret_env` | string | Yes | Environment variable for webhook secret |
| `config.events` | array | Yes | Jira webhook events to listen for |
| `config.filters` | object | No | Filter events by project, type, status, etc. |
| `config.allowed_users` | array | No | Whitelist of Jira usernames |
| `commands` | map | No | Command routing to agents/fleets |
| `default_agent` | string | No | Default agent for unmatched events |

---

## Multi-Project Configuration

AOF uses a **single DaemonConfig + multiple Triggers** architecture for multi-project Jira automation.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Jira Webhook (single URL: /webhook/jira)                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  DaemonConfig (global)                                          │
│  - Enables Jira platform                                        │
│  - Points to triggers/ directory                                │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│ bugs.yaml        │ │ features.yaml    │ │ support.yaml     │
│ projects: PROJ   │ │ projects: FEAT   │ │ projects: SUP    │
│ types: Bug       │ │ types: Story     │ │ types: Support   │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

### Example: Per-Team Triggers

**Directory structure:**
```
config/
  daemon.yaml           # Global config

triggers/
  jira-bugs.yaml        # Bug triage team
  jira-features.yaml    # Feature team
  jira-support.yaml     # Support team
  jira-devops.yaml      # DevOps team
```

**Bug triage trigger** (`triggers/jira-bugs.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: bug-triage-bot
  labels:
    team: qa

spec:
  type: Jira
  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET

    events:
      - issue_created
      - issue_updated
      - comment_created

    filters:
      projects:
        - "PROJ"
        - "API"
      issue_types:
        - "Bug"
      statuses:
        - "Open"
        - "To Do"
        - "Reopened"

  commands:
    /triage:
      agent: bug-triager

    /reproduce:
      agent: bug-reproducer

    /assign:
      agent: bug-assigner

  default_agent: bug-assistant
```

**Feature planning trigger** (`triggers/jira-features.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: feature-planning-bot
  labels:
    team: product

spec:
  type: Jira
  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET

    events:
      - issue_created
      - issue_updated
      - sprint_started
      - sprint_closed

    filters:
      projects:
        - "FEAT"
        - "PROD"
      issue_types:
        - "Story"
        - "Epic"
      statuses:
        - "Backlog"
        - "Ready for Dev"

  commands:
    /estimate:
      agent: story-estimator

    /refine:
      fleet: refinement-fleet

    /acceptance:
      agent: acceptance-criteria-writer

  default_agent: product-assistant
```

**Support team trigger** (`triggers/jira-support.yaml`):
```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: support-bot
  labels:
    team: support

spec:
  type: Jira
  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET

    events:
      - issue_created
      - comment_created

    filters:
      projects:
        - "SUP"
        - "HELP"
      issue_types:
        - "Support"
        - "Incident"
      priorities:
        - "High"
        - "Critical"

  commands:
    /escalate:
      flow: escalation-flow

    /investigate:
      fleet: support-fleet

  default_agent: support-assistant
```

### Webhook Routing

When a webhook arrives, AOF:
1. Parses the project key from the event payload
2. Matches against `filters.projects` in each Trigger
3. Routes to the **first matching Trigger**
4. If no match, event is ignored (logged as unhandled)

---

## Event Reference

### Supported Events

AOF supports all major Jira webhook events:

| Event Type | Action | Description |
|------------|--------|-------------|
| `issue_created` | N/A | New issue created |
| `issue_updated` | N/A | Issue fields updated (status, assignee, etc.) |
| `issue_deleted` | N/A | Issue deleted |
| `comment_created` | N/A | New comment posted |
| `comment_updated` | N/A | Comment edited |
| `comment_deleted` | N/A | Comment deleted |
| `sprint_created` | N/A | New sprint created |
| `sprint_started` | N/A | Sprint started |
| `sprint_closed` | N/A | Sprint closed |
| `sprint_updated` | N/A | Sprint details updated |
| `sprint_deleted` | N/A | Sprint deleted |
| `worklog_created` | N/A | Work logged on issue |
| `worklog_updated` | N/A | Work log updated |
| `worklog_deleted` | N/A | Work log deleted |

### Event Details

#### issue_created

Triggered when a new issue is created.

**When it fires:**
- User creates issue via UI
- Issue created via API
- Issue imported from CSV/external system

**Payload fields:**
- `issue` - Full issue object
- `user` - User who created the issue
- `changelog` - N/A (new issue)

#### issue_updated

Triggered when any issue field changes.

**When it fires:**
- Status transition
- Assignee changed
- Fields updated (priority, labels, custom fields)
- Attachments added/removed

**Payload fields:**
- `issue` - Updated issue object
- `user` - User who made the change
- `changelog` - Array of field changes with old/new values

**Common changelog items:**
| Field | Description |
|-------|-------------|
| `status` | Status transition (e.g., "To Do" → "In Progress") |
| `assignee` | Assignee changed |
| `priority` | Priority changed |
| `labels` | Labels added/removed |
| `sprint` | Sprint added/removed |
| `fixVersions` | Fix versions changed |
| `description` | Description updated |
| `summary` | Title updated |

#### comment_created

Triggered when a comment is posted on an issue.

**When it fires:**
- User posts comment via UI
- Comment added via API
- Bot or integration posts comment

**Payload fields:**
- `issue` - Issue the comment is on
- `comment` - Comment object with body, author, created timestamp
- `user` - User who posted the comment

#### sprint_started

Triggered when a sprint begins.

**When it fires:**
- Sprint manually started by Scrum Master
- Sprint auto-started at scheduled time

**Payload fields:**
- `sprint` - Sprint object (id, name, startDate, endDate, goal)
- `board` - Board the sprint belongs to

#### sprint_closed

Triggered when a sprint ends.

**When it fires:**
- Sprint manually completed
- Sprint auto-completed at end date

**Payload fields:**
- `sprint` - Sprint object
- `board` - Board object
- `issues_completed` - Count of issues completed
- `issues_incomplete` - Count of issues not completed

---

## Webhook Payload Structure

AOF normalizes Jira webhook payloads into a consistent structure.

### Common Fields

All Jira webhooks include these base fields:

```json
{
  "webhookEvent": "jira:issue_created",
  "issue_event_type_name": "issue_created",
  "user": {
    "accountId": "5b10a2844c20165700ede21g",
    "displayName": "Alice Developer",
    "emailAddress": "alice@company.com"
  },
  "timestamp": 1640000000000
}
```

### Issue Events

```json
{
  "webhookEvent": "jira:issue_updated",
  "issue": {
    "id": "10001",
    "key": "PROJ-123",
    "fields": {
      "summary": "Fix login bug",
      "description": "Users cannot log in with SSO",
      "status": {
        "name": "In Progress",
        "statusCategory": {
          "key": "indeterminate"
        }
      },
      "issuetype": {
        "name": "Bug",
        "subtask": false
      },
      "priority": {
        "name": "High"
      },
      "assignee": {
        "accountId": "5b10a2844c20165700ede21g",
        "displayName": "Alice Developer"
      },
      "reporter": {
        "accountId": "5b10a2844c20165700ede22h",
        "displayName": "Bob Manager"
      },
      "labels": ["security", "sso"],
      "created": "2024-01-15T10:30:00.000+0000",
      "updated": "2024-01-15T14:45:00.000+0000"
    }
  },
  "changelog": {
    "items": [
      {
        "field": "status",
        "fieldtype": "jira",
        "from": "10000",
        "fromString": "To Do",
        "to": "10001",
        "toString": "In Progress"
      }
    ]
  }
}
```

### Comment Events

```json
{
  "webhookEvent": "comment_created",
  "comment": {
    "id": "10200",
    "body": "I can reproduce this on staging",
    "author": {
      "accountId": "5b10a2844c20165700ede21g",
      "displayName": "Alice Developer"
    },
    "created": "2024-01-15T15:00:00.000+0000",
    "updated": "2024-01-15T15:00:00.000+0000"
  },
  "issue": {
    "key": "PROJ-123",
    "fields": { ... }
  }
}
```

### Sprint Events

```json
{
  "webhookEvent": "sprint_started",
  "sprint": {
    "id": 42,
    "name": "Sprint 10",
    "state": "active",
    "startDate": "2024-01-15T09:00:00.000Z",
    "endDate": "2024-01-29T17:00:00.000Z",
    "goal": "Complete authentication refactor"
  },
  "board": {
    "id": 1,
    "name": "Engineering Board",
    "type": "scrum"
  }
}
```

---

## Context Variables

When an agent is triggered by a Jira event, these variables are available in `metadata`:

### Common Variables

| Variable | Type | Description |
|----------|------|-------------|
| `event_type` | string | Event type (issue_created, etc.) |
| `timestamp` | integer | Event timestamp (milliseconds) |
| `user_account_id` | string | User account ID |
| `user_display_name` | string | User display name |
| `user_email` | string | User email address |

### Issue Variables

Available for all issue events:

| Variable | Type | Description |
|----------|------|-------------|
| `issue_id` | string | Issue ID |
| `issue_key` | string | Issue key (PROJ-123) |
| `issue_summary` | string | Issue title |
| `issue_description` | string | Issue description |
| `issue_type` | string | Issue type (Bug, Story, etc.) |
| `issue_status` | string | Current status |
| `issue_priority` | string | Priority (High, Medium, Low) |
| `issue_assignee_id` | string | Assignee account ID |
| `issue_assignee_name` | string | Assignee display name |
| `issue_reporter_id` | string | Reporter account ID |
| `issue_reporter_name` | string | Reporter display name |
| `issue_labels` | array | Labels on issue |
| `issue_created` | string | Creation timestamp |
| `issue_updated` | string | Last update timestamp |
| `project_key` | string | Project key |
| `project_name` | string | Project name |

### Comment Variables

Available for comment events:

| Variable | Type | Description |
|----------|------|-------------|
| `comment_id` | string | Comment ID |
| `comment_body` | string | Comment text |
| `comment_author_id` | string | Author account ID |
| `comment_author_name` | string | Author display name |
| `comment_created` | string | Comment creation timestamp |

### Sprint Variables

Available for sprint events:

| Variable | Type | Description |
|----------|------|-------------|
| `sprint_id` | integer | Sprint ID |
| `sprint_name` | string | Sprint name |
| `sprint_state` | string | Sprint state (future, active, closed) |
| `sprint_start_date` | string | Sprint start date |
| `sprint_end_date` | string | Sprint end date |
| `sprint_goal` | string | Sprint goal |
| `board_id` | integer | Board ID |
| `board_name` | string | Board name |

### Changelog Variables

Available for `issue_updated` events:

| Variable | Type | Description |
|----------|------|-------------|
| `changelog_items` | array | Array of changed fields |

Each changelog item has:
- `field` - Field name (status, assignee, etc.)
- `from_string` - Old value (human-readable)
- `to_string` - New value (human-readable)

### Usage in Agent Instructions

```yaml
spec:
  instructions: |
    You are analyzing issue {{ metadata.issue_key }}.

    Issue Details:
    - Title: {{ metadata.issue_summary }}
    - Type: {{ metadata.issue_type }}
    - Status: {{ metadata.issue_status }}
    - Priority: {{ metadata.issue_priority }}
    - Assignee: {{ metadata.issue_assignee_name }}
    - Reporter: {{ metadata.issue_reporter_name }}

    Description:
    {{ metadata.issue_description }}

    {% if metadata.comment_body %}
    Latest Comment by {{ metadata.comment_author_name }}:
    {{ metadata.comment_body }}
    {% endif %}

    {% if metadata.changelog_items %}
    Recent Changes:
    {% for item in metadata.changelog_items %}
    - {{ item.field }}: {{ item.from_string }} → {{ item.to_string }}
    {% endfor %}
    {% endif %}
```

---

## API Actions

Actions agents can perform on Jira via built-in tools.

### Post Comment

Post a comment on an issue.

```yaml
# In agent instructions
tools:
  - jira_post_comment

# Usage
Post a comment to {{ metadata.issue_key }}:
"This issue has been triaged and assigned to the backend team."
```

**Rust API:**
```rust
platform.post_comment("PROJ-123", "Comment text").await?;
```

**Parameters:**
- `issue_key` (string) - Issue key (PROJ-123)
- `body` (string) - Comment text (supports Jira markup)

**Returns:** Comment ID

### Update Issue

Update issue fields.

```yaml
# Usage
Update {{ metadata.issue_key }}:
- Set status to "In Progress"
- Set assignee to "alice.developer"
- Add labels ["backend", "urgent"]
```

**Rust API:**
```rust
use serde_json::json;

platform.update_issue("PROJ-123", json!({
    "fields": {
        "assignee": {"accountId": "5b10a2844c20165700ede21g"},
        "labels": ["backend", "urgent"],
        "priority": {"name": "High"}
    }
})).await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `fields` (object) - Fields to update

**Common fields:**
| Field | Value Format | Example |
|-------|--------------|---------|
| `assignee` | `{"accountId": "..."}` | Assign to user |
| `priority` | `{"name": "High"}` | Set priority |
| `labels` | `["label1", "label2"]` | Set labels |
| `description` | String | Update description |
| `summary` | String | Update title |
| Custom fields | Varies by field type | See Jira API docs |

### Transition Issue

Change issue status (transition workflow).

```yaml
# Usage
Transition {{ metadata.issue_key }} to "In Progress"
```

**Rust API:**
```rust
platform.transition_issue("PROJ-123", "31").await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `transition_id` (string) - Transition ID (get from workflow)

**Finding transition IDs:**
```bash
curl -u email@example.com:api_token \
  https://yourcompany.atlassian.net/rest/api/3/issue/PROJ-123/transitions
```

### Add Label

Add labels to an issue.

```yaml
# Usage
Add labels ["needs-review", "security"] to {{ metadata.issue_key }}
```

**Rust API:**
```rust
platform.add_labels("PROJ-123", &["needs-review", "security"]).await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `labels` (array) - Labels to add

### Remove Label

Remove a label from an issue.

```yaml
# Usage
Remove label "needs-triage" from {{ metadata.issue_key }}
```

**Rust API:**
```rust
platform.remove_label("PROJ-123", "needs-triage").await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `label` (string) - Label to remove

### Assign User

Assign an issue to a user.

```yaml
# Usage
Assign {{ metadata.issue_key }} to "alice.developer"
```

**Rust API:**
```rust
platform.assign_issue("PROJ-123", "5b10a2844c20165700ede21g").await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `account_id` (string) - User account ID

**Finding account IDs:**
```bash
curl -u email@example.com:api_token \
  https://yourcompany.atlassian.net/rest/api/3/user/search?query=alice
```

### Log Work

Log time spent on an issue.

```yaml
# Usage
Log 2 hours of work on {{ metadata.issue_key }}
Comment: "Fixed authentication bug"
```

**Rust API:**
```rust
use serde_json::json;

platform.log_work("PROJ-123", json!({
    "timeSpent": "2h",
    "comment": "Fixed authentication bug",
    "started": "2024-01-15T09:00:00.000+0000"
})).await?;
```

**Parameters:**
- `issue_key` (string) - Issue key
- `time_spent` (string) - Time format (e.g., "2h 30m", "1d")
- `comment` (string, optional) - Work log comment
- `started` (string, optional) - Start timestamp

**Time formats:**
- `1w` - 1 week
- `2d` - 2 days
- `4h` - 4 hours
- `30m` - 30 minutes
- `1h 30m` - Combined

### Link Issues

Create a link between two issues.

```yaml
# Usage
Link {{ metadata.issue_key }} to PROJ-456 as "relates to"
```

**Rust API:**
```rust
use serde_json::json;

platform.link_issues(json!({
    "type": {"name": "Relates"},
    "inwardIssue": {"key": "PROJ-123"},
    "outwardIssue": {"key": "PROJ-456"}
})).await?;
```

**Parameters:**
- `type` (string) - Link type name
- `inward_issue` (string) - Source issue key
- `outward_issue` (string) - Target issue key

**Common link types:**
| Type | Description |
|------|-------------|
| `Blocks` | This issue blocks another |
| `Relates` | General relationship |
| `Duplicates` | Duplicate issue |
| `Causes` | This issue causes another |
| `Clones` | Cloned from another issue |

---

## Security

### Webhook Verification

AOF validates webhook requests using a shared secret.

**How it works:**
1. Configure webhook secret in Jira webhook settings
2. AOF validates incoming requests against configured secret
3. Requests without valid signature are rejected

**Configuration:**
```yaml
spec:
  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET  # Required
```

**Generate secure secret:**
```bash
openssl rand -hex 32
```

### Authentication Methods

#### API Token (Jira Cloud - Recommended)

**Create API token:**
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click "Create API token"
3. Copy token and store securely

**Configure:**
```bash
export JIRA_EMAIL="your-email@company.com"
export JIRA_API_TOKEN="your-api-token"
```

**Permissions:** Inherits permissions from your Jira Cloud account.

#### Personal Access Token (Self-Hosted)

**For Jira Server/Data Center:**
1. Go to Profile → Personal Access Tokens
2. Create new token with required permissions
3. Copy token immediately (shown only once)

**Configure:**
```bash
export JIRA_PAT="your-personal-access-token"
```

#### OAuth 2.0 (Advanced)

For programmatic access without user credentials:

1. Register OAuth 2.0 app in Jira
2. Obtain client ID and secret
3. Implement OAuth flow
4. Store refresh token

**Configuration:**
```yaml
platforms:
  jira:
    auth:
      type: oauth2
      client_id_env: JIRA_OAUTH_CLIENT_ID
      client_secret_env: JIRA_OAUTH_SECRET
      token_env: JIRA_OAUTH_TOKEN
```

### Project Filtering

Whitelist projects for security:

```yaml
filters:
  projects:
    - "PROJ"           # Specific project
    - "TEAM"           # Another project
    - "PROD-*"         # Pattern matching (if supported)
```

### User Filtering

Restrict who can trigger automation:

```yaml
config:
  allowed_users:
    - "alice.developer"
    - "bob.manager"
    - "qa-bot"
```

Events from users not in this list will be ignored.

> **Note**: User filtering is based on Jira usernames/account IDs. Group-based authorization is planned for a future release.

---

## Rate Limiting

### Jira API Limits

| Type | Limit | Notes |
|------|-------|-------|
| **Jira Cloud** | Varies by plan | ~100-300 req/sec depending on subscription |
| **Jira Server** | Configurable | Set by admin in Rate Limiting settings |
| **Search API** | 20 req/sec | Lower limit for JQL searches |

**Jira Cloud rate limits:**
- Free tier: ~100 requests/second
- Standard/Premium: ~200-300 requests/second
- Enterprise: Custom limits

### Mitigation Strategies

1. **Batch operations** - Update multiple fields in single API call
2. **Use webhooks** - React to events instead of polling
3. **Cache data** - Store frequently accessed data locally
4. **Implement backoff** - Retry with exponential backoff
5. **Use bulk APIs** - Bulk update, bulk delete operations

**Check rate limit:**
```bash
# Jira Cloud
curl -u email@example.com:api_token \
  -X GET "https://yourcompany.atlassian.net/rest/api/3/myself" \
  -H "Accept: application/json"
```

Check response headers:
- `X-RateLimit-Limit` - Rate limit ceiling
- `X-RateLimit-Remaining` - Requests left in window
- `Retry-After` - Seconds until retry (if rate limited)

**AOF handles retries automatically** with exponential backoff when rate limit is hit.

---

## Environment Variables

```bash
# Jira Cloud (API Token)
export JIRA_EMAIL="your-email@company.com"
export JIRA_API_TOKEN="your-api-token"
export JIRA_WEBHOOK_SECRET="random-secret-key"

# Jira Server/Data Center (PAT)
export JIRA_PAT="your-personal-access-token"
export JIRA_WEBHOOK_SECRET="random-secret-key"

# LLM provider
export GOOGLE_API_KEY="xxxxx"
# OR
export ANTHROPIC_API_KEY="xxxxx"
```

---

## Examples

### Auto-Triage Bugs

```yaml
# triggers/jira-bug-triage.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: bug-triage

spec:
  type: Jira
  config:
    webhook_secret_env: JIRA_WEBHOOK_SECRET
    events:
      - issue_created
    filters:
      projects: ["PROJ"]
      issue_types: ["Bug"]

  default_agent: bug-triager
```

```yaml
# agents/bug-triager.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: bug-triager

spec:
  model: google:gemini-2.5-flash

  instructions: |
    Analyze bug {{ metadata.issue_key }}.

    Issue: {{ metadata.issue_summary }}
    Description: {{ metadata.issue_description }}

    1. Determine severity based on description
    2. Set priority (Critical/High/Medium/Low)
    3. Add relevant labels (backend, frontend, database, etc.)
    4. Suggest assignee based on component
    5. Post analysis comment

  tools:
    - jira_update_issue
    - jira_add_labels
    - jira_post_comment
```

### Story Point Estimation

```yaml
# triggers/jira-estimation.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: story-estimation

spec:
  type: Jira
  config:
    events:
      - issue_created
    filters:
      projects: ["FEAT"]
      issue_types: ["Story", "Task"]

  commands:
    /estimate:
      agent: story-estimator
```

```yaml
# agents/story-estimator.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: story-estimator

spec:
  model: google:gemini-2.5-flash

  instructions: |
    Estimate story points for {{ metadata.issue_key }}.

    Story: {{ metadata.issue_summary }}
    Description: {{ metadata.issue_description }}

    Analyze complexity based on:
    1. Technical complexity
    2. Unknown factors
    3. Dependencies
    4. Testing requirements

    Recommend story points (1, 2, 3, 5, 8, 13) and explain reasoning.
    Post comment with analysis.

  tools:
    - jira_post_comment
```

### Sprint Planning Automation

```yaml
# triggers/jira-sprint.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: sprint-planning

spec:
  type: Jira
  config:
    events:
      - sprint_started
      - sprint_closed

  default_fleet: sprint-fleet
```

```yaml
# fleets/sprint-fleet.yaml
apiVersion: aof.dev/v1
kind: Fleet
metadata:
  name: sprint-fleet

spec:
  agents:
    - name: sprint-analyzer
      role: coordinator
    - name: velocity-tracker
      role: specialist
    - name: report-generator
      role: specialist

  workflow: |
    On sprint start:
    1. sprint-analyzer: Analyze sprint capacity and commitments
    2. velocity-tracker: Calculate team velocity from previous sprints
    3. report-generator: Generate sprint kickoff report

    On sprint close:
    1. sprint-analyzer: Review completed vs planned work
    2. velocity-tracker: Update velocity metrics
    3. report-generator: Generate retrospective report
```

### GitHub/Jira Cross-Reference

```yaml
# agents/github-jira-sync.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: github-jira-sync

spec:
  model: google:gemini-2.5-flash

  instructions: |
    You sync GitHub PRs with Jira issues.

    When PR is opened:
    1. Extract Jira issue key from PR title (PROJ-123)
    2. Post comment on Jira issue with PR link
    3. Add "in-review" label to Jira issue

    When PR is merged:
    1. Transition Jira issue to "Done"
    2. Post merge confirmation comment
    3. Log work time based on PR size

  tools:
    - jira_post_comment
    - jira_transition_issue
    - jira_add_labels
    - jira_log_work
```

---

## Webhook Setup

There are two ways to configure Jira webhooks:

### Option A: Jira Automation Rules (Project-Level)

Use this method if you don't have Jira admin access or want per-project control.

#### 1. Create Automation Rule

1. Go to your Jira project
2. Navigate to **Project Settings** → **Automation**
3. Click **Create rule**
4. Choose a trigger (e.g., **When: Issue created**)
5. Add action → **Send web request**

#### 2. Configure Web Request

**URL**: `https://your-domain.com/webhook/jira`

**HTTP method**: `POST`

**Headers**:
| Key | Value |
|-----|-------|
| `Content-Type` | `application/json` |
| `X-Hub-Signature` | `<your JIRA_WEBHOOK_SECRET value>` |

**Web request body**: Select **Custom data** and use a payload template.

#### 3. Payload Templates

**Issue Created/Updated:**
```json
{
  "webhookEvent": "jira:issue_created",
  "timestamp": {{now.asLong}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "description": "{{issue.description}}",
      "issuetype": { "name": "{{issue.issueType.name}}" },
      "status": { "name": "{{issue.status.name}}" },
      "priority": { "name": "{{issue.priority.name}}" },
      "project": {
        "key": "{{issue.project.key}}",
        "name": "{{issue.project.name}}"
      },
      "assignee": {
        "displayName": "{{issue.assignee.displayName}}",
        "accountId": "{{issue.assignee.accountId}}"
      },
      "reporter": {
        "displayName": "{{issue.reporter.displayName}}",
        "accountId": "{{issue.reporter.accountId}}"
      },
      "labels": {{issue.labels.asJsonArray}}
    }
  },
  "user": {
    "accountId": "{{initiator.accountId}}",
    "displayName": "{{initiator.displayName}}"
  }
}
```

**Comment Created:**
```json
{
  "webhookEvent": "comment_created",
  "timestamp": {{now.asLong}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "project": {
        "key": "{{issue.project.key}}",
        "name": "{{issue.project.name}}"
      }
    }
  },
  "comment": {
    "id": "{{comment.id}}",
    "body": "{{comment.body}}",
    "author": {
      "accountId": "{{comment.author.accountId}}",
      "displayName": "{{comment.author.displayName}}"
    }
  },
  "user": {
    "accountId": "{{initiator.accountId}}",
    "displayName": "{{initiator.displayName}}"
  }
}
```

**Work Logged:**
```json
{
  "webhookEvent": "worklog_created",
  "timestamp": {{now.asLong}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "issuetype": { "name": "{{issue.issueType.name}}" },
      "status": { "name": "{{issue.status.name}}" },
      "priority": { "name": "{{issue.priority.name}}" },
      "project": {
        "key": "{{issue.project.key}}",
        "name": "{{issue.project.name}}"
      }
    }
  },
  "user": {
    "accountId": "{{initiator.accountId}}",
    "displayName": "{{initiator.displayName}}"
  }
}
```

> **Important**: The `{{...}}` placeholders are Jira smart values. They get replaced with actual data when the webhook fires.

#### 4. Signature Verification

Jira Automation sends the `X-Hub-Signature` header value as a **static secret** (not computed HMAC). Your `JIRA_WEBHOOK_SECRET` environment variable must **exactly match** the value you configure in the header.

---

### Option B: System Webhooks (Admin Only)

Use this method if you have Jira admin access. System webhooks automatically include complete payloads.

#### Jira Cloud

1. Go to **Settings** → **System** → **WebHooks**
2. Click **Create a WebHook**
3. Configure:
   - **Name**: AOF Automation
   - **Status**: Enabled
   - **URL**: `https://your-domain.com/webhook/jira`
   - **Secret**: Your `JIRA_WEBHOOK_SECRET` value (enables HMAC verification)
   - **Events**: Select desired events
   - **Exclude body**: Uncheck (AOF needs full payload)

#### Jira Server/Data Center

1. Go to **Settings** → **System** → **Advanced** → **WebHooks**
2. Create webhook with same configuration as Cloud
3. Ensure firewall allows webhook traffic to AOF daemon

---

### Expose Endpoint

**For production:**
```bash
# HTTPS required
https://your-domain.com/webhook/jira
```

**For local testing:**
Use a tunnel service:

```bash
# Option 1: Cloudflared (no signup)
brew install cloudflared
cloudflared tunnel --url http://localhost:3000

# Option 2: ngrok (free account)
ngrok http 3000
```

Use tunnel URL as webhook URL in Jira.

### Verify Webhook

1. Test webhook using Jira's "Validate" button (Automation) or delivery logs (System webhooks)
2. Check AOF daemon logs for received events

```bash
# Check logs
RUST_LOG=debug aofctl serve --config daemon.yaml

# Look for:
# INFO  Received webhook for platform: jira
# DEBUG Jira signature verified via direct secret match
# INFO  Processing event: jira:issue_created
```

---

## Troubleshooting

### Webhook Not Triggering

**Symptoms:** Jira webhook shows success but agent doesn't run

**Solutions:**
1. Check webhook delivery logs in Jira admin panel
2. Verify webhook secret matches `JIRA_WEBHOOK_SECRET`
3. Check firewall allows Jira Cloud IPs (if using Cloud)
4. Verify daemon is running: `aofctl serve --config daemon.yaml`
5. Check logs: `RUST_LOG=debug aofctl serve`

### Authentication Failed

**Symptoms:** `401 Unauthorized` in logs

**Solutions:**
1. Verify API token/PAT is valid
2. For API token: Confirm `JIRA_EMAIL` matches token owner
3. Test credentials manually:
   ```bash
   curl -u email@example.com:api_token \
     https://yourcompany.atlassian.net/rest/api/3/myself
   ```
4. Check token hasn't expired (PATs expire in Server/DC)

### Cannot Post Comments

**Symptoms:** Agent runs but comments don't appear

**Solutions:**
1. Verify token has correct permissions
2. Check user has access to project
3. Verify issue exists and is not deleted
4. Test comment API manually:
   ```bash
   curl -u email@example.com:api_token \
     -X POST "https://yourcompany.atlassian.net/rest/api/3/issue/PROJ-123/comment" \
     -H "Content-Type: application/json" \
     -d '{"body": "Test comment"}'
   ```

### Base URL Configuration

**Symptoms:** `404 Not Found` or connection errors

**Solutions:**
1. Verify `base_url` format:
   - Jira Cloud: `https://yourcompany.atlassian.net`
   - Self-hosted: `https://jira.yourcompany.com`
2. No trailing slash in URL
3. For self-hosted: Verify DNS resolution and network access

### Rate Limit Issues

**Symptoms:** `429 Too Many Requests` errors

**Solutions:**
1. Check rate limit headers in API responses
2. Reduce webhook event frequency (filter by project/type)
3. Implement caching for frequently accessed data
4. Contact Atlassian support to increase limits (Cloud)

---

## Best Practices

### 1. Filter Events Precisely

**❌ Too broad:**
```yaml
events:
  - issue_created
  - issue_updated
  - comment_created
```

**✅ Specific:**
```yaml
events:
  - issue_created
  - comment_created
filters:
  projects: ["PROJ"]
  issue_types: ["Bug"]
  statuses: ["Open"]
```

### 2. Use Project Filters

**For production:**
```yaml
filters:
  projects:
    - "PROD"
    - "API"
  issue_types:
    - "Bug"
    - "Incident"
```

Prevents accidental automation on test projects.

### 3. Implement Approval Gates

For destructive actions:
```yaml
spec:
  instructions: |
    Before transitioning to Done:

    1. Verify all subtasks are complete
    2. Check PR is merged (if linked)
    3. Ensure QA has signed off
    4. Confirm no blocking issues
    5. Request human approval if uncertain
```

### 4. Cache Jira Data

Reduce API calls:
```yaml
spec:
  memory: "File:./jira-cache.json:1000"

  instructions: |
    Before making API calls, check memory cache for:
    - User account IDs
    - Transition IDs
    - Custom field IDs
```

### 5. Use Structured Comments

Post comments with clear structure:
```yaml
spec:
  instructions: |
    Post comment with this format:

    h3. AI Analysis
    [Analysis summary]

    h4. Recommended Actions
    * Action 1
    * Action 2

    h4. Severity Assessment
    Priority: [High/Medium/Low]
    Labels: [suggested-labels]
```

### 6. Log Work Automatically

Track time spent on automated tasks:
```yaml
spec:
  instructions: |
    After completing analysis:
    1. Log work time based on complexity
    2. Add comment explaining what was done
    3. Update issue status if appropriate
```

---

## Enterprise Considerations

### Self-Hosted Deployment

**Network requirements:**
- Jira must be able to reach AOF daemon webhook endpoint
- Consider reverse proxy for SSL termination
- Firewall rules to allow Jira → AOF traffic

**Configuration:**
```yaml
platforms:
  jira:
    base_url: https://jira.company.internal
    proxy:
      http_proxy: http://proxy.company.com:8080
      https_proxy: https://proxy.company.com:8080
    tls:
      verify: true
      ca_cert_path: /etc/ssl/certs/company-ca.pem
```

### High Availability

Run multiple AOF daemon replicas:

```yaml
# Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-daemon
spec:
  replicas: 3
  template:
    spec:
      containers:
        - name: aof
          image: ghcr.io/agenticdevops/aof:latest
```

### Scaling Recommendations

| Scale | Projects | Issues/day | Setup |
|-------|----------|------------|-------|
| Small | 1-10 | Up to 1000 | Single daemon |
| Medium | 10-50 | 1000-5000 | 2-3 replicas |
| Large | 50-200 | 5000-20000 | Sharded by project |
| Enterprise | 200+ | 20000+ | Message queue (planned) |

---

## See Also

- [Trigger Specification](./trigger-spec.md) - Complete trigger reference
- [Agent Specification](./agent-spec.md) - Agent configuration
- [Daemon Configuration](./daemon-config.md) - Daemon setup
- [GitHub Integration](./github-integration.md) - GitHub automation
- [Platform Policies](./platform-policies.md) - Cross-platform policies
