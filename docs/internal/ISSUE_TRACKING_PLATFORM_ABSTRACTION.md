# Issue Tracking Platform Abstraction Design

## Overview

AOF's Issue Tracking Platform abstraction provides a unified interface for integrating with multiple issue tracking systems (Jira, Linear, Asana, Monday.com, etc.). This abstraction enables:

- **Reusable agent workflows** - Write once, run on any issue tracking platform
- **Consistent event handling** - Normalized webhook events across platforms
- **Unified API operations** - Platform-agnostic actions (create issue, add comment, transition status)
- **Easy platform addition** - Clear adapter interface for new platforms

### Why Abstraction Matters

Without abstraction, every agent would need platform-specific logic:
```yaml
# ❌ Bad: Platform-specific agents
- name: jira-issue-triage
  triggers:
    - type: Jira  # Only works with Jira

- name: linear-issue-triage  # Duplicate logic for Linear
  triggers:
    - type: Linear
```

With abstraction:
```yaml
# ✅ Good: Platform-agnostic agents
- name: issue-triage
  triggers:
    - type: IssuePlatform  # Works with Jira, Linear, Asana, Monday
      config:
        platform: Jira | Linear | Asana | Monday
        events: [issue.created, issue.updated]
```

### Key Use Cases

1. **Automated Triage** - Classify and route issues based on content
2. **SLA Monitoring** - Track response times and escalate overdue issues
3. **Sprint Automation** - Auto-assign issues to sprints based on priority
4. **Status Synchronization** - Keep issues in sync across platforms
5. **Incident Management** - Create and track incidents with automated workflows
6. **Knowledge Base Integration** - Link similar issues and suggest solutions
7. **Approval Workflows** - Route issues through approval chains

## Unified Event Model

### Core Event Types

All platforms map to these normalized event types:

| Unified Event | Jira | Linear | Asana | Monday.com |
|---------------|------|--------|-------|------------|
| `issue` | Issue | Issue | Task | Item |
| `comment` | Comment | Comment | Story/Comment | Update |
| `sprint` | Sprint | Cycle | N/A (Projects) | N/A (Boards) |
| `status_change` | Transition | State Change | Section Move | Status Change |
| `assignment` | Assignee | Assignee | Task Assignment | Person Column |
| `priority_change` | Priority | Priority | N/A (Custom) | Priority Column |

### Event Actions

Each event type has standardized actions:

#### `issue` Actions
- `created` - New issue/task created
- `updated` - Issue fields modified
- `deleted` - Issue removed (soft delete)
- `moved` - Issue moved to different project/board
- `linked` - Issue linked to another issue
- `cloned` - Issue duplicated
- `converted` - Issue type changed

#### `comment` Actions
- `added` - New comment posted
- `updated` - Comment edited
- `deleted` - Comment removed

#### `sprint` Actions (Jira/Linear specific)
- `started` - Sprint/cycle begins
- `completed` - Sprint/cycle ends
- `updated` - Sprint dates/scope changed
- `issue_added` - Issue added to sprint
- `issue_removed` - Issue removed from sprint

#### `status_change` Actions
- `transitioned` - Issue moved to new status
- `reopened` - Closed issue reopened
- `resolved` - Issue marked as resolved
- `closed` - Issue closed

#### `assignment` Actions
- `assigned` - User assigned to issue
- `unassigned` - User removed from issue
- `reassigned` - Issue assigned to different user

### Event Data Normalization

All platforms provide this normalized context to agents:

