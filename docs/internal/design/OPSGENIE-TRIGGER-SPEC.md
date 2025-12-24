# Opsgenie Trigger Platform - Internal Design Specification

**Version:** 1.0
**Date:** 2025-12-23
**Status:** Draft
**Owner:** AOF Core Team

## Table of Contents

1. [Overview](#overview)
2. [Platform Background](#platform-background)
3. [Webhook Events](#webhook-events)
4. [Configuration](#configuration)
5. [Implementation Details](#implementation-details)
6. [Response Handling](#response-handling)
7. [Example YAML Resources](#example-yaml-resources)
8. [Example Agent Integration](#example-agent-integration)
9. [API Reference](#api-reference)
10. [Testing Strategy](#testing-strategy)

---

## Overview

### What is Opsgenie?

Opsgenie is Atlassian's incident management and on-call scheduling platform. It provides:
- **Alert Management**: Centralized alert routing, escalation, and tracking
- **On-Call Scheduling**: Team rotation management and escalation policies
- **Incident Response**: Coordinated incident response workflows
- **Integration Hub**: 200+ integrations with monitoring and DevOps tools
- **Mobile-First**: Critical alerts delivered to mobile devices with rich actions

### Why Integrate with AOF?

Opsgenie integration enables AOF agents to:
1. **Automated Incident Response**: Trigger agent workflows on critical alerts
2. **Intelligent Triage**: Use AI to analyze alert context and suggest actions
3. **Self-Healing**: Execute remediation workflows automatically
4. **Escalation Intelligence**: Smart escalation based on alert patterns
5. **Post-Incident Analysis**: Automated RCA and documentation

### Use Cases

- **Auto-Remediation**: P1 database alert → Agent runs diagnostic → Auto-scales cluster
- **Smart Escalation**: Alert patterns detected → Agent analyzes → Escalates with context
- **Incident Commander**: Alert created → Agent coordinates response → Updates stakeholders
- **Post-Mortem**: Incident closed → Agent generates timeline → Creates Jira ticket

---

## Platform Background

### Opsgenie Alert Lifecycle

```
Created → Acknowledged → In Progress → Resolved → Closed
    ↓          ↓              ↓            ↓
  Snoozed   Escalated    AddNote    Custom Action
```

### Alert vs Incident Model

**Alerts:**
- Individual notification events (e.g., "CPU > 90%")
- Multiple alerts can be aggregated into incidents
- Can be acknowledged, closed, snoozed, escalated

**Incidents:**
- Higher-level problems requiring coordination (e.g., "Production Outage")
- Contain multiple alerts
- Have incident timeline, stakeholders, and status pages

**AOF Integration Strategy:**
- Primary: Alert-based triggers (most common use case)
- Secondary: Incident-based triggers (for complex workflows)
- Use alert metadata to determine if it's part of an incident

---

## Webhook Events

### Supported Alert Actions

Opsgenie sends webhooks for the following alert actions:

#### 1. Alert Created (`alert-created`)

**When:** New alert is created in Opsgenie
**Trigger Condition:** Alert matches configured filters (tags, priority, teams)
**Use Case:** Start automated diagnostic workflow

```yaml
# Example Payload Structure
{
  "action": "Create",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "message": "CPU usage above 90%",
    "priority": "P1",
    "tags": ["production", "database"],
    "source": "datadog",
    "responders": [
      {"type": "team", "id": "team-id", "name": "Platform"}
    ],
    "alias": "cpu-high-db-prod-01",
    "description": "CPU spiked to 95% on db-prod-01",
    "entity": "db-prod-01",
    "createdAt": 1640000000000
  },
  "integrationId": "integration-id",
  "integrationName": "AOF"
}
```

#### 2. Alert Acknowledged (`alert-acknowledged`)

**When:** Alert is manually or automatically acknowledged
**Trigger Condition:** Acknowledgement notes contain agent command
**Use Case:** Coordinate handoff between human and agent

```yaml
# Example Payload Structure
{
  "action": "Acknowledge",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "tinyId": "123",
    "owner": "user@example.com",
    "note": "/run diagnostics --verbose"
  },
  "source": {"name": "John Doe", "type": "user"}
}
```

#### 3. Alert Closed (`alert-closed`)

**When:** Alert is resolved and closed
**Trigger Condition:** Close trigger enabled
**Use Case:** Trigger post-incident cleanup, documentation

```yaml
# Example Payload Structure
{
  "action": "Close",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "note": "Issue auto-resolved by scaling",
    "closedAt": 1640001000000
  },
  "source": {"name": "AOF Agent", "type": "integration"}
}
```

#### 4. Alert Escalated (`alert-escalated`)

**When:** Alert escalated to next level in escalation policy
**Trigger Condition:** Escalation event enabled
**Use Case:** Trigger additional diagnostics, expand remediation scope

```yaml
# Example Payload Structure
{
  "action": "Escalate",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "escalation": {
      "name": "L2 Platform Team",
      "nextLevel": "L2"
    }
  }
}
```

#### 5. Add Note (`alert-note-added`)

**When:** Note/comment added to alert
**Trigger Condition:** Note contains agent command prefix
**Use Case:** Interactive agent control during incident

```yaml
# Example Payload Structure
{
  "action": "AddNote",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "note": "/run fix-database-connection",
    "user": "user@example.com"
  }
}
```

#### 6. Custom Action (`alert-custom-action`)

**When:** Custom action button clicked in Opsgenie UI/mobile
**Trigger Condition:** Action registered via integration
**Use Case:** One-click remediation workflows

```yaml
# Example Payload Structure
{
  "action": "CustomAction",
  "alert": {
    "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
    "customAction": {
      "name": "Restart Service",
      "actionId": "restart-action-id"
    }
  }
}
```

### Event Filtering

Opsgenie allows filtering at webhook configuration:
- **Priority Filter**: Only P1/P2 alerts
- **Tag Filter**: Only alerts with specific tags
- **Team Filter**: Only alerts assigned to specific teams
- **Source Filter**: Only alerts from specific integrations

**AOF Strategy:**
- Configure broad webhook at Opsgenie level (e.g., all P1/P2)
- Fine-grained filtering in AgentFlow YAML (allows dynamic updates)

---

## Configuration

### OpsgenieConfig Struct

```rust
/// Opsgenie platform configuration
///
/// Supports both Opsgenie Cloud (US/EU) and on-premises instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsgenieConfig {
    /// Opsgenie API base URL
    /// - US: https://api.opsgenie.com
    /// - EU: https://api.eu.opsgenie.com
    /// - On-prem: https://opsgenie.yourcompany.com
    pub api_url: String,

    /// API key for Opsgenie integration
    /// Generate from: Opsgenie → Settings → Integrations → API
    pub api_key: String,

    /// Webhook verification token (optional but recommended)
    /// Used to verify webhook authenticity
    #[serde(default)]
    pub webhook_token: Option<String>,

    /// Bot/Integration name for identification
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Alert action filters (which events to process)
    /// Options: ["Create", "Acknowledge", "Close", "Escalate", "AddNote", "CustomAction"]
    /// Default: all actions if empty
    #[serde(default)]
    pub allowed_actions: Vec<String>,

    /// Priority filter (e.g., ["P1", "P2"])
    /// Only process alerts matching these priorities
    #[serde(default)]
    pub priority_filter: Option<Vec<String>>,

    /// Tag filter (e.g., ["production", "critical"])
    /// Only process alerts with ALL these tags
    #[serde(default)]
    pub tag_filter: Option<Vec<String>>,

    /// Team filter (team IDs or names)
    /// Only process alerts assigned to these teams
    #[serde(default)]
    pub team_filter: Option<Vec<String>>,

    /// Source filter (e.g., ["datadog", "prometheus"])
    /// Only process alerts from these sources
    #[serde(default)]
    pub source_filter: Option<Vec<String>>,

    /// Enable posting notes to alerts
    #[serde(default = "default_true")]
    pub enable_notes: bool,

    /// Enable updating alert fields
    #[serde(default = "default_true")]
    pub enable_updates: bool,

    /// Enable closing alerts
    #[serde(default = "default_true")]
    pub enable_close: bool,

    /// Enable acknowledging alerts
    #[serde(default = "default_true")]
    pub enable_acknowledge: bool,

    /// Enable creating new alerts
    #[serde(default = "default_true")]
    pub enable_create: bool,

    /// Rate limit: max API calls per minute
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,

    /// HTTP timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}

fn default_true() -> bool {
    true
}

fn default_rate_limit() -> u32 {
    600 // Opsgenie limit: 600 requests/minute
}

fn default_timeout() -> u64 {
    30
}
```

### Environment Variable Resolution

Support for environment variable references in configuration:

```yaml
# Example with env vars
type: Opsgenie
config:
  api_url: https://api.opsgenie.com
  api_key_env: OPSGENIE_API_KEY          # Read from env
  webhook_token_env: OPSGENIE_WEBHOOK_TOKEN
  allowed_actions:
    - Create
    - AddNote
  priority_filter:
    - P1
    - P2
```

### API Key Setup

**Step 1: Create API Integration**
1. Go to Opsgenie → Settings → Integrations
2. Click "Add integration" → Search "API"
3. Name: "AOF Integration"
4. Check permissions:
   - Read alerts
   - Create alerts
   - Update alerts
   - Add notes
   - Close alerts
   - Acknowledge alerts
5. Copy API key

**Step 2: Configure Webhook**
1. In same integration, scroll to "Webhook URL"
2. Enter: `https://your-aof-server.com/triggers/opsgenie`
3. Select actions to send:
   - Alert Created
   - Alert Acknowledged
   - Alert Closed
   - Alert Note Added
   - Alert Custom Action
4. Add filters (optional): priority, tags, teams
5. Save integration

**Step 3: Test Webhook**
```bash
# Opsgenie provides webhook test button
# Or manually create test alert:
curl -X POST https://api.opsgenie.com/v2/alerts \
  -H "Authorization: GenieKey $OPSGENIE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Test alert for AOF",
    "tags": ["test", "aof"],
    "priority": "P3"
  }'
```

---

## Implementation Details

### Webhook Authentication

Opsgenie does NOT use HMAC signatures like Slack/GitHub. Instead:

1. **Integration Token Method** (Recommended):
   - Opsgenie includes `integrationId` and `integrationName` in payload
   - Platform validates these match configured integration
   - No signature header

2. **API Key Validation Method**:
   - After receiving webhook, make API call to fetch alert details
   - If API call succeeds, webhook is authentic
   - Adds latency but provides strong validation

3. **Network Filtering Method**:
   - Restrict webhook endpoint to Opsgenie IP ranges
   - US: 44.234.97.0/24, 52.26.112.0/24, 52.89.34.0/24
   - EU: 18.185.127.0/24, 3.122.36.0/24

**AOF Implementation Strategy:**
- Use Integration Token validation (fastest, sufficient for most cases)
- Optional: Enable API callback validation for high-security environments
- Configuration flag: `verify_with_api_callback: bool`

```rust
async fn verify_signature(&self, payload: &[u8], _signature: &str) -> bool {
    // Opsgenie doesn't use signatures in headers
    // Instead, validate integration ID from payload

    let webhook: OpsgenieWebhookPayload = match serde_json::from_slice(payload) {
        Ok(w) => w,
        Err(_) => return false,
    };

    // Validate integration ID matches configuration
    if let Some(ref expected_id) = self.config.integration_id {
        if webhook.integration_id != *expected_id {
            warn!("Integration ID mismatch");
            return false;
        }
    }

    // Optional: Verify by fetching alert from API
    if self.config.verify_with_api_callback {
        return self.verify_alert_via_api(&webhook.alert.alert_id).await;
    }

    true
}

async fn verify_alert_via_api(&self, alert_id: &str) -> bool {
    let url = format!("{}/v2/alerts/{}", self.config.api_url, alert_id);

    let response = self.client
        .get(&url)
        .header("Authorization", format!("GenieKey {}", self.config.api_key))
        .send()
        .await;

    response.map(|r| r.status().is_success()).unwrap_or(false)
}
```

### Payload Parsing

**Webhook Payload Structure:**

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpsgenieWebhookPayload {
    /// Action type: Create, Acknowledge, Close, Escalate, AddNote, CustomAction
    pub action: String,

    /// Alert details
    pub alert: OpsgenieAlert,

    /// Integration metadata
    #[serde(default)]
    pub integration_id: Option<String>,

    #[serde(default)]
    pub integration_name: Option<String>,

    /// Source of action (user, integration, schedule)
    #[serde(default)]
    pub source: Option<OpsgenieSource>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpsgenieAlert {
    /// Alert UUID
    pub alert_id: String,

    /// Short numeric ID (for display)
    #[serde(default)]
    pub tiny_id: Option<String>,

    /// Alert message/title
    pub message: String,

    /// Alert description (can be very long)
    #[serde(default)]
    pub description: Option<String>,

    /// Priority: P1, P2, P3, P4, P5
    #[serde(default)]
    pub priority: Option<String>,

    /// Alert tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Source system (e.g., "datadog", "prometheus")
    #[serde(default)]
    pub source: Option<String>,

    /// Entity (resource identifier, e.g., "db-prod-01")
    #[serde(default)]
    pub entity: Option<String>,

    /// Alert alias (deduplication key)
    #[serde(default)]
    pub alias: Option<String>,

    /// Current owner (who acknowledged)
    #[serde(default)]
    pub owner: Option<String>,

    /// Responders (teams/users assigned)
    #[serde(default)]
    pub responders: Vec<OpsgenieResponder>,

    /// Custom fields (key-value pairs)
    #[serde(default)]
    pub details: HashMap<String, String>,

    /// Note added (for AddNote action)
    #[serde(default)]
    pub note: Option<String>,

    /// Timestamps
    #[serde(default)]
    pub created_at: Option<i64>,

    #[serde(default)]
    pub updated_at: Option<i64>,

    #[serde(default)]
    pub acknowledged_at: Option<i64>,

    #[serde(default)]
    pub closed_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpsgenieResponder {
    /// Type: team, user, escalation, schedule
    #[serde(rename = "type")]
    pub responder_type: String,

    /// ID
    pub id: String,

    /// Name
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpsgenieSource {
    /// Source name (username or integration name)
    pub name: String,

    /// Source type: user, integration, schedule
    #[serde(rename = "type")]
    pub source_type: String,
}
```

### Building TriggerMessage

Convert Opsgenie webhook to AOF TriggerMessage:

```rust
fn build_trigger_message(
    &self,
    payload: &OpsgenieWebhookPayload,
) -> Result<TriggerMessage, PlatformError> {
    let alert = &payload.alert;

    // Build message text based on action
    let text = match payload.action.as_str() {
        "Create" => {
            format!(
                "alert:created:{} {} - {}",
                alert.alert_id,
                alert.message,
                alert.description.as_deref().unwrap_or("")
            )
        }
        "Acknowledge" => {
            let note = alert.note.as_deref().unwrap_or("");
            format!("alert:acknowledged:{} {}", alert.alert_id, note)
        }
        "Close" => {
            let note = alert.note.as_deref().unwrap_or("");
            format!("alert:closed:{} {}", alert.alert_id, note)
        }
        "AddNote" => {
            let note = alert.note.as_deref().unwrap_or("");
            format!("alert:note:{} {}", alert.alert_id, note)
        }
        "Escalate" => {
            format!("alert:escalated:{}", alert.alert_id)
        }
        "CustomAction" => {
            format!("alert:action:{}", alert.alert_id)
        }
        _ => format!("alert:{}:{}", payload.action.to_lowercase(), alert.alert_id),
    };

    // Extract user from source
    let (user_id, user_name) = if let Some(ref source) = payload.source {
        (source.name.clone(), Some(source.name.clone()))
    } else {
        ("system".to_string(), Some("Opsgenie System".to_string()))
    };

    let trigger_user = TriggerUser {
        id: user_id.clone(),
        username: Some(user_name.clone().unwrap_or_default()),
        display_name: user_name,
        is_bot: payload.source.as_ref().map(|s| s.source_type == "integration").unwrap_or(false),
    };

    // Channel ID is the alert ID (for threading)
    let channel_id = alert.alert_id.clone();

    // Build metadata with ALL alert fields
    let mut metadata = HashMap::new();
    metadata.insert("action".to_string(), serde_json::json!(payload.action));
    metadata.insert("alert_id".to_string(), serde_json::json!(alert.alert_id));

    if let Some(ref tiny_id) = alert.tiny_id {
        metadata.insert("tiny_id".to_string(), serde_json::json!(tiny_id));
    }

    metadata.insert("message".to_string(), serde_json::json!(alert.message));

    if let Some(ref priority) = alert.priority {
        metadata.insert("priority".to_string(), serde_json::json!(priority));
    }

    metadata.insert("tags".to_string(), serde_json::json!(alert.tags));

    if let Some(ref source) = alert.source {
        metadata.insert("source".to_string(), serde_json::json!(source));
    }

    if let Some(ref entity) = alert.entity {
        metadata.insert("entity".to_string(), serde_json::json!(entity));
    }

    if let Some(ref alias) = alert.alias {
        metadata.insert("alias".to_string(), serde_json::json!(alias));
    }

    // Store all custom details
    if !alert.details.is_empty() {
        metadata.insert("details".to_string(), serde_json::json!(alert.details));
    }

    // Store responders
    metadata.insert("responders".to_string(), serde_json::json!(alert.responders));

    // Message ID from alert ID and action
    let message_id = format!("opsgenie-{}-{}", alert.alert_id, payload.action);

    // Thread ID is alert ID (all actions on same alert are threaded)
    let thread_id = Some(alert.alert_id.clone());

    // Timestamp from created_at or now
    let timestamp = alert.created_at
        .and_then(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0))
        .unwrap_or_else(chrono::Utc::now);

    Ok(TriggerMessage {
        id: message_id,
        platform: "opsgenie".to_string(),
        channel_id,
        user: trigger_user,
        text,
        timestamp,
        metadata,
        thread_id,
        reply_to: None,
    })
}
```

---

## Response Handling

### API Methods

The `OpsgeniePlatform` struct provides these public methods:

#### 1. Add Note to Alert

```rust
/// Add a note/comment to an alert
///
/// # Arguments
/// * `alert_id` - Alert UUID
/// * `note` - Note text (supports Markdown)
///
/// # Returns
/// Note ID on success
pub async fn add_note(&self, alert_id: &str, note: &str) -> Result<String, PlatformError>
```

**API Call:**
```
POST /v2/alerts/{alertId}/notes
Authorization: GenieKey {api_key}
Content-Type: application/json

{
  "note": "Agent diagnosis: High CPU due to runaway query. Killing PID 12345."
}
```

#### 2. Acknowledge Alert

```rust
/// Acknowledge an alert
///
/// # Arguments
/// * `alert_id` - Alert UUID
/// * `note` - Optional acknowledgement note
///
/// # Returns
/// Success
pub async fn acknowledge_alert(
    &self,
    alert_id: &str,
    note: Option<&str>,
) -> Result<(), PlatformError>
```

**API Call:**
```
POST /v2/alerts/{alertId}/acknowledge
Authorization: GenieKey {api_key}
Content-Type: application/json

{
  "user": "aofbot@example.com",
  "note": "Agent is investigating this issue"
}
```

#### 3. Close Alert

```rust
/// Close an alert
///
/// # Arguments
/// * `alert_id` - Alert UUID
/// * `note` - Optional close note
///
/// # Returns
/// Success
pub async fn close_alert(
    &self,
    alert_id: &str,
    note: Option<&str>,
) -> Result<(), PlatformError>
```

**API Call:**
```
POST /v2/alerts/{alertId}/close
Authorization: GenieKey {api_key}
Content-Type: application/json

{
  "user": "aofbot@example.com",
  "note": "Auto-remediated: Scaled cluster from 3 to 5 nodes"
}
```

#### 4. Create New Alert

```rust
/// Create a new alert
///
/// # Arguments
/// * `message` - Alert message/title
/// * `description` - Detailed description
/// * `priority` - P1/P2/P3/P4/P5
/// * `tags` - Alert tags
/// * `details` - Custom fields
///
/// # Returns
/// Alert ID
pub async fn create_alert(
    &self,
    message: &str,
    description: Option<&str>,
    priority: Option<&str>,
    tags: Vec<String>,
    details: HashMap<String, String>,
) -> Result<String, PlatformError>
```

**API Call:**
```
POST /v2/alerts
Authorization: GenieKey {api_key}
Content-Type: application/json

{
  "message": "Agent detected anomaly in database metrics",
  "description": "Query latency increased 300% in last 5 minutes",
  "priority": "P2",
  "tags": ["agent-detected", "database", "performance"],
  "source": "aof-agent",
  "entity": "db-prod-cluster",
  "details": {
    "query_latency_p95": "2.5s",
    "baseline_latency_p95": "0.8s",
    "increase_pct": "312%"
  }
}
```

#### 5. Add Responder

```rust
/// Add a responder (team/user) to an alert
///
/// # Arguments
/// * `alert_id` - Alert UUID
/// * `responder_type` - "team" or "user"
/// * `responder_id` - Team/user ID
///
/// # Returns
/// Success
pub async fn add_responder(
    &self,
    alert_id: &str,
    responder_type: &str,
    responder_id: &str,
) -> Result<(), PlatformError>
```

#### 6. Escalate to Next Level

```rust
/// Escalate alert to next level in escalation policy
///
/// # Arguments
/// * `alert_id` - Alert UUID
/// * `escalation_id` - Escalation policy ID
/// * `note` - Optional escalation note
///
/// # Returns
/// Success
pub async fn escalate_alert(
    &self,
    alert_id: &str,
    escalation_id: &str,
    note: Option<&str>,
) -> Result<(), PlatformError>
```

### TriggerPlatform Implementation

```rust
#[async_trait]
impl TriggerPlatform for OpsgeniePlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // 1. Parse webhook payload
        let payload = self.parse_webhook_payload(raw)?;

        // 2. Verify integration ID (if configured)
        if !self.verify_integration(&payload) {
            return Err(PlatformError::InvalidSignature(
                "Invalid integration ID".to_string()
            ));
        }

        // 3. Apply action filter
        if !self.is_action_allowed(&payload.action) {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // 4. Apply priority filter
        if let Some(ref priority) = payload.alert.priority {
            if !self.is_priority_allowed(priority) {
                return Err(PlatformError::UnsupportedMessageType);
            }
        }

        // 5. Apply tag filter
        if !self.is_tag_match(&payload.alert.tags) {
            return Err(PlatformError::UnsupportedMessageType);
        }

        // 6. Apply source filter
        if let Some(ref source) = payload.alert.source {
            if !self.is_source_allowed(source) {
                return Err(PlatformError::UnsupportedMessageType);
            }
        }

        // 7. Build TriggerMessage
        self.build_trigger_message(&payload)
    }

    async fn send_response(
        &self,
        channel: &str, // Alert ID
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        let note = self.format_response_text(&response);
        self.add_note(channel, &note).await?;
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "opsgenie"
    }

    async fn verify_signature(&self, payload: &[u8], _signature: &str) -> bool {
        // Opsgenie uses integration ID validation, not HMAC
        let webhook: OpsgenieWebhookPayload = match serde_json::from_slice(payload) {
            Ok(w) => w,
            Err(_) => return false,
        };

        self.verify_integration(&webhook)
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // All alert actions are threaded by alert ID
    }

    fn supports_interactive(&self) -> bool {
        true // Custom actions supported
    }

    fn supports_files(&self) -> bool {
        false // Opsgenie API doesn't support file attachments via notes
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

---

## Example YAML Resources

### Example 1: Auto-Acknowledge Critical Alerts

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: opsgenie-auto-ack-critical
  namespace: production

spec:
  description: Auto-acknowledge P1 alerts and start diagnostic workflow

  trigger:
    type: Opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key_env: OPSGENIE_API_KEY
      webhook_token_env: OPSGENIE_WEBHOOK_TOKEN
      bot_name: aof-auto-responder

      # Only process alert creation
      allowed_actions:
        - Create

      # Only P1 alerts
      priority_filter:
        - P1

      # Only production environment
      tag_filter:
        - production

      # Only from monitoring tools
      source_filter:
        - datadog
        - prometheus
        - cloudwatch

  agent:
    runtime: python
    model: google:gemini-2.5-flash

    tools:
      - name: kubectl
      - name: aws-cli
      - name: diagnostics

    instructions: |
      You are an incident response agent for critical production alerts.

      When a P1 alert is created:
      1. Acknowledge the alert immediately
      2. Parse alert details to identify affected service
      3. Run automated diagnostics
      4. Add findings as notes to the alert
      5. If auto-remediation possible, ask for approval
      6. Otherwise, escalate with context

  nodes:
    - id: acknowledge
      type: execute
      execute:
        command: |
          import os
          import json

          # Extract alert details from trigger
          alert_id = trigger.metadata["alert_id"]
          message = trigger.metadata["message"]
          entity = trigger.metadata.get("entity", "unknown")

          # Acknowledge alert
          opsgenie.acknowledge_alert(
              alert_id=alert_id,
              note=f"🤖 AOF Agent acknowledged. Running diagnostics on {entity}..."
          )

          context.set("alert_id", alert_id)
          context.set("entity", entity)

    - id: diagnose
      type: execute
      depends_on: [acknowledge]
      execute:
        command: |
          import subprocess

          entity = context.get("entity")
          alert_id = context.get("alert_id")

          # Run diagnostics based on entity type
          if "db-" in entity:
              # Database diagnostics
              result = subprocess.run([
                  "kubectl", "exec", entity, "--",
                  "pg_top", "-b", "-n", "1"
              ], capture_output=True, text=True)

              diagnostics = result.stdout

          elif "api-" in entity:
              # API service diagnostics
              result = subprocess.run([
                  "kubectl", "logs", entity, "--tail=100"
              ], capture_output=True, text=True)

              diagnostics = result.stdout

          else:
              diagnostics = "Unknown entity type"

          # Add findings to alert
          opsgenie.add_note(
              alert_id=alert_id,
              note=f"📊 Diagnostic Results:\n```\n{diagnostics}\n```"
          )

          context.set("diagnostics", diagnostics)

    - id: analyze
      type: llm
      depends_on: [diagnose]
      llm:
        prompt: |
          Alert: {{ trigger.metadata.message }}
          Description: {{ trigger.metadata.description }}

          Diagnostics:
          {{ context.diagnostics }}

          Analyze this incident and determine:
          1. Root cause (if identifiable)
          2. Recommended remediation steps
          3. Can this be auto-remediated safely? (yes/no)

          Respond in JSON:
          {
            "root_cause": "...",
            "remediation_steps": ["step1", "step2"],
            "auto_remediate": true/false,
            "confidence": 0.0-1.0
          }

    - id: report_findings
      type: execute
      depends_on: [analyze]
      execute:
        command: |
          import json

          alert_id = context.get("alert_id")
          analysis = json.loads(context.get("llm_response"))

          # Format findings
          note = f"""
          🔍 **Analysis Complete**

          **Root Cause:** {analysis['root_cause']}

          **Recommended Steps:**
          {chr(10).join(f"- {step}" for step in analysis['remediation_steps'])}

          **Auto-Remediation:** {'✅ Possible' if analysis['auto_remediate'] else '❌ Manual intervention required'}
          **Confidence:** {analysis['confidence']*100:.0f}%
          """

          opsgenie.add_note(alert_id=alert_id, note=note)

          if analysis['auto_remediate'] and analysis['confidence'] > 0.8:
              # High confidence auto-remediation possible
              context.set("proceed_to_remediation", True)
          else:
              # Escalate
              opsgenie.escalate_alert(
                  alert_id=alert_id,
                  note="⚠️ Auto-remediation not recommended. Human review needed."
              )

  security:
    approval_required: true
    approval_timeout: 300  # 5 minutes
```

### Example 2: Interactive Incident Response via Notes

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: opsgenie-interactive-responder

spec:
  description: Respond to commands in Opsgenie alert notes

  trigger:
    type: Opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key_env: OPSGENIE_API_KEY

      # Only process note additions
      allowed_actions:
        - AddNote

      # All priorities
      priority_filter: null

  agent:
    runtime: python
    model: google:gemini-2.5-flash

    instructions: |
      Parse commands from Opsgenie alert notes.

      Supported commands:
      - /diagnose <service> - Run diagnostics
      - /logs <service> [lines] - Fetch logs
      - /scale <service> <replicas> - Scale deployment
      - /restart <service> - Restart service
      - /status <service> - Get service status

  nodes:
    - id: parse_command
      type: execute
      execute:
        command: |
          import re

          # Extract note from trigger
          note = trigger.metadata.get("note", "")
          alert_id = trigger.metadata["alert_id"]

          # Parse command (format: /command arg1 arg2)
          match = re.match(r'/(\w+)\s*(.*)', note.strip())

          if not match:
              # Not a command, ignore
              context.set("skip", True)
              return

          command = match.group(1)
          args = match.group(2).split()

          context.set("command", command)
          context.set("args", args)
          context.set("alert_id", alert_id)

    - id: execute_command
      type: execute
      depends_on: [parse_command]
      execute:
        command: |
          import subprocess

          if context.get("skip"):
              return

          command = context.get("command")
          args = context.get("args")
          alert_id = context.get("alert_id")

          result = ""

          if command == "diagnose":
              service = args[0] if args else "unknown"
              result = subprocess.run(
                  ["kubectl", "describe", "pod", service],
                  capture_output=True, text=True
              ).stdout

          elif command == "logs":
              service = args[0] if args else "unknown"
              lines = args[1] if len(args) > 1 else "100"
              result = subprocess.run(
                  ["kubectl", "logs", service, f"--tail={lines}"],
                  capture_output=True, text=True
              ).stdout

          elif command == "scale":
              service = args[0] if args else "unknown"
              replicas = args[1] if len(args) > 1 else "3"
              result = subprocess.run(
                  ["kubectl", "scale", "deployment", service, f"--replicas={replicas}"],
                  capture_output=True, text=True
              ).stdout

          elif command == "restart":
              service = args[0] if args else "unknown"
              result = subprocess.run(
                  ["kubectl", "rollout", "restart", "deployment", service],
                  capture_output=True, text=True
              ).stdout

          elif command == "status":
              service = args[0] if args else "unknown"
              result = subprocess.run(
                  ["kubectl", "get", "pod", "-l", f"app={service}"],
                  capture_output=True, text=True
              ).stdout

          else:
              result = f"Unknown command: {command}"

          # Post result as note
          opsgenie.add_note(
              alert_id=alert_id,
              note=f"✅ **Command:** /{command} {' '.join(args)}\n\n```\n{result}\n```"
          )
```

### Example 3: Custom Action Handler

```yaml
apiVersion: aof.dev/v1
kind: AgentFlow
metadata:
  name: opsgenie-custom-actions

spec:
  description: Handle custom action buttons in Opsgenie mobile app

  trigger:
    type: Opsgenie
    config:
      api_url: https://api.opsgenie.com
      api_key_env: OPSGENIE_API_KEY

      # Only process custom actions
      allowed_actions:
        - CustomAction

  agent:
    runtime: python
    model: google:gemini-2.5-flash

    instructions: |
      Execute predefined remediation workflows triggered by
      Opsgenie custom action buttons.

  nodes:
    - id: route_action
      type: execute
      execute:
        command: |
          import json

          alert_id = trigger.metadata["alert_id"]
          custom_action = trigger.metadata.get("customAction", {})
          action_name = custom_action.get("name", "unknown")

          context.set("alert_id", alert_id)
          context.set("action_name", action_name)

          # Route to appropriate handler
          if action_name == "Restart Service":
              context.set("workflow", "restart")
          elif action_name == "Scale Up":
              context.set("workflow", "scale_up")
          elif action_name == "Run Diagnostics":
              context.set("workflow", "diagnostics")
          else:
              context.set("workflow", "unknown")

    - id: execute_workflow
      type: execute
      depends_on: [route_action]
      execute:
        command: |
          alert_id = context.get("alert_id")
          workflow = context.get("workflow")
          entity = trigger.metadata.get("entity", "unknown")

          if workflow == "restart":
              # Restart service
              subprocess.run(["kubectl", "rollout", "restart", "deployment", entity])

              opsgenie.add_note(
                  alert_id=alert_id,
                  note=f"🔄 Service {entity} restarted via custom action"
              )

              # Close alert after successful restart
              opsgenie.close_alert(
                  alert_id=alert_id,
                  note="Auto-remediated via service restart"
              )

          elif workflow == "scale_up":
              # Scale up by 50%
              subprocess.run([
                  "kubectl", "scale", "deployment", entity,
                  "--replicas=5"  # Could calculate dynamically
              ])

              opsgenie.add_note(
                  alert_id=alert_id,
                  note=f"📈 Scaled {entity} to 5 replicas via custom action"
              )
```

---

## Example Agent Integration

### Python Agent with Opsgenie SDK

```python
# Example: Custom agent using Opsgenie platform

from aof import Agent, Context
import os

class OpsgenieIncidentResponder(Agent):
    """
    Intelligent incident responder for Opsgenie alerts.
    """

    def __init__(self):
        super().__init__(
            name="opsgenie-responder",
            model="google:gemini-2.5-flash"
        )

        # Opsgenie client (provided by platform)
        self.opsgenie = self.trigger_platform

    async def on_trigger(self, trigger: TriggerMessage, context: Context):
        """
        Handle Opsgenie alert trigger.
        """

        # Extract alert metadata
        alert_id = trigger.metadata["alert_id"]
        action = trigger.metadata["action"]
        priority = trigger.metadata.get("priority", "P3")
        message = trigger.metadata["message"]
        entity = trigger.metadata.get("entity")

        # Route based on action
        if action == "Create":
            await self.handle_new_alert(alert_id, priority, message, entity)
        elif action == "AddNote":
            await self.handle_note_command(alert_id, trigger.metadata.get("note"))
        elif action == "Escalate":
            await self.handle_escalation(alert_id)

    async def handle_new_alert(self, alert_id, priority, message, entity):
        """
        Handle new alert creation.
        """

        # Acknowledge immediately for P1/P2
        if priority in ["P1", "P2"]:
            await self.opsgenie.acknowledge_alert(
                alert_id=alert_id,
                note="🤖 AOF Agent acknowledged. Starting automated response."
            )

        # Run diagnostics
        diagnostics = await self.run_diagnostics(entity)

        # Add findings to alert
        await self.opsgenie.add_note(
            alert_id=alert_id,
            note=f"📊 Diagnostic Results:\n```\n{diagnostics}\n```"
        )

        # Analyze with LLM
        analysis = await self.llm_analyze(message, diagnostics)

        # Decide next action
        if analysis["auto_remediate"]:
            await self.execute_remediation(alert_id, entity, analysis["steps"])
        else:
            await self.escalate_with_context(alert_id, analysis)

    async def execute_remediation(self, alert_id, entity, steps):
        """
        Execute auto-remediation steps.
        """

        results = []

        for step in steps:
            result = await self.execute_step(step, entity)
            results.append(result)

            # Post progress
            await self.opsgenie.add_note(
                alert_id=alert_id,
                note=f"✅ {step}: {result}"
            )

        # Close alert if successful
        if all(r["success"] for r in results):
            await self.opsgenie.close_alert(
                alert_id=alert_id,
                note="🎉 Auto-remediated successfully. All steps completed."
            )
```

### Rust Integration Example

```rust
// Example: Rust agent with Opsgenie platform

use aof_core::{Agent, AgentBuilder, Context};
use aof_triggers::platforms::{OpsgeniePlatform, OpsgenieConfig};
use aof_triggers::TriggerMessage;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Build Opsgenie platform
    let opsgenie = OpsgeniePlatform::new(OpsgenieConfig {
        api_url: "https://api.opsgenie.com".to_string(),
        api_key: std::env::var("OPSGENIE_API_KEY")?,
        webhook_token: Some(std::env::var("OPSGENIE_WEBHOOK_TOKEN")?),
        bot_name: "aof-responder".to_string(),
        allowed_actions: vec!["Create".to_string(), "AddNote".to_string()],
        priority_filter: Some(vec!["P1".to_string(), "P2".to_string()]),
        ..Default::default()
    })?;

    // Build agent
    let agent = AgentBuilder::new()
        .name("opsgenie-responder")
        .model("google:gemini-2.5-flash")
        .trigger_platform(opsgenie)
        .on_trigger(handle_trigger)
        .build()?;

    // Start listening
    agent.listen().await?;

    Ok(())
}

async fn handle_trigger(
    trigger: TriggerMessage,
    context: &mut Context,
    opsgenie: &OpsgeniePlatform,
) -> Result<()> {
    // Extract alert details
    let alert_id = trigger.metadata.get("alert_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let action = trigger.metadata.get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    match action {
        "Create" => handle_new_alert(alert_id, &trigger, context, opsgenie).await?,
        "AddNote" => handle_note(alert_id, &trigger, context, opsgenie).await?,
        _ => {}
    }

    Ok(())
}

async fn handle_new_alert(
    alert_id: &str,
    trigger: &TriggerMessage,
    context: &mut Context,
    opsgenie: &OpsgeniePlatform,
) -> Result<()> {
    // Acknowledge alert
    opsgenie.acknowledge_alert(
        alert_id,
        Some("🤖 AOF Agent acknowledged. Running diagnostics..."),
    ).await?;

    // Run diagnostics (example)
    let diagnostics = run_diagnostics(trigger).await?;

    // Add findings
    opsgenie.add_note(
        alert_id,
        &format!("📊 Diagnostics:\n```\n{}\n```", diagnostics),
    ).await?;

    Ok(())
}
```

---

## API Reference

### Opsgenie REST API Endpoints

**Base URLs:**
- US: `https://api.opsgenie.com`
- EU: `https://api.eu.opsgenie.com`

**Authentication:**
```
Authorization: GenieKey {api_key}
```

**Key Endpoints:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v2/alerts` | Create alert |
| GET | `/v2/alerts/{id}` | Get alert details |
| POST | `/v2/alerts/{id}/acknowledge` | Acknowledge alert |
| POST | `/v2/alerts/{id}/close` | Close alert |
| POST | `/v2/alerts/{id}/notes` | Add note |
| POST | `/v2/alerts/{id}/responders` | Add responder |
| POST | `/v2/alerts/{id}/escalate` | Escalate alert |
| GET | `/v2/alerts/{id}/activities` | Get alert timeline |

**Rate Limits:**
- 600 requests/minute per API key
- Burst: up to 1000 requests in 10 seconds
- 429 response includes `X-RateLimit-Reset` header

**Error Handling:**
```json
{
  "message": "Rate limit exceeded",
  "took": 0.001,
  "requestId": "uuid"
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opsgenie_config_validation() {
        let config = OpsgenieConfig {
            api_url: "".to_string(),
            api_key: "".to_string(),
            ..Default::default()
        };

        let result = OpsgeniePlatform::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_filter() {
        let config = OpsgenieConfig {
            api_url: "https://api.opsgenie.com".to_string(),
            api_key: "test-key".to_string(),
            priority_filter: Some(vec!["P1".to_string(), "P2".to_string()]),
            ..Default::default()
        };

        let platform = OpsgeniePlatform::new(config).unwrap();

        assert!(platform.is_priority_allowed("P1"));
        assert!(platform.is_priority_allowed("P2"));
        assert!(!platform.is_priority_allowed("P3"));
    }

    #[tokio::test]
    async fn test_parse_create_alert_webhook() {
        let webhook_json = r#"{
            "action": "Create",
            "alert": {
                "alertId": "1c222736-5ec3-46e7-aeeb-1d608a7c1c12",
                "message": "CPU High",
                "priority": "P1",
                "tags": ["production"],
                "source": "datadog"
            },
            "integrationId": "test-integration"
        }"#;

        let config = OpsgenieConfig {
            api_url: "https://api.opsgenie.com".to_string(),
            api_key: "test-key".to_string(),
            integration_id: Some("test-integration".to_string()),
            ..Default::default()
        };

        let platform = OpsgeniePlatform::new(config).unwrap();
        let headers = HashMap::new();

        let result = platform.parse_message(webhook_json.as_bytes(), &headers).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.platform, "opsgenie");
        assert!(message.text.contains("alert:created"));
    }
}
```

### Integration Tests

```bash
# Test webhook endpoint
curl -X POST http://localhost:8080/triggers/opsgenie \
  -H "Content-Type: application/json" \
  -d '{
    "action": "Create",
    "alert": {
      "alertId": "test-alert-id",
      "message": "Test Alert",
      "priority": "P2",
      "tags": ["test"]
    }
  }'
```

### End-to-End Test

```python
# test_opsgenie_e2e.py

import requests
import os

def test_create_alert_triggers_agent():
    """
    Test that creating an alert in Opsgenie triggers AOF agent.
    """

    # 1. Create alert in Opsgenie
    response = requests.post(
        "https://api.opsgenie.com/v2/alerts",
        headers={
            "Authorization": f"GenieKey {os.environ['OPSGENIE_API_KEY']}",
            "Content-Type": "application/json"
        },
        json={
            "message": "E2E Test Alert",
            "tags": ["e2e-test", "automated"],
            "priority": "P3"
        }
    )

    assert response.status_code == 202
    alert_id = response.json()["data"]["alertId"]

    # 2. Wait for webhook to trigger agent (async)
    time.sleep(5)

    # 3. Check that agent added a note
    response = requests.get(
        f"https://api.opsgenie.com/v2/alerts/{alert_id}/notes",
        headers={
            "Authorization": f"GenieKey {os.environ['OPSGENIE_API_KEY']}"
        }
    )

    notes = response.json()["data"]
    assert any("AOF Agent" in note["note"] for note in notes)

    # Cleanup
    requests.delete(
        f"https://api.opsgenie.com/v2/alerts/{alert_id}",
        headers={
            "Authorization": f"GenieKey {os.environ['OPSGENIE_API_KEY']}"
        }
    )
```

---

## Implementation Checklist

- [ ] Create `platforms/opsgenie.rs` module
- [ ] Implement `OpsgenieConfig` struct with env var support
- [ ] Implement `OpsgeniePlatform` struct
- [ ] Implement `TriggerPlatform` trait
- [ ] Add webhook payload types
- [ ] Implement API methods (note, acknowledge, close, create)
- [ ] Add integration ID verification
- [ ] Add priority/tag/source filters
- [ ] Update `platforms/mod.rs` to include Opsgenie
- [ ] Update `PlatformRegistry` to register Opsgenie
- [ ] Add unit tests
- [ ] Add integration tests
- [ ] Add example YAML resources to `examples/`
- [ ] Update user documentation
- [ ] Add to Changelog

---

## References

- [Opsgenie API Documentation](https://docs.opsgenie.com/docs/api-overview)
- [Opsgenie Webhook Documentation](https://docs.opsgenie.com/docs/webhook-integration)
- [Opsgenie Alert API](https://docs.opsgenie.com/docs/alert-api)
- [Atlassian Opsgenie Integrations](https://www.atlassian.com/software/opsgenie/integrations)
