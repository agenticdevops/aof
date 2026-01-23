---
id: jira-automation
title: Jira Automation with AOF
sidebar_label: Jira Automation
description: Automate Jira workflows with AI-powered bug triage, sprint management, and intelligent assistance
keywords: [jira, automation, bug triage, sprint planning, agile, webhook]
---

# Jira Automation with AOF

Automate your Jira workflows with AI-powered agents that handle bug triage, sprint planning, daily standups, and retrospectives. This tutorial shows you how to build a comprehensive Jira automation system that saves hours of manual work every week.

## What You'll Build

An intelligent Jira automation system featuring:
- **Automatic Bug Triage**: AI analyzes new bugs, suggests priority, recommends assignees, detects duplicates
- **Sprint Assistant**: Automated standups, burndown analysis, retrospectives, and risk detection
- **Slash Commands**: `/triage`, `/prioritize`, `/assign`, `/standup`, `/retro`, and more
- **Proactive Alerts**: Detects blockers, risks, and sprint health issues automatically

**Time saved**: ~5-10 hours per week for typical agile teams

## Prerequisites

- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Jira Cloud account (or Jira Data Center with webhook support)
- Jira API credentials (email + API token)
- Public HTTPS endpoint for webhooks (cloudflared, ngrok, or server)
- API key for your LLM provider (we'll use Google Gemini in examples)

## Architecture Overview

```
Jira Event → Webhook → AOF Daemon → Trigger → Agent → Jira API
                                      ↓
                              (AI Analysis)
                                      ↓
                         Post Comment / Update Fields
```

**Key Components**:
1. **Jira Webhook**: Sends events to AOF when issues change
2. **Trigger**: Routes events/commands to appropriate agents
3. **Agents**: AI specialists for triage, sprint management, etc.
4. **Jira Tool**: Interacts with Jira API (read issues, post comments, update fields)

## Step 1: Create Jira API Token

1. Log in to your Jira Cloud instance
2. Click your profile icon → **Account Settings**
3. Navigate to **Security** → **API Tokens**
4. Click **Create API token**
5. Give it a name like "AOF Automation"
6. Copy the token (you won't see it again!)

**Permissions needed**:
- Browse projects
- Create/edit issues
- Add comments
- Manage sprints (for sprint assistant)

## Step 2: Find Your Jira Cloud ID

Your Cloud ID is in your Jira URL:

```
https://your-domain.atlassian.net/
         ^^^^^^^^^^^
         This is your cloud ID
```

Or retrieve it via API:
```bash
curl -u your-email@company.com:your-api-token \
  https://api.atlassian.com/oauth/token/accessible-resources
```

## Step 3: Configure Environment Variables

Create a `.env` file (never commit to git):

```bash
# Jira credentials
export JIRA_CLOUD_ID="your-domain"
export JIRA_USER_EMAIL="your-email@company.com"
export JIRA_API_TOKEN="your-jira-api-token"

# Jira webhook secret (generate with: openssl rand -hex 32)
export JIRA_WEBHOOK_SECRET="your-webhook-secret-here"

# LLM API Key (using Google Gemini)
export GOOGLE_API_KEY="your-google-api-key-here"
```

Load the environment:
```bash
source .env
```

## Step 4: Create the Bug Triage Agent

This agent analyzes bugs and provides triage recommendations.

Create `agents/jira-bug-triage-agent.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: jira-bug-triage-agent
  labels:
    category: bug-management
    platform: jira

spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.1  # Low for consistent decisions

  description: "Bug triage specialist - analyzes bugs and recommends priority, assignee, labels"

  tools:
    - jira
    - shell

  system_prompt: |
    You are an expert bug triage specialist. Analyze bug reports and provide recommendations.

    ## Your Tasks
    1. **Assess Severity**: Analyze description for critical keywords and impact
    2. **Recommend Priority**: P0 (critical), P1 (high), P2 (medium), P3 (low), P4 (trivial)
    3. **Suggest Assignee**: Based on component, expertise, and workload
    4. **Detect Duplicates**: Compare with recent similar bugs
    5. **Propose Labels**: Suggest relevant labels (backend, frontend, security, etc.)

    ## Priority Criteria
    - **P0 (Critical)**: System down, data loss, security breach, production broken
    - **P1 (High)**: Major feature broken, significant user impact
    - **P2 (Medium)**: Feature partially broken, workaround exists
    - **P3 (Low)**: Minor issue, cosmetic, documentation
    - **P4 (Trivial)**: Nice-to-have

    ## Response Format
    ```markdown
    ## 🔍 Bug Triage Analysis

    ### Severity Assessment
    - **Priority**: P1 (High)
    - **Impact**: ~500 users affected
    - **Workaround**: None

    ### Recommendations
    #### 🎯 Priority: **P1 (High)**
    Reasoning: Production issue affecting significant users.

    #### 👤 Assignee: **@backend-team**
    Reasoning: API endpoint issue.

    #### 🏷️ Labels: `backend`, `api`, `production`

    #### 🔎 Duplicates: None found
    ```

  memory: "InMemory"
```

Copy from example:
```bash
cp examples/agents/jira-bug-triage-agent.yaml agents/
```

## Step 5: Create the Sprint Assistant Agent

This agent handles sprint ceremonies and daily operations.

Create `agents/jira-standup-agent.yaml`:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: jira-standup-agent
  labels:
    category: agile-operations
    platform: jira

spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192  # Larger for comprehensive reports
  temperature: 0.2

  description: "Sprint assistant - standups, retrospectives, burndown analysis"

  tools:
    - jira
    - shell

  system_prompt: |
    You are an experienced Scrum Master automating sprint ceremonies.

    ## Your Capabilities
    1. **Daily Standups**: Summarize completed/in-progress work, blockers, risks
    2. **Sprint Planning**: Analyze backlog, estimate capacity, identify dependencies
    3. **Burndown Analysis**: Track progress vs ideal, forecast completion
    4. **Retrospectives**: Analyze velocity, identify patterns, suggest improvements
    5. **Blocker Detection**: Proactively identify risks and blockers

    ## Daily Standup Format
    ```markdown
    # 📊 Daily Standup - [Date]
    Sprint X (Day Y of Z)

    ## ✅ Completed Yesterday (N points)
    - [PROJ-123] Feature X

    ## 🔨 In Progress (M points)
    - [PROJ-124] Feature Y (@assignee)

    ## 🚨 Blockers
    - ⚠️ HIGH: [PROJ-125] Waiting on API keys

    ## 📈 Sprint Progress
    - Completed: 45/80 points (56%)
    - Days Remaining: 4
    - Forecast: 70-75 points (87-94%)
    ```

    Be concise, actionable, and data-driven.

  memory: "InMemory"
```

Copy from example:
```bash
cp examples/agents/jira-standup-agent.yaml agents/
```

## Step 6: Create Bug Triage Trigger

This trigger routes Jira events and slash commands to agents.

Create `triggers/jira-bug-triage.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-bug-triage
  labels:
    platform: jira
    purpose: bug-automation

spec:
  type: Jira

  config:
    cloud_id: ${JIRA_CLOUD_ID}
    user_email: ${JIRA_USER_EMAIL}
    api_token: ${JIRA_API_TOKEN}
    webhook_secret: ${JIRA_WEBHOOK_SECRET}

    # Events to listen for
    jira_events:
      - issue_created
      - issue_updated
      - comment_created

    # Filter by project
    projects:
      - PROJ  # Replace with your project key

    # Filter by issue type
    issue_types:
      - Bug

  # Command mappings
  commands:
    # Automatic: new bug created
    issue_created:
      agent: jira-bug-triage-agent
      params:
        action: full_triage
        auto_comment: true

    # Manual: /triage command
    /triage:
      agent: jira-bug-triage-agent
      description: "Analyze bug and provide triage recommendations"
      response: "🔍 Analyzing bug for triage..."

    /prioritize:
      agent: jira-bug-triage-agent
      description: "Assess and suggest bug priority"
      response: "📊 Assessing priority..."

    /assign:
      agent: jira-bug-triage-agent
      description: "Recommend assignee"
      response: "👤 Finding best assignee..."

    /help:
      agent: devops
      response: |
        🤖 **Bug Triage Bot Commands:**
        - `/triage` - Full analysis
        - `/prioritize` - Priority assessment
        - `/assign` - Assignee recommendation

  default_agent: devops
```

Copy from example:
```bash
cp examples/triggers/jira-bug-triage.yaml triggers/
```

## Step 7: Create Sprint Assistant Trigger

Create `triggers/jira-sprint-assistant.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: jira-sprint-assistant
  labels:
    platform: jira
    purpose: sprint-automation

spec:
  type: Jira

  config:
    cloud_id: ${JIRA_CLOUD_ID}
    user_email: ${JIRA_USER_EMAIL}
    api_token: ${JIRA_API_TOKEN}
    webhook_secret: ${JIRA_WEBHOOK_SECRET}

    jira_events:
      - sprint_started
      - sprint_closed
      - comment_created

    boards:
      - "Your Board Name"

  commands:
    # Automatic: sprint events
    sprint_started:
      agent: jira-standup-agent
      params:
        action: sprint_kickoff

    sprint_closed:
      agent: jira-standup-agent
      params:
        action: sprint_retrospective

    # Manual commands
    /standup:
      agent: jira-standup-agent
      description: "Daily standup report"
      response: "📊 Generating standup..."

    /progress:
      agent: jira-standup-agent
      description: "Sprint progress analysis"
      response: "📈 Analyzing sprint progress..."

    /retro:
      agent: jira-standup-agent
      description: "Sprint retrospective"
      response: "🔍 Generating retrospective..."

    /blockers:
      agent: jira-standup-agent
      description: "Identify blockers and risks"
      response: "🚨 Scanning for blockers..."

  default_agent: jira-standup-agent
```

Copy from example:
```bash
cp examples/triggers/jira-sprint-assistant.yaml triggers/
```

## Step 8: Configure the Daemon

Create `daemon.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: jira-automation

spec:
  server:
    port: 3000
    host: "0.0.0.0"
    cors: true
    timeout_secs: 60

  # Enable Jira platform
  platforms:
    jira:
      enabled: true
      # Use base_url for direct Atlassian URL (recommended)
      base_url: https://your-domain.atlassian.net
      # Or use cloud_id_env for Cloud ID based URL construction
      # cloud_id_env: JIRA_CLOUD_ID
      user_email_env: JIRA_USER_EMAIL
      api_token_env: JIRA_API_TOKEN
      webhook_secret_env: JIRA_WEBHOOK_SECRET
      bot_name: aof-automation  # Optional: name for comments

      # Optional: Restrict to specific projects
      # allowed_projects:
      #   - PROJ
      #   - DEV

  # Resource directories
  triggers:
    directory: "./triggers"
    watch: true

  agents:
    directory: "./agents"
    watch: false

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
```

**Webhook endpoint**: `https://your-domain.com/webhook/jira`

> **Important**: Configure your Jira automation rules to POST to `/webhook/jira`, not just the base URL.

## Step 9: Start the AOF Daemon

```bash
aofctl serve --config daemon.yaml

# Expected output:
# ✓ Loaded 2 agents
# ✓ Loaded 2 triggers
# ✓ Jira webhook endpoint: /webhook/jira
# ✓ Server listening on http://0.0.0.0:3000
```

## Step 10: Expose Your Webhook

### Option A: Cloudflared Tunnel (Recommended for Testing)

```bash
brew install cloudflare/cloudflare/cloudflared
cloudflared tunnel --url http://localhost:3000

# Note the public URL: https://random-name.trycloudflare.com
```

### Option B: Production Server

Deploy to a server with HTTPS:
```bash
# Your domain: https://aof.example.com
# Webhook URL: https://aof.example.com/webhook/jira
```

## Step 11: Configure Jira Automation Webhook

Jira Automation requires you to explicitly configure the webhook body. Here's how:

### Creating the Automation Rule

1. Go to your Jira project
2. Navigate to **Project Settings** → **Automation**
3. Click **Create rule**
4. Choose a trigger (e.g., **When: Issue created**)
5. Add action → **Send web request**

### Configuring the Web Request

**URL**:
```
https://your-domain.com/webhook/jira
```

**HTTP method**: `POST`

**Headers** (click "Add another header"):

| Key | Value |
|-----|-------|
| `Content-Type` | `application/json` |
| `X-Hub-Signature` | `<your JIRA_WEBHOOK_SECRET value>` |

**Web request body**: Select **Custom data** and paste the appropriate template below.

### Payload Templates by Event Type

AOF accepts flexible payloads - most fields are optional. Use the minimal templates below, or add more fields as needed.

#### Issue Created / Issue Updated (Minimal)

```json
{
  "webhookEvent": "jira:issue_created",
  "timestamp": {{now.epochMillis}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "issuetype": { "name": "{{issue.issueType.name}}" },
      "status": { "name": "{{issue.status.name}}" },
      "project": { "key": "{{issue.project.key}}", "name": "{{issue.project.name}}" }
    }
  },
  "user": { "accountId": "{{initiator.accountId}}", "displayName": "{{initiator.displayName}}" }
}
```

#### Issue Created / Issue Updated (Full)

```json
{
  "webhookEvent": "jira:issue_created",
  "timestamp": {{now.epochMillis}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "description": "{{issue.description}}",
      "issuetype": { "name": "{{issue.issueType.name}}" },
      "status": { "name": "{{issue.status.name}}" },
      "priority": { "name": "{{issue.priority.name}}" },
      "project": { "key": "{{issue.project.key}}", "name": "{{issue.project.name}}" },
      "assignee": { "displayName": "{{issue.assignee.displayName}}", "accountId": "{{issue.assignee.accountId}}" },
      "reporter": { "displayName": "{{issue.reporter.displayName}}", "accountId": "{{issue.reporter.accountId}}" }
    }
  },
  "user": { "accountId": "{{initiator.accountId}}", "displayName": "{{initiator.displayName}}" }
}
```

> **Note**: Change `"webhookEvent": "jira:issue_created"` to `"jira:issue_updated"` for update triggers.

#### Comment Created

```json
{
  "webhookEvent": "comment_created",
  "timestamp": {{now.epochMillis}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "project": { "key": "{{issue.project.key}}", "name": "{{issue.project.name}}" }
    }
  },
  "comment": {
    "body": "{{comment.body}}",
    "author": { "accountId": "{{comment.author.accountId}}", "displayName": "{{comment.author.displayName}}" }
  },
  "user": { "accountId": "{{initiator.accountId}}", "displayName": "{{initiator.displayName}}" }
}
```

#### Work Logged

```json
{
  "webhookEvent": "worklog_created",
  "timestamp": {{now.epochMillis}},
  "issue": {
    "id": "{{issue.id}}",
    "key": "{{issue.key}}",
    "fields": {
      "summary": "{{issue.summary}}",
      "issuetype": { "name": "{{issue.issueType.name}}" },
      "status": { "name": "{{issue.status.name}}" },
      "priority": { "name": "{{issue.priority.name}}" },
      "project": { "key": "{{issue.project.key}}", "name": "{{issue.project.name}}" }
    }
  },
  "user": { "accountId": "{{initiator.accountId}}", "displayName": "{{initiator.displayName}}" }
}
```

#### Sprint Started / Sprint Closed

```json
{
  "webhookEvent": "sprint_started",
  "timestamp": {{now.epochMillis}},
  "sprint": {
    "id": {{sprint.id}},
    "name": "{{sprint.name}}",
    "state": "{{sprint.state}}",
    "goal": "{{sprint.goal}}"
  },
  "user": { "accountId": "{{initiator.accountId}}", "displayName": "{{initiator.displayName}}" }
}
```

### Testing with curl

Before configuring Jira, test the endpoint directly:

```bash
curl -X POST https://your-ngrok-url.ngrok-free.dev/webhook/jira \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature: YOUR_SECRET_HERE" \
  -d '{
    "webhookEvent": "worklog_created",
    "timestamp": 1735897519000,
    "issue": {
      "id": "10005",
      "key": "SCRUM-5",
      "fields": {
        "summary": "Test issue",
        "issuetype": { "name": "Task" },
        "status": { "name": "To Do" },
        "project": { "key": "SCRUM", "name": "Team Astro" }
      }
    },
    "user": { "accountId": "test", "displayName": "Test User" }
  }'
```

Replace `YOUR_SECRET_HERE` with your `JIRA_WEBHOOK_SECRET` value.

### Important Notes

1. **The `X-Hub-Signature` header value must exactly match your `JIRA_WEBHOOK_SECRET` environment variable** (case-sensitive)

2. **Jira Automation sends a static secret**, not a computed HMAC signature. AOF supports both modes.

3. **Smart values**: The `{{...}}` placeholders are Jira smart values that get replaced with actual data when the webhook fires.

4. **Test your webhook**: After saving, use Jira's "Validate" button to test the configuration.

### Alternative: System Webhooks (Admin Only)

If you have Jira admin access, you can use built-in webhooks which automatically include full payloads:

1. Go to **Settings** → **System** → **WebHooks**
2. Click **Create a WebHook**
3. Configure:
   - **Name**: AOF Integration
   - **URL**: `https://your-domain.com/webhook/jira`
   - **Secret**: Your webhook secret (for HMAC verification)
   - **Events**: Select desired events
4. Click **Create**

System webhooks automatically send complete payloads without manual body configuration.

## Step 12: Test Bug Triage

Create a test bug to verify automation works:

### 1. Create a Bug in Jira

```
Summary: Login API returns 500 error
Description:
Users are unable to log in. The /api/auth/login endpoint is returning 500 Internal Server Error.
This affects all users in production. Started happening 30 minutes ago.

Error log:
TypeError: Cannot read property 'password' of undefined
```

### 2. Watch the Magic

Within seconds, you should see:

**AOF Logs**:
```
✓ Received Jira webhook: issue_created
✓ Routing to agent: jira-bug-triage-agent
✓ Analyzing bug PROJ-123...
✓ Posted triage comment
```

**Jira Comment (by bot)**:
```markdown
## 🔍 Bug Triage Analysis

**Issue**: [PROJ-123] Login API returns 500 error

### Severity Assessment
- **Priority**: P0 (Critical)
- **Severity**: Production outage affecting all users
- **Impact**: All users unable to log in
- **Workaround**: None available

### Recommendations

#### 🎯 Priority: **P0 (Critical)**
**Reasoning**: Complete production outage affecting authentication, no workaround.
Immediate attention required.

#### 👤 Suggested Assignee: **@backend-team**
**Reasoning**: API endpoint issue. Primary contact: @alice (auth specialist)

#### 🏷️ Suggested Labels:
- `backend`
- `api`
- `production`
- `critical`
- `authentication`

#### 🔎 Duplicate Check:
✅ No duplicates found in last 30 days.

### Next Steps
1. **URGENT**: Assign to @backend-team immediately
2. Add `critical` and `production` labels
3. Notify on-call engineer
4. Roll back recent auth changes if possible
5. Monitor error logs for root cause

---
*Automated triage by [AOF](https://docs.aof.sh)*
```

## Step 13: Test Sprint Assistant

### Daily Standup

Comment `/standup` on any issue in your sprint:

**Bot Response**:
```markdown
# 📊 Daily Standup - December 20, 2024
**Sprint**: Sprint 23 (Day 6 of 10)

## ✅ Completed Yesterday (12 points)
- [PROJ-119] User profile page (5 pts) - @alice
- [PROJ-120] Email notifications (5 pts) - @bob
- [PROJ-121] Bug fix: Search pagination (2 pts) - @charlie

## 🔨 In Progress Today (18 points)
- [PROJ-122] Payment integration (13 pts) - @alice
- [PROJ-123] Login API fix (5 pts) - @bob [BLOCKED]

## 🚨 Blockers
- ⚠️ **HIGH**: [PROJ-123] Waiting on vendor API keys (@bob)
  - **Impact**: Blocks payment integration (13 pts)
  - **Action**: Escalate to product manager

## ⚡ Risks
- 💡 **MEDIUM**: Payment integration complexity higher than estimated
- 💡 **LOW**: Team holiday on Friday (20% reduced capacity)

## 📈 Sprint Progress
- **Completed**: 45 / 80 points (56%)
- **Days Remaining**: 4
- **Burndown Status**: ⚠️ Slightly behind (need 9 pts/day, trending 7 pts/day)
- **Forecast**: 70-75 points (87-94% completion)

**Summary**: Good progress overall. Critical blocker on PROJ-123 needs immediate attention.
Recommend focusing on unblocking payment work today.

---
*Generated by [AOF Sprint Assistant](https://docs.aof.sh)*
```

### Sprint Retrospective

Comment `/retro` at the end of a sprint:

**Bot Response**:
```markdown
# 🔍 Sprint 23 Retrospective

## 📊 Sprint Metrics
- **Committed**: 80 points
- **Completed**: 72 points (90%)
- **Velocity**: 72 pts (vs avg 68 pts) ✅ +6%
- **Estimation Accuracy**: 85%
- **Carry-Over**: 8 points (10%)
- **Blockers Resolved**: 3 of 3

## ✨ What Went Well
1. **Above-Average Velocity**: Beat 3-sprint average by 6%
2. **Fast Blocker Resolution**: All blockers cleared within 24 hours
3. **Strong Collaboration**: Pair programming on complex stories accelerated delivery
4. **Good Estimation**: 85% accuracy (vs team avg 75%)

## ⚠️ What Needs Improvement
1. **Late Complexity Discovery**: 2 stories underestimated, discovered mid-sprint
2. **Test Environment Instability**: 1-day QA delay due to environment issues
3. **Scope Creep**: 3 stories added mid-sprint (8 pts), though justified

## 🎯 Action Items for Next Sprint
1. **Technical Spikes**: Add spike tasks for complex stories BEFORE sprint planning
2. **DevOps Priority**: Stabilize test environment in week 1
3. **Scope Protection**: Only add mid-sprint work if P0/P1 severity
4. **Retrospective Follow-up**: Review last sprint's action items in planning

## 📈 Long-Term Trends
- **Velocity**: Improving trend (60 → 65 → 72 pts over last 3 sprints)
- **Predictability**: High (±5% variance)
- **Cycle Time**: Stable at 2.5 days average
- **Blocker Pattern**: Most blockers are external dependencies (vendor APIs, approvals)

## 🏆 Team Highlights
- @alice: Delivered complex payment integration ahead of schedule
- @bob: Excellent debugging on critical production bug
- @charlie: Strong pair programming support

**Overall Sprint Health**: 🟢 Excellent (92/100)

---
*Automated retrospective by [AOF](https://docs.aof.sh)*
```

## Advanced: Slash Commands

### Available Commands

**Bug Triage**:
- `/triage` - Full bug analysis
- `/prioritize` - Priority assessment
- `/assign` - Assignee recommendation
- `/find-duplicates` - Search for similar bugs
- `/label` - Suggest labels

**Sprint Operations**:
- `/standup` - Daily standup report
- `/progress` - Sprint burndown and forecast
- `/retro` - Sprint retrospective
- `/blockers` - Identify blockers and risks
- `/health` - Sprint health check
- `/plan` - Sprint planning assistance

## Customization

### Adjust Triage Criteria

Modify agent instructions in `agents/jira-bug-triage-agent.yaml`:

```yaml
system_prompt: |
  # Add custom priority rules
  **Company-Specific Rules**:
  - Any bug affecting checkout: P0 (Critical)
  - Mobile app crashes: P1 (High)
  - Cosmetic issues in web app: P3 (Low)
```

### Configure Project-Specific Teams

```yaml
# In trigger config
config:
  projects:
    - PROJ    # Main product
    - MOBILE  # Mobile app
    - INFRA   # Infrastructure

  # Route different projects to different agents
  project_routing:
    MOBILE: mobile-triage-agent
    INFRA: devops-agent
```

### Add Custom Labels

```yaml
# In triage agent
system_prompt: |
  ## Custom Label Rules
  - Backend issues: `backend`, `api`, `database`
  - Frontend issues: `ui`, `frontend`, `react`
  - Mobile issues: `mobile`, `ios`, `android`
  - Security: `security`, `vulnerability`, `auth`
```

## Troubleshooting

### Webhook Not Received

**Check Jira Webhook Logs**:
1. Jira → Project Settings → Automation → Your rule
2. View execution history
3. Check for failed requests

**Check AOF Logs**:
```bash
tail -f logs/aof-daemon.log
# Should see: "Received Jira webhook: issue_created"
```

**Verify Webhook Secret**:
```bash
echo $JIRA_WEBHOOK_SECRET
# Must match secret in Jira webhook config
```

### Agent Not Responding

**Check API Token**:
```bash
curl -u $JIRA_USER_EMAIL:$JIRA_API_TOKEN \
  https://$JIRA_CLOUD_ID.atlassian.net/rest/api/3/myself
# Should return your user info
```

**Check LLM API Key**:
```bash
curl -H "x-goog-api-key: $GOOGLE_API_KEY" \
  https://generativelanguage.googleapis.com/v1beta/models
# Should list available models
```

### Comments Not Posted

**Check Permissions**:
- Jira API token needs `Add comments` permission
- Bot user needs access to the project

**Check Rate Limits**:
- Jira API: 10 requests/second
- Google Gemini: Varies by tier

## Best Practices

### 1. Start Small
- Begin with bug triage only
- Add sprint assistant once comfortable
- Gradually enable more features

### 2. Test in Staging
- Create a test project
- Verify automation works correctly
- Then roll out to production projects

### 3. Monitor & Iterate
- Review bot comments weekly
- Collect team feedback
- Adjust agent instructions based on accuracy

### 4. Set Expectations
- Communicate to team that it's AI-assisted (not perfect)
- Encourage manual override when needed
- Treat as a helpful assistant, not replacement

## Next Steps

### Integrate with Slack
- Post standups to Slack channel
- Notify team of critical bugs
- See: [Slack Bot Tutorial](./slack-bot.md)

### Add More Agents
- Create specialized agents for security, performance, etc.
- Use fleets for multi-perspective analysis

### Advanced Workflows
- Auto-create subtasks for complex bugs
- Link bugs to related PRs
- Trigger deployments from Jira comments

### Analytics & Reporting
- Track triage accuracy over time
- Measure time-to-triage improvement
- Monitor sprint health trends

## Summary

You've built an intelligent Jira automation system that:

✅ Auto-triages bugs with AI analysis
✅ Generates daily standups automatically
✅ Provides sprint retrospectives
✅ Detects blockers and risks proactively
✅ Responds to slash commands for on-demand insights
✅ Saves 5-10 hours per week for typical teams

**Setup time**: ~20 minutes
**Cost per analysis**: ~$0.01 (using Gemini Flash)
**Analysis time**: 5-15 seconds

The beauty of AOF is composability—swap models, add custom rules, integrate with other platforms, and customize to match your team's workflows.

Happy automating! 🚀