```rust
pub struct IssuePlatformEvent {
    pub event_type: IssueEventType,
    pub action: IssueEventAction,
    pub workspace: Workspace,
    pub sender: User,
    pub issue: Option<Issue>,
    pub comment: Option<Comment>,
    pub sprint: Option<Sprint>,
    pub status_change: Option<StatusChange>,
}

pub enum IssueEventType {
    Issue,
    Comment,
    Sprint,
    StatusChange,
    Assignment,
    PriorityChange,
}

pub enum IssueEventAction {
    Created,
    Updated,
    Deleted,
    Moved,
    Transitioned,
    Assigned,
    Unassigned,
    // ... more actions
}

pub struct Workspace {
    pub id: String,
    pub name: String,
    pub url: String,
    pub project_id: String,      // Project/Team/Board ID
    pub project_name: String,
}

pub struct Issue {
    pub id: String,              // Platform-specific ID
    pub key: String,             // Human-readable key (PROJ-123, TEAM-456)
    pub title: String,
    pub description: String,
    pub issue_type: IssueType,   // bug, feature, task, story, epic
    pub status: Status,
    pub priority: Priority,
    pub assignee: Option<User>,
    pub reporter: User,
    pub labels: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub estimate: Option<f64>,   // Story points or hours
    pub sprint: Option<SprintRef>,
    pub parent: Option<IssueRef>, // Epic/Parent task
    pub subtasks: Vec<IssueRef>,
    pub links: Vec<IssueLink>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub url: String,
}

pub struct IssueType {
    pub id: String,
    pub name: String,            // bug, feature, task, story, epic
    pub icon_url: Option<String>,
}

pub struct Status {
    pub id: String,
    pub name: String,            // To Do, In Progress, Done
    pub category: StatusCategory, // todo, in_progress, done
}

pub enum StatusCategory {
    ToDo,
    InProgress,
    Done,
    Blocked,
    Backlog,
}

pub struct Priority {
    pub id: String,
    pub name: String,            // Critical, High, Medium, Low
    pub level: u8,               // 1-5 (1=highest)
    pub icon_url: Option<String>,
}

pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
}

pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub author: User,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub visibility: CommentVisibility,
}

pub enum CommentVisibility {
    Public,
    Internal,       // Visible to team only
    Private,        // Visible to specific roles
}

pub struct Sprint {
    pub id: String,
    pub name: String,
    pub state: SprintState,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub goal: Option<String>,
}

pub enum SprintState {
    Future,
    Active,
    Closed,
}

pub struct SprintRef {
    pub id: String,
    pub name: String,
}

pub struct IssueRef {
    pub id: String,
    pub key: String,
}

pub struct IssueLink {
    pub link_type: String,       // blocks, duplicates, relates to
    pub target_issue: IssueRef,
    pub direction: LinkDirection,
}

pub enum LinkDirection {
    Outward,  // This issue blocks target
    Inward,   // Target blocks this issue
}

pub struct StatusChange {
    pub issue_id: String,
    pub from_status: Status,
    pub to_status: Status,
    pub changed_by: User,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}
```

## Unified Trigger Configuration Schema

### Basic Configuration

```yaml
triggers:
  - type: IssuePlatform
    config:
      # Platform selection
      platform: Jira | Linear | Asana | Monday

      # Common fields (all platforms)
      webhook_secret: ${WEBHOOK_SECRET}

      # Event filtering
      events:
        - issue.created
        - issue.updated
        - comment.added
        - status_change.transitioned

      # Project/Board filtering (optional)
      projects:
        - PROJECT-KEY
        - TEAM-ID
        - board-123

      # Issue type filtering (optional)
      issue_types:
        - bug
        - feature
        - incident

      # Status filtering (optional)
      statuses:
        - To Do
        - In Progress
        - Blocked

      # Priority filtering (optional)
      priorities:
        - Critical
        - High

      # Label filtering (optional)
      labels:
        - needs-triage
        - security

      # Platform-specific configuration
      platform_config:
        # Jira-specific
        jira_url: https://company.atlassian.net
        jira_email: ${JIRA_EMAIL}
        jira_api_token: ${JIRA_API_TOKEN}
        jira_cloud: true  # vs Server/DC

        # Linear-specific
        linear_api_key: ${LINEAR_API_KEY}
        linear_team_id: ${LINEAR_TEAM_ID}

        # Asana-specific
        asana_access_token: ${ASANA_ACCESS_TOKEN}
        asana_workspace_id: ${ASANA_WORKSPACE_ID}

        # Monday-specific
        monday_api_token: ${MONDAY_API_TOKEN}
        monday_board_id: ${MONDAY_BOARD_ID}
```

### Advanced Filtering

```yaml
triggers:
  - type: IssuePlatform
    config:
      platform: Jira
      events: [issue.created]

      # Field-based filters
      field_filters:
        # Only issues with specific custom field values
        custom_field_10001:  # Environment
          - production
          - staging

        # Only issues with estimate
        has_estimate: true

        # Only issues assigned to specific teams
        assignee_team:
          - backend-team
          - security-team

      # Reporter filters
      reporter_filter:
        include: [user@company.com, bot@company.com]
        exclude: [spam@external.com]

      # Time-based filters
      created_within: 24h  # Only new issues
      updated_within: 1h   # Only recent updates

      # SLA tracking
      sla_filter:
        overdue: true
        breach_threshold: 4h

      # Conditional processing
      conditions:
        - field: priority.name
          equals: Critical
        - field: labels
          contains: security
        - field: status.category
          not_equals: done
```

## Unified Agent Context

Agents receive normalized context regardless of platform:

```yaml
# Agent receives this context from any platform
context:
  platform: Jira | Linear | Asana | Monday
  event_type: issue
  action: created

  workspace:
    id: "10000"
    name: Engineering
    url: https://company.atlassian.net
    project_id: PROJ
    project_name: Backend Platform

  issue:
    id: "12345"
    key: PROJ-123
    title: "Critical production bug in auth service"
    description: |
      Users unable to login after deployment...
    issue_type:
      name: bug
    status:
      name: To Do
      category: todo
    priority:
      name: Critical
      level: 1
    assignee: null
    reporter:
      display_name: Jane Developer
      email: jane@company.com
    labels: [production, auth-service, p0]
    created_at: 2025-12-20T10:00:00Z
    updated_at: 2025-12-20T10:00:00Z
    due_date: 2025-12-20T18:00:00Z
    sprint:
      id: "100"
      name: Sprint 42
    custom_fields:
      environment: production
      severity: critical
    url: https://company.atlassian.net/browse/PROJ-123
```

### Using Context in Agents

```yaml
spec:
  agent:
    prompt: |
      You are triaging {{issue.issue_type.name}} {{issue.key}}: "{{issue.title}}"

      Project: {{workspace.project_name}}
      Priority: {{issue.priority.name}}
      Reporter: {{issue.reporter.display_name}}
      Created: {{issue.created_at | date: "%Y-%m-%d %H:%M"}}

      Labels: {{issue.labels | join: ", "}}

      Description:
      {{issue.description}}

      {% if issue.priority.level <= 2 %}
      This is a HIGH PRIORITY issue. Assign immediately.
      {% endif %}

      {% if issue.labels contains "security" %}
      SECURITY ISSUE - Route to security team.
      {% endif %}

      Analyze the issue and suggest:
      1. Correct assignee team
      2. Estimated effort
      3. Related issues to link
```

## Platform Adapter Interface

All platform adapters implement this trait:

```rust
#[async_trait]
pub trait IssuePlatformAdapter: Send + Sync {
    /// Parse incoming webhook payload into normalized event
    async fn parse_webhook(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> Result<IssuePlatformEvent>;

    /// Verify webhook signature
    async fn verify_webhook(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
        secret: &str,
    ) -> Result<bool>;

    // ========== Issue Operations ==========

    /// Create a new issue
    async fn create_issue(
        &self,
        project_id: &str,
        issue: &CreateIssueRequest,
    ) -> Result<Issue>;

    /// Update an existing issue
    async fn update_issue(
        &self,
        issue_id: &str,
        updates: &UpdateIssueRequest,
    ) -> Result<Issue>;

    /// Get issue details
    async fn get_issue(
        &self,
        issue_id: &str,
    ) -> Result<Issue>;

    /// Search issues with JQL/filter
    async fn search_issues(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<Issue>>;

    /// Delete issue (soft delete)
    async fn delete_issue(
        &self,
        issue_id: &str,
    ) -> Result<()>;

    // ========== Comment Operations ==========

    /// Add comment to issue
    async fn add_comment(
        &self,
        issue_id: &str,
        body: &str,
        visibility: CommentVisibility,
    ) -> Result<Comment>;

    /// Update existing comment
    async fn update_comment(
        &self,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment>;

    /// Delete comment
    async fn delete_comment(
        &self,
        comment_id: &str,
    ) -> Result<()>;

    /// Get all comments on issue
    async fn get_comments(
        &self,
        issue_id: &str,
    ) -> Result<Vec<Comment>>;

    // ========== Status Operations ==========

    /// Transition issue to new status
    async fn transition_status(
        &self,
        issue_id: &str,
        to_status: &str,
        comment: Option<&str>,
    ) -> Result<Issue>;

    /// Get available transitions for issue
    async fn get_available_transitions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<StatusTransition>>;

    // ========== Assignment Operations ==========

    /// Assign issue to user
    async fn assign_issue(
        &self,
        issue_id: &str,
        assignee_id: &str,
    ) -> Result<Issue>;

    /// Unassign issue
    async fn unassign_issue(
        &self,
        issue_id: &str,
    ) -> Result<Issue>;

    // ========== Label Operations ==========

    /// Add labels to issue
    async fn add_labels(
        &self,
        issue_id: &str,
        labels: Vec<String>,
    ) -> Result<()>;

    /// Remove labels from issue
    async fn remove_labels(
        &self,
        issue_id: &str,
        labels: Vec<String>,
    ) -> Result<()>;

    /// Set labels (replace all)
    async fn set_labels(
        &self,
        issue_id: &str,
        labels: Vec<String>,
    ) -> Result<()>;

    // ========== Link Operations ==========

    /// Link two issues
    async fn link_issues(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: &str,
    ) -> Result<IssueLink>;

    /// Remove issue link
    async fn unlink_issues(
        &self,
        link_id: &str,
    ) -> Result<()>;

    // ========== Sprint Operations (Jira/Linear) ==========

    /// Add issue to sprint
    async fn add_to_sprint(
        &self,
        issue_id: &str,
        sprint_id: &str,
    ) -> Result<()>;

    /// Remove issue from sprint
    async fn remove_from_sprint(
        &self,
        issue_id: &str,
        sprint_id: &str,
    ) -> Result<()>;

    /// Get active sprints
    async fn get_active_sprints(
        &self,
        project_id: &str,
    ) -> Result<Vec<Sprint>>;

    // ========== Work Logging (Jira-specific) ==========

    /// Log work time on issue
    async fn log_work(
        &self,
        issue_id: &str,
        time_spent: &str,  // "2h 30m" format
        comment: Option<&str>,
    ) -> Result<()>;

    // ========== Metadata Operations ==========

    /// Get available issue types
    async fn get_issue_types(
        &self,
        project_id: &str,
    ) -> Result<Vec<IssueType>>;

    /// Get available statuses
    async fn get_statuses(
        &self,
        project_id: &str,
    ) -> Result<Vec<Status>>;

    /// Get available priorities
    async fn get_priorities(&self) -> Result<Vec<Priority>>;
}

pub struct CreateIssueRequest {
    pub title: String,
    pub description: String,
    pub issue_type: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub parent_id: Option<String>,
    pub sprint_id: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

pub struct SearchQuery {
    pub query: String,           // JQL, Linear filter, etc.
    pub project_id: Option<String>,
    pub issue_types: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub priorities: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub struct StatusTransition {
    pub id: String,
    pub name: String,
    pub to_status: Status,
}
```

### Adapter Factory

```rust
pub struct IssuePlatformAdapterFactory;

impl IssuePlatformAdapterFactory {
    pub fn create(config: &IssuePlatformConfig) -> Result<Box<dyn IssuePlatformAdapter>> {
        match config.platform {
            IssuePlatform::Jira => {
                let adapter = JiraAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
            IssuePlatform::Linear => {
                let adapter = LinearAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
            IssuePlatform::Asana => {
                let adapter = AsanaAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
            IssuePlatform::Monday => {
                let adapter = MondayAdapter::new(&config.platform_config)?;
                Ok(Box::new(adapter))
            }
        }
    }
}
```

## Platform-Specific Considerations

### Jira

**Deployment Models:**
- **Jira Cloud** - atlassian.net, REST API v3, Webhooks v2
- **Jira Server/DC** - Self-hosted, REST API v2, Webhooks v1

**Key Features:**
- **JQL (Jira Query Language)** - Powerful search syntax
- **Custom Fields** - Extensive custom field types (cascading, multi-select)
- **Issue Hierarchies** - Epic → Story → Subtask
- **Workflows** - Complex status transitions with conditions
- **Work Logging** - Time tracking built-in

**Webhook Events:**
```
jira:issue_created → issue.created
jira:issue_updated → issue.updated
jira:issue_deleted → issue.deleted
comment_created → comment.added
comment_updated → comment.updated
issue_transitioned → status_change.transitioned
sprint_started → sprint.started
sprint_closed → sprint.completed
worklog_updated → (custom handling)
```

**Authentication:**
- **Cloud**: Email + API Token, OAuth 2.0
- **Server/DC**: Personal Access Token (PAT), Basic Auth

**API Challenges:**
- Custom fields have numeric IDs (customfield_10001)
- Different field schemas per project
- Rate limiting (Cloud: 10 req/sec, DC: configurable)
- Pagination with startAt/maxResults

### Linear

**Deployment Model:**
- Cloud-only (linear.app)
- GraphQL API

**Key Features:**
- **Teams & Workspaces** - Multi-team support
- **Cycles** - Like sprints but more flexible
- **Triage View** - Built-in issue classification
- **Projects** - Cross-team initiatives
- **SLA Tracking** - First response time built-in

**Webhook Events:**
```
Issue.create → issue.created
Issue.update → issue.updated
Issue.remove → issue.deleted
Comment.create → comment.added
Cycle.create → sprint.started
Cycle.complete → sprint.completed
```

**Authentication:**
- Personal API keys
- OAuth 2.0

**API Characteristics:**
- GraphQL-only (no REST)
- Real-time subscriptions available
- Strong typing
- No rate limits (within reason)

### Asana

**Deployment Model:**
- Cloud-only (asana.com)
- REST API

**Key Features:**
- **Projects & Sections** - Instead of sprints
- **Portfolio View** - Cross-project tracking
- **Custom Fields** - Dropdown, number, text
- **Subtasks** - Hierarchical tasks
- **Approvals** - Built-in approval workflows

**Webhook Events:**
```
task.created → issue.created
task.changed → issue.updated (any field)
task.deleted → issue.deleted
story.added → comment.added (stories are comments)
```

**Authentication:**
- Personal Access Token
- OAuth 2.0

**API Challenges:**
- Stories (comments) are separate from tasks
- Limited webhook event granularity
- Custom fields are workspace-specific
- No native sprint concept

### Monday.com

**Deployment Model:**
- Cloud-only (monday.com)
- GraphQL & REST APIs

**Key Features:**
- **Boards & Items** - Flexible structure
- **Column Types** - Rich field types (status, person, timeline)
- **Automations** - Built-in workflow engine
- **Integrations** - Extensive integration marketplace

**Webhook Events:**
```
create_item → issue.created
change_column_value → issue.updated
create_update → comment.added
delete_item → issue.deleted
```

**Authentication:**
- API Token
- OAuth 2.0

**API Characteristics:**
- GraphQL primary, REST secondary
- Column-based data model (not field-based)
- Rate limiting: 100 req/min (varies by plan)
- Real-time updates via webhooks

## Jira Deep Dive (Primary Platform)

### Webhook Events Reference

**Issue Events:**
```json
{
  "webhookEvent": "jira:issue_created",
  "issue": {
    "id": "10000",
    "key": "PROJ-123",
    "fields": {
      "summary": "Issue title",
      "description": "Issue description",
      "issuetype": { "name": "Bug" },
      "priority": { "name": "High" },
      "status": { "name": "To Do" },
      "assignee": { "emailAddress": "user@company.com" },
      "reporter": { "emailAddress": "reporter@company.com" },
      "labels": ["backend", "urgent"],
      "created": "2025-12-20T10:00:00.000+0000",
      "updated": "2025-12-20T10:00:00.000+0000",
      "duedate": "2025-12-22",
      "customfield_10001": "production"
    }
  }
}
```

**Comment Events:**
```json
{
  "webhookEvent": "comment_created",
  "comment": {
    "id": "10200",
    "body": "This is a comment",
    "author": { "emailAddress": "user@company.com" },
    "created": "2025-12-20T11:00:00.000+0000"
  },
  "issue": { "key": "PROJ-123" }
}
```

**Sprint Events:**
```json
{
  "webhookEvent": "sprint_started",
  "sprint": {
    "id": 100,
    "name": "Sprint 42",
    "state": "active",
    "startDate": "2025-12-20T00:00:00.000+0000",
    "endDate": "2025-12-27T00:00:00.000+0000"
  }
}
```

### Authentication Methods

**1. API Token (Cloud - Recommended)**
```rust
let auth = format!("{}:{}", email, api_token);
let encoded = base64::encode(auth);
headers.insert("Authorization", format!("Basic {}", encoded));
```

**2. Personal Access Token (Server/DC)**
```rust
headers.insert("Authorization", format!("Bearer {}", pat));
```

**3. OAuth 2.0 (Apps)**
- More complex but provides fine-grained permissions
- Requires app registration in Atlassian Developer Console
- Access tokens expire, need refresh tokens

### Common Automation Use Cases

**1. Automated Triage**
```yaml
- name: jira-auto-triage
  triggers:
    - type: IssuePlatform
      config:
        platform: Jira
        events: [issue.created]
  spec:
    agent:
      model: google:gemini-2.5-flash
      prompt: |
        Analyze this {{issue.issue_type.name}}: "{{issue.title}}"

        {{issue.description}}

        Classify and suggest:
        1. Severity (Critical/High/Medium/Low)
        2. Component (auth/api/ui/db)
        3. Team assignment
        4. Estimated effort

      actions:
        - type: IssuePlatform
          operation: update_issue
          issue_id: "{{issue.id}}"
          updates:
            priority: "{{agent_output.severity}}"
            labels: ["{{agent_output.component}}", "triaged"]
            assignee: "{{agent_output.assignee}}"

        - type: IssuePlatform
          operation: add_comment
          issue_id: "{{issue.id}}"
          body: |
            🤖 Auto-triage results:
            - Severity: {{agent_output.severity}}
            - Component: {{agent_output.component}}
            - Assigned to: {{agent_output.assignee}}
            - Estimated: {{agent_output.estimate}}
```

**2. SLA Monitoring**
```yaml
- name: jira-sla-monitor
  triggers:
    - type: Schedule
      cron: "*/15 * * * *"  # Every 15 minutes
  spec:
    agent:
      model: google:gemini-2.5-flash
      prompt: |
        Check for overdue critical issues in Jira

      tools:
        - type: IssuePlatform
          operation: search_issues
          query: |
            priority = Critical AND
            status != Done AND
            created < -4h AND
            assignee is EMPTY

      actions:
        - type: IssuePlatform
          operation: add_comment
          issue_id: "{{issue.id}}"
          body: "⚠️ SLA BREACH: This critical issue has been unassigned for 4+ hours"

        - type: IssuePlatform
          operation: add_labels
          issue_id: "{{issue.id}}"
          labels: ["sla-breach"]
```

**3. Sprint Automation**
```yaml
- name: jira-sprint-auto-assign
  triggers:
    - type: IssuePlatform
      config:
        platform: Jira
        events: [sprint.started]
  spec:
    agent:
      model: google:gemini-2.5-flash
      prompt: |
        Sprint {{sprint.name}} started. Auto-assign unassigned high-priority issues.

      tools:
        - type: IssuePlatform
          operation: search_issues
          query: |
            sprint = {{sprint.id}} AND
            priority in (Critical, High) AND
            assignee is EMPTY

      actions:
        - type: IssuePlatform
          operation: assign_issue
          issue_id: "{{issue.id}}"
          assignee: "{{suggested_assignee}}"
```

**4. Incident Management**
```yaml
- name: jira-incident-response
  triggers:
    - type: IssuePlatform
      config:
        platform: Jira
        events: [issue.created]
        field_filters:
          issue_type: [Incident]
  spec:
    agent:
      model: google:gemini-2.5-flash
      prompt: |
        INCIDENT DETECTED: {{issue.key}} - {{issue.title}}

        {{issue.description}}

        1. Create subtasks for investigation, fix, and postmortem
        2. Link to related incidents
        3. Notify on-call team

      actions:
        - type: IssuePlatform
          operation: create_issue
          parent_id: "{{issue.id}}"
          issue:
            title: "Investigation: {{issue.title}}"
            issue_type: "Subtask"
            assignee: "{{on_call_engineer}}"

        - type: IssuePlatform
          operation: add_labels
          issue_id: "{{issue.id}}"
          labels: ["active-incident", "p0"]

        - type: IssuePlatform
          operation: transition_status
          issue_id: "{{issue.id}}"
          to_status: "In Progress"
```

## Implementation Status

| Platform | Status | Priority | Webhook Events | API Actions | Notes |
|----------|--------|----------|----------------|-------------|-------|
| **Jira** | 🔜 Planned | P1 | 🔜 | 🔜 | Cloud + Server/DC |
| **Linear** | 🔜 Planned | P2 | 🔜 | 🔜 | GraphQL API |
| **Asana** | 📋 Backlog | P3 | - | - | REST API |
| **Monday.com** | 📋 Backlog | P4 | - | - | GraphQL + REST |
| **ClickUp** | 📋 Backlog | P5 | - | - | REST API |
| **GitHub Issues** | ✅ Partial | - | ✅ | ✅ | Via GitHub adapter |

## Extension Guide

### Adding a New Platform

**Step 1: Define Platform Enum**

```rust
// In aof-core/src/issue_platform/mod.rs
pub enum IssuePlatform {
    Jira,
    Linear,
    Asana,
    Monday,
    YourPlatform,  // Add here
}
```

**Step 2: Implement Adapter**

Create `aof-core/src/issue_platform/adapters/your_platform.rs`:

```rust
use super::*;

pub struct YourPlatformAdapter {
    client: reqwest::Client,
    config: PlatformConfig,
}

impl YourPlatformAdapter {
    pub fn new(config: &HashMap<String, String>) -> Result<Self> {
        let api_token = config.get("your_platform_token")
            .ok_or_else(|| anyhow!("Missing your_platform_token"))?;

        Ok(Self {
            client: reqwest::Client::new(),
            config: PlatformConfig { api_token: api_token.clone() },
        })
    }

    fn normalize_issue(&self, raw: &YourPlatformIssue) -> Issue {
        Issue {
            id: raw.id.to_string(),
            key: raw.identifier.clone(),
            title: raw.title.clone(),
            description: raw.description.clone().unwrap_or_default(),
            issue_type: self.map_issue_type(&raw.type_name),
            status: self.map_status(&raw.status),
            priority: self.map_priority(&raw.priority),
            // ... map remaining fields
        }
    }
}

#[async_trait]
impl IssuePlatformAdapter for YourPlatformAdapter {
    async fn parse_webhook(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> Result<IssuePlatformEvent> {
        // 1. Parse webhook JSON
        let payload: YourPlatformWebhook = serde_json::from_slice(body)?;

        // 2. Determine event type and action
        let event_type = match payload.event_type.as_str() {
            "issue.created" => IssueEventType::Issue,
            "comment.added" => IssueEventType::Comment,
            _ => return Err(anyhow!("Unknown event type")),
        };

        let action = match payload.action.as_str() {
            "created" => IssueEventAction::Created,
            "updated" => IssueEventAction::Updated,
            _ => return Err(anyhow!("Unknown action")),
        };

        // 3. Normalize to IssuePlatformEvent
        Ok(IssuePlatformEvent {
            event_type,
            action,
            workspace: self.normalize_workspace(&payload.workspace),
            sender: self.normalize_user(&payload.sender),
            issue: Some(self.normalize_issue(&payload.issue)),
            comment: None,
            sprint: None,
            status_change: None,
        })
    }

    async fn create_issue(
        &self,
        project_id: &str,
        request: &CreateIssueRequest,
    ) -> Result<Issue> {
        let url = format!("https://api.yourplatform.com/projects/{}/issues", project_id);

        let response = self.client
            .post(&url)
            .bearer_auth(&self.config.api_token)
            .json(&serde_json::json!({
                "title": request.title,
                "description": request.description,
                "type": request.issue_type,
                "priority": request.priority,
            }))
            .send()
            .await?;

        let issue: YourPlatformIssue = response.json().await?;
        Ok(self.normalize_issue(&issue))
    }

    // Implement remaining trait methods...
}
```

**Step 3: Add to Factory**

```rust
impl IssuePlatformAdapterFactory {
    pub fn create(config: &IssuePlatformConfig) -> Result<Box<dyn IssuePlatformAdapter>> {
        match config.platform {
            IssuePlatform::Jira => Ok(Box::new(JiraAdapter::new(&config.platform_config)?)),
            IssuePlatform::Linear => Ok(Box::new(LinearAdapter::new(&config.platform_config)?)),
            IssuePlatform::YourPlatform => Ok(Box::new(YourPlatformAdapter::new(&config.platform_config)?)),
        }
    }
}
```

## Best Practices

### 1. Handle Custom Fields Carefully

```yaml
# Use custom_fields mapping for platform-specific fields
spec:
  agent:
    prompt: |
      Environment: {{issue.custom_fields.environment}}
      {% if issue.custom_fields.severity == "critical" %}
      URGENT: This is a critical issue!
      {% endif %}
```

### 2. Graceful Platform Degradation

```rust
// Not all platforms support all features
match adapter.log_work(issue_id, "2h").await {
    Ok(_) => info!("Work logged"),
    Err(e) if e.to_string().contains("not supported") => {
        // Fallback: use comment instead
        adapter.add_comment(issue_id, "Work logged: 2h", CommentVisibility::Public).await?;
    }
    Err(e) => return Err(e),
}
```

### 3. Use Platform-Specific Query Syntax

```yaml
# Jira uses JQL
platform_config:
  jira_search_query: |
    project = PROJ AND
    status = "In Progress" AND
    assignee = currentUser()

# Linear uses filters
platform_config:
  linear_filter: |
    { team: { key: { eq: "ENG" } }, state: { type: { eq: "started" } } }
```

## Security Considerations

### Webhook Verification

```rust
async fn verify_webhook(&self, headers: &HashMap<String, String>, body: &[u8], secret: &str) -> Result<bool> {
    // Jira: No signature (use IP allowlist + secret token)
    let token = headers.get("authorization")
        .ok_or_else(|| anyhow!("Missing authorization header"))?;
    Ok(token == &format!("Bearer {}", secret))

    // Linear: X-Linear-Signature with HMAC-SHA256
    // Asana: X-Hook-Secret with SHA256
    // Monday: X-Monday-Signature with HMAC-SHA256
}
```

### Token Scopes

| Platform | Scope | Purpose |
|----------|-------|---------|
| Jira | `read:jira-work` | Read issues |
| Jira | `write:jira-work` | Create/update issues |
| Linear | Full access | API keys have all permissions |
| Asana | `default` | Full access (no granular scopes) |
| Monday | Read/Write boards | Board-level permissions |

## References

- [Jira Cloud REST API](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Jira Webhook Events](https://developer.atlassian.com/server/jira/platform/webhooks/)
- [Linear API](https://developers.linear.app/docs/graphql/working-with-the-graphql-api)
- [Asana API](https://developers.asana.com/docs/overview)
- [Monday.com API](https://developer.monday.com/api-reference/docs)
- [AOF Trigger System](../concepts/triggers.md)

---

**Status**: Design Document (Implementation planned)
**Last Updated**: 2025-12-20
**Owner**: AOF Core Team
