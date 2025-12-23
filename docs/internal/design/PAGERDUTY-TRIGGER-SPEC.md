# PagerDuty Trigger Platform - Internal Design Specification

## 1. Overview

### 1.1 Purpose

PagerDuty is a leading incident management platform that enables teams to detect, respond to, and resolve incidents across their infrastructure and applications. This specification defines the integration of PagerDuty as a trigger platform in AOF, enabling agents to respond to incident events automatically.

### 1.2 Why Integrate PagerDuty?

PagerDuty integration provides:

- **Automated Incident Response**: Trigger AOF agents automatically when incidents occur
- **Intelligent Escalation**: Respond to incident state changes (triggered, acknowledged, escalated, resolved)
- **Context-Rich Events**: Access detailed incident metadata for informed agent decisions
- **Production-Ready**: Leverage PagerDuty's battle-tested incident management workflows
- **Enterprise Integration**: Seamless integration with existing PagerDuty deployments

### 1.3 Use Cases

**Automated Incident Diagnostics**
```yaml
# When incident triggers, automatically run diagnostic agent
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: auto-diagnose-incidents
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.triggered
  agent: diagnostic-agent
  filter:
    metadata.service: "production-api"
```

**Escalation Response**
```yaml
# Trigger senior SRE agent when incident escalates
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: escalation-response
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.escalated
      allowed_services:
        - production-database
        - payment-service
  agent: senior-sre-agent
```

**Automated Resolution Actions**
```yaml
# Trigger cleanup agent when incident resolves
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: post-incident-cleanup
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.resolved
  agent: cleanup-agent
  parameters:
    create_postmortem: true
    cleanup_temp_resources: true
```

---

## 2. Webhook Events

### 2.1 Supported Event Types (V3 Webhooks)

PagerDuty V3 webhooks support the following incident event types:

| Event Type | Description | Priority |
|------------|-------------|----------|
| `incident.triggered` | New incident created | **Phase 1** ✓ |
| `incident.acknowledged` | Incident acknowledged by responder | **Phase 1** ✓ |
| `incident.resolved` | Incident marked as resolved | **Phase 1** ✓ |
| `incident.escalated` | Incident escalated to next level | **Phase 1** ✓ |
| `incident.reassigned` | Incident reassigned to different user/team | **Phase 1** ✓ |
| `incident.unacknowledged` | Acknowledgement removed | Phase 2 |
| `incident.annotated` | Note/annotation added | Phase 2 |
| `incident.delegated` | Incident delegated to another user | Phase 2 |
| `incident.priority_updated` | Incident priority changed | Phase 2 |
| `incident.reopened` | Resolved incident reopened | Phase 2 |
| `incident.responder.added` | Responder added to incident | Phase 2 |
| `incident.responder.replied` | Responder replied to incident | Phase 2 |

**Note**: PagerDuty V2 webhooks reach end-of-support as of October 31, 2022. This implementation targets **V3 webhooks only**.

### 2.2 Webhook Payload Structure

#### 2.2.1 Incident Triggered Event

```json
{
  "event": {
    "id": "01DRF6BV56ABCDEFGHIJK12345",
    "event_type": "incident.triggered",
    "resource_type": "incident",
    "occurred_at": "2025-12-23T10:30:00Z",
    "agent": {
      "html_url": "https://acme.pagerduty.com/users/P123ABC",
      "id": "P123ABC",
      "self": "https://api.pagerduty.com/users/P123ABC",
      "summary": "John Doe",
      "type": "user_reference"
    },
    "client": null,
    "data": {
      "id": "Q2KURS8RXYZ123",
      "type": "incident",
      "self": "https://api.pagerduty.com/incidents/Q2KURS8RXYZ123",
      "html_url": "https://acme.pagerduty.com/incidents/Q2KURS8RXYZ123",
      "number": 123,
      "status": "triggered",
      "incident_key": "srv01/high_cpu",
      "created_at": "2025-12-23T10:30:00Z",
      "title": "High CPU usage on srv01",
      "service": {
        "id": "PXYZ123",
        "type": "service_reference",
        "summary": "Production API",
        "self": "https://api.pagerduty.com/services/PXYZ123",
        "html_url": "https://acme.pagerduty.com/services/PXYZ123"
      },
      "assignees": [
        {
          "id": "P123ABC",
          "type": "user_reference",
          "summary": "John Doe",
          "self": "https://api.pagerduty.com/users/P123ABC",
          "html_url": "https://acme.pagerduty.com/users/P123ABC"
        }
      ],
      "escalation_policy": {
        "id": "P789XYZ",
        "type": "escalation_policy_reference",
        "summary": "Production Escalation",
        "self": "https://api.pagerduty.com/escalation_policies/P789XYZ",
        "html_url": "https://acme.pagerduty.com/escalation_policies/P789XYZ"
      },
      "teams": [
        {
          "id": "P456DEF",
          "type": "team_reference",
          "summary": "Infrastructure Team",
          "self": "https://api.pagerduty.com/teams/P456DEF",
          "html_url": "https://acme.pagerduty.com/teams/P456DEF"
        }
      ],
      "priority": {
        "id": "P1HIGH",
        "type": "priority_reference",
        "summary": "P1",
        "self": "https://api.pagerduty.com/priorities/P1HIGH"
      },
      "urgency": "high",
      "conference_bridge": null,
      "resolve_reason": null
    }
  }
}
```

#### 2.2.2 Incident Acknowledged Event

```json
{
  "event": {
    "id": "01DRF6BV78DEFGHIJK56789",
    "event_type": "incident.acknowledged",
    "resource_type": "incident",
    "occurred_at": "2025-12-23T10:31:30Z",
    "agent": {
      "html_url": "https://acme.pagerduty.com/users/P123ABC",
      "id": "P123ABC",
      "self": "https://api.pagerduty.com/users/P123ABC",
      "summary": "John Doe",
      "type": "user_reference"
    },
    "data": {
      "id": "Q2KURS8RXYZ123",
      "status": "acknowledged",
      "acknowledgements": [
        {
          "at": "2025-12-23T10:31:30Z",
          "acknowledger": {
            "id": "P123ABC",
            "type": "user_reference",
            "summary": "John Doe"
          }
        }
      ]
    }
  }
}
```

#### 2.2.3 Incident Resolved Event

```json
{
  "event": {
    "id": "01DRF6BV90GHIJK12345ABC",
    "event_type": "incident.resolved",
    "resource_type": "incident",
    "occurred_at": "2025-12-23T11:00:00Z",
    "agent": {
      "html_url": "https://acme.pagerduty.com/users/P123ABC",
      "id": "P123ABC",
      "self": "https://api.pagerduty.com/users/P123ABC",
      "summary": "John Doe",
      "type": "user_reference"
    },
    "data": {
      "id": "Q2KURS8RXYZ123",
      "status": "resolved",
      "resolve_reason": {
        "type": "resolve_reason_reference",
        "incident": {
          "html_url": "https://acme.pagerduty.com/incidents/Q2KURS8RXYZ123",
          "id": "Q2KURS8RXYZ123",
          "self": "https://api.pagerduty.com/incidents/Q2KURS8RXYZ123",
          "summary": "High CPU usage on srv01",
          "type": "incident_reference"
        }
      },
      "last_status_change_at": "2025-12-23T11:00:00Z"
    }
  }
}
```

---

## 3. Configuration

### 3.1 PagerDutyConfig Struct

```rust
/// PagerDuty platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// Webhook secret for signature verification (required)
    /// Obtained when creating a Generic Webhook (v3) in PagerDuty
    pub webhook_secret: String,

    /// Bot name for display purposes
    #[serde(default = "default_bot_name")]
    pub bot_name: String,

    /// Event types to process (default: all incident events)
    #[serde(default)]
    pub event_types: Option<Vec<String>>,

    /// Allowed service IDs (optional - filter by service)
    #[serde(default)]
    pub allowed_services: Option<Vec<String>>,

    /// Allowed team IDs (optional - filter by team)
    #[serde(default)]
    pub allowed_teams: Option<Vec<String>>,

    /// Minimum priority level to process (optional)
    /// Values: "P1", "P2", "P3", "P4", "P5" (P1 = highest)
    #[serde(default)]
    pub min_priority: Option<String>,

    /// Minimum urgency level to process (optional)
    /// Values: "high", "low"
    #[serde(default)]
    pub min_urgency: Option<String>,

    /// PagerDuty API token for response actions (optional)
    /// Required for updating incidents, adding notes, etc.
    #[serde(default)]
    pub api_token: Option<String>,
}

fn default_bot_name() -> String {
    "aofbot".to_string()
}
```

### 3.2 Environment Variables

```bash
# Required: Webhook signature verification
export PAGERDUTY_WEBHOOK_SECRET="abc123def456..."

# Optional: API token for response actions
export PAGERDUTY_API_TOKEN="u+abc123..."

# Optional: Filter configuration
export PAGERDUTY_ALLOWED_SERVICES="PXYZ123,PABC456"
export PAGERDUTY_ALLOWED_TEAMS="P456DEF"
export PAGERDUTY_MIN_PRIORITY="P2"
export PAGERDUTY_MIN_URGENCY="high"
```

### 3.3 YAML Resource Specification

#### 3.3.1 Basic Trigger

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: pagerduty-incident-handler
  namespace: production
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      event_types:
        - incident.triggered
        - incident.acknowledged
        - incident.resolved
  agent: incident-response-agent
  enabled: true
```

#### 3.3.2 Filtered Trigger (Service-Specific)

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: critical-service-incidents
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      api_token: ${PAGERDUTY_API_TOKEN}
      event_types:
        - incident.triggered
      allowed_services:
        - PXYZ123  # Production API
        - PXYZ456  # Payment Service
      min_priority: "P1"
      min_urgency: "high"
  agent: critical-incident-agent
```

#### 3.3.3 Team-Based Routing

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: infrastructure-team-incidents
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      allowed_teams:
        - P456DEF  # Infrastructure Team
  agent: infra-diagnostic-agent
```

---

## 4. Implementation Details

### 4.1 Webhook Signature Verification

PagerDuty V3 webhooks use HMAC-SHA256 signature verification with the `x-pagerduty-signature` header.

#### 4.1.1 Signature Format

```
x-pagerduty-signature: v1=<hex_encoded_signature>
```

Example:
```
x-pagerduty-signature: v1=2600a856b1c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9
```

#### 4.1.2 Verification Algorithm

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify PagerDuty webhook signature
fn verify_pagerduty_signature(
    payload: &[u8],
    signature: &str,
    webhook_secret: &str,
) -> bool {
    // Signature format: v1=<hex_signature>
    if !signature.starts_with("v1=") {
        debug!("Invalid signature format - must start with v1=");
        return false;
    }

    let provided_signature = &signature[3..];

    // Create HMAC-SHA256 with webhook secret
    let mut mac = match HmacSha256::new_from_slice(webhook_secret.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            error!("HMAC setup failed: {}", e);
            return false;
        }
    };

    // Hash the raw payload
    mac.update(payload);

    let result = mac.finalize();
    let computed_signature = hex::encode(result.into_bytes());

    // Compare signatures
    if computed_signature == provided_signature {
        debug!("PagerDuty signature verified successfully");
        true
    } else {
        debug!(
            "Signature mismatch - computed: {}, provided: {}",
            &computed_signature[..8],
            &provided_signature[..8.min(provided_signature.len())]
        );
        false
    }
}
```

#### 4.1.3 Additional Headers

```rust
// Additional PagerDuty webhook headers
const HEADER_SIGNATURE: &str = "x-pagerduty-signature";
const HEADER_SUBSCRIPTION: &str = "x-webhook-subscription";

// Example header values:
// x-pagerduty-signature: v1=abc123...
// x-webhook-subscription: P1WEBHOOK2SUBSCRIPTION3ID
```

### 4.2 Payload Parsing

#### 4.2.1 Event Envelope Structure

```rust
/// PagerDuty webhook event envelope
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyWebhook {
    event: PagerDutyEvent,
}

#[derive(Debug, Clone, Deserialize)]
struct PagerDutyEvent {
    /// Unique event ID
    id: String,

    /// Event type (e.g., "incident.triggered")
    event_type: String,

    /// Resource type (always "incident" for incident events)
    resource_type: String,

    /// Timestamp when event occurred
    occurred_at: String,

    /// User/agent who triggered the event
    #[serde(default)]
    agent: Option<PagerDutyAgent>,

    /// Event data (incident details)
    data: PagerDutyIncident,
}
```

#### 4.2.2 Incident Data Structure

```rust
#[derive(Debug, Clone, Deserialize)]
struct PagerDutyIncident {
    /// Incident ID
    id: String,

    /// Resource type
    #[serde(rename = "type")]
    resource_type: String,

    /// API URL
    #[serde(rename = "self")]
    self_url: String,

    /// Web UI URL
    html_url: String,

    /// Incident number (human-readable)
    number: u64,

    /// Current status
    status: String,

    /// Incident key (deduplication key)
    incident_key: String,

    /// Created timestamp
    created_at: String,

    /// Incident title/summary
    title: String,

    /// Service reference
    service: PagerDutyServiceRef,

    /// Assigned users
    #[serde(default)]
    assignees: Vec<PagerDutyUserRef>,

    /// Escalation policy
    #[serde(default)]
    escalation_policy: Option<PagerDutyEscalationRef>,

    /// Teams
    #[serde(default)]
    teams: Vec<PagerDutyTeamRef>,

    /// Priority
    #[serde(default)]
    priority: Option<PagerDutyPriorityRef>,

    /// Urgency level
    urgency: String,
}
```

#### 4.2.3 Conversion to TriggerMessage

```rust
async fn parse_pagerduty_event(
    &self,
    webhook: PagerDutyWebhook,
) -> Result<TriggerMessage, PlatformError> {
    let event = webhook.event;
    let incident = event.data;

    // Create TriggerUser from agent
    let trigger_user = if let Some(agent) = event.agent {
        TriggerUser {
            id: agent.id.clone(),
            username: Some(agent.id),
            display_name: Some(agent.summary),
            is_bot: false,
        }
    } else {
        // System event (no agent)
        TriggerUser {
            id: "system".to_string(),
            username: Some("system".to_string()),
            display_name: Some("PagerDuty System".to_string()),
            is_bot: true,
        }
    };

    // Build metadata
    let mut metadata = HashMap::new();
    metadata.insert("event_id".to_string(), json!(event.id));
    metadata.insert("event_type".to_string(), json!(event.event_type));
    metadata.insert("occurred_at".to_string(), json!(event.occurred_at));
    metadata.insert("incident_id".to_string(), json!(incident.id));
    metadata.insert("incident_number".to_string(), json!(incident.number));
    metadata.insert("incident_key".to_string(), json!(incident.incident_key));
    metadata.insert("status".to_string(), json!(incident.status));
    metadata.insert("urgency".to_string(), json!(incident.urgency));
    metadata.insert("html_url".to_string(), json!(incident.html_url));
    metadata.insert("service_id".to_string(), json!(incident.service.id));
    metadata.insert("service_name".to_string(), json!(incident.service.summary));

    if let Some(priority) = incident.priority {
        metadata.insert("priority".to_string(), json!(priority.summary));
    }

    if !incident.teams.is_empty() {
        let team_ids: Vec<_> = incident.teams.iter().map(|t| &t.id).collect();
        metadata.insert("team_ids".to_string(), json!(team_ids));
    }

    // Create TriggerMessage
    Ok(TriggerMessage {
        id: event.id,
        platform: "pagerduty".to_string(),
        channel_id: incident.service.id.clone(),
        user: trigger_user,
        text: incident.title,
        timestamp: chrono::Utc::now(),
        metadata,
        thread_id: Some(incident.id.clone()),
        reply_to: None,
    })
}
```

### 4.3 Event Type Routing

```rust
/// Check if event should be processed based on configuration
fn should_process_event(&self, event: &PagerDutyEvent) -> bool {
    // Check event type filter
    if let Some(ref allowed_types) = self.config.event_types {
        if !allowed_types.contains(&event.event_type) {
            debug!("Event type {} not in allowed list", event.event_type);
            return false;
        }
    }

    let incident = &event.data;

    // Check service filter
    if let Some(ref allowed_services) = self.config.allowed_services {
        if !allowed_services.contains(&incident.service.id) {
            debug!("Service {} not in allowed list", incident.service.id);
            return false;
        }
    }

    // Check team filter
    if let Some(ref allowed_teams) = self.config.allowed_teams {
        let has_allowed_team = incident.teams.iter()
            .any(|team| allowed_teams.contains(&team.id));
        if !has_allowed_team {
            debug!("No allowed teams found for incident");
            return false;
        }
    }

    // Check priority filter
    if let Some(ref min_priority) = self.config.min_priority {
        if let Some(ref priority) = incident.priority {
            // P1 > P2 > P3 > P4 > P5
            if !is_priority_sufficient(&priority.summary, min_priority) {
                debug!("Priority {} below minimum {}", priority.summary, min_priority);
                return false;
            }
        }
    }

    // Check urgency filter
    if let Some(ref min_urgency) = self.config.min_urgency {
        if incident.urgency != *min_urgency && *min_urgency == "high" {
            debug!("Urgency {} below minimum high", incident.urgency);
            return false;
        }
    }

    true
}

fn is_priority_sufficient(current: &str, minimum: &str) -> bool {
    let priority_map: HashMap<&str, u8> = [
        ("P1", 1),
        ("P2", 2),
        ("P3", 3),
        ("P4", 4),
        ("P5", 5),
    ].iter().cloned().collect();

    let current_level = priority_map.get(current).unwrap_or(&99);
    let min_level = priority_map.get(minimum).unwrap_or(&0);

    current_level <= min_level
}
```

---

## 5. Response Handling

### 5.1 PagerDuty Events API

The PagerDuty platform can perform response actions using the Events API v2.

#### 5.1.1 API Authentication

```rust
const EVENTS_API_URL: &str = "https://events.pagerduty.com/v2/enqueue";
const REST_API_URL: &str = "https://api.pagerduty.com";

/// Create HTTP client with API token
fn create_api_client(&self) -> Result<reqwest::Client, PlatformError> {
    if let Some(ref api_token) = self.config.api_token {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Token token={}", api_token).parse().unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/json".parse().unwrap(),
        );

        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PlatformError::ApiError(format!("Failed to create client: {}", e)))
    } else {
        Err(PlatformError::ApiError("API token not configured".to_string()))
    }
}
```

### 5.2 Update Incident Status

```rust
/// Update incident status (acknowledge, resolve)
pub async fn update_incident_status(
    &self,
    incident_id: &str,
    status: &str,
    from_email: &str,
) -> Result<(), PlatformError> {
    let client = self.create_api_client()?;

    let payload = json!({
        "incident": {
            "type": "incident_reference",
            "status": status,
        }
    });

    let url = format!("{}/incidents/{}", REST_API_URL, incident_id);

    let response = client
        .put(&url)
        .header("From", from_email)
        .json(&payload)
        .send()
        .await
        .map_err(|e| PlatformError::ApiError(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(PlatformError::ApiError(format!("API error: {}", error)));
    }

    Ok(())
}
```

### 5.3 Add Notes to Incident

```rust
/// Add a note to an incident
pub async fn add_incident_note(
    &self,
    incident_id: &str,
    note_content: &str,
    from_email: &str,
) -> Result<(), PlatformError> {
    let client = self.create_api_client()?;

    let payload = json!({
        "note": {
            "content": note_content,
        }
    });

    let url = format!("{}/incidents/{}/notes", REST_API_URL, incident_id);

    let response = client
        .post(&url)
        .header("From", from_email)
        .json(&payload)
        .send()
        .await
        .map_err(|e| PlatformError::ApiError(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(PlatformError::ApiError(format!("API error: {}", error)));
    }

    Ok(())
}
```

### 5.4 Create Incident

```rust
/// Create a new incident
pub async fn create_incident(
    &self,
    title: &str,
    service_id: &str,
    from_email: &str,
    urgency: &str,
) -> Result<String, PlatformError> {
    let client = self.create_api_client()?;

    let payload = json!({
        "incident": {
            "type": "incident",
            "title": title,
            "service": {
                "id": service_id,
                "type": "service_reference",
            },
            "urgency": urgency,
        }
    });

    let url = format!("{}/incidents", REST_API_URL);

    let response = client
        .post(&url)
        .header("From", from_email)
        .json(&payload)
        .send()
        .await
        .map_err(|e| PlatformError::ApiError(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(PlatformError::ApiError(format!("API error: {}", error)));
    }

    let result: serde_json::Value = response.json().await
        .map_err(|e| PlatformError::ParseError(format!("Failed to parse response: {}", e)))?;

    let incident_id = result["incident"]["id"]
        .as_str()
        .ok_or_else(|| PlatformError::ParseError("Missing incident ID in response".to_string()))?;

    Ok(incident_id.to_string())
}
```

### 5.5 Send Response

```rust
#[async_trait]
impl TriggerPlatform for PagerDutyPlatform {
    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // channel = incident_id
        let incident_id = channel;

        // Add note to incident with agent response
        if let Some(ref api_token) = self.config.api_token {
            let note = format!(
                "**AOF Agent Response**\n\n{}",
                response.text
            );

            // Use a default email or extract from metadata
            let from_email = response.metadata
                .get("from_email")
                .and_then(|v| v.as_str())
                .unwrap_or("aof@example.com");

            self.add_incident_note(incident_id, &note, from_email).await?;
        } else {
            warn!("API token not configured - cannot add notes to incidents");
        }

        Ok(())
    }
}
```

---

## 6. Example YAML Trigger Resource

### 6.1 Full Example with All Features

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: pagerduty-production-incidents
  namespace: production
  labels:
    environment: production
    team: sre
    platform: pagerduty
spec:
  # Platform configuration
  platform:
    type: pagerduty
    config:
      # Webhook verification secret (from PagerDuty webhook settings)
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}

      # Optional: API token for response actions
      api_token: ${PAGERDUTY_API_TOKEN}

      # Bot name for display
      bot_name: "aof-incident-bot"

      # Filter by event types
      event_types:
        - incident.triggered
        - incident.acknowledged
        - incident.escalated
        - incident.resolved
        - incident.reassigned

      # Filter by specific services
      allowed_services:
        - PXYZ123  # Production API
        - PXYZ456  # Payment Service
        - PXYZ789  # Database Service

      # Filter by teams
      allowed_teams:
        - P456DEF  # Infrastructure Team
        - P789GHI  # Platform Team

      # Only process P1 and P2 incidents
      min_priority: "P2"

      # Only process high urgency incidents
      min_urgency: "high"

  # Agent to trigger
  agent: incident-response-agent

  # Trigger enabled
  enabled: true

  # Optional: Agent parameters
  parameters:
    auto_acknowledge: false
    create_postmortem: true
    notify_channels:
      - "#incidents"
      - "#ops-alerts"
```

### 6.2 Auto-Resolution Trigger

```yaml
apiVersion: aof.sh/v1
kind: Trigger
metadata:
  name: auto-resolve-known-issues
spec:
  platform:
    type: pagerduty
    config:
      webhook_secret: ${PAGERDUTY_WEBHOOK_SECRET}
      api_token: ${PAGERDUTY_API_TOKEN}
      event_types:
        - incident.triggered
      allowed_services:
        - PXYZ123  # Production API
  agent: auto-resolution-agent
  parameters:
    # Agent will check known issues database
    check_known_issues: true
    # Auto-resolve if known issue with automatic fix
    auto_resolve_if_known: true
```

---

## 7. Example Agent Integration

### 7.1 Incident Response Agent

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: incident-response-agent
  namespace: production
spec:
  llm:
    provider: google
    model: gemini-2.5-flash
    temperature: 0.2

  system_prompt: |
    You are an SRE incident response agent integrated with PagerDuty.

    When an incident is triggered, you should:
    1. Analyze the incident metadata
    2. Check recent logs and metrics
    3. Determine if this is a known issue
    4. Suggest remediation steps
    5. Update the incident with findings

    Available context:
    - Incident ID: {{incident_id}}
    - Service: {{service_name}}
    - Status: {{status}}
    - Urgency: {{urgency}}
    - Priority: {{priority}}
    - Title: {{title}}

  tools:
    - name: kubectl
      enabled: true
    - name: prometheus
      enabled: true
    - name: elasticsearch
      enabled: true

  memory:
    enabled: true
    persistence: redis

### 7.2 Diagnostic Agent (Triggered on Incident)

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: diagnostic-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are a diagnostic agent that runs automated checks when PagerDuty incidents trigger.

    Your workflow:
    1. Extract incident metadata (service, urgency, priority)
    2. Run relevant diagnostic commands
    3. Analyze logs and metrics
    4. Generate diagnostic report
    5. Add findings as note to PagerDuty incident

    Use the incident context provided in the message metadata.

  tools:
    - kubectl
    - prometheus
    - elasticsearch

  mcp:
    - server: kubernetes
      enabled: true
```

### 7.3 Cleanup Agent (Triggered on Incident Resolved)

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: cleanup-agent
spec:
  llm:
    provider: google
    model: gemini-2.5-flash

  system_prompt: |
    You are a post-incident cleanup agent.

    When an incident is resolved, you should:
    1. Create incident postmortem template
    2. Collect relevant logs and artifacts
    3. Clean up temporary resources
    4. Update documentation
    5. Create follow-up tasks

  tools:
    - kubectl
    - git
```

---

## 8. Platform Capabilities

### 8.1 TriggerPlatform Trait Implementation

```rust
impl TriggerPlatform for PagerDutyPlatform {
    fn platform_name(&self) -> &'static str {
        "pagerduty"
    }

    fn bot_name(&self) -> &str {
        &self.config.bot_name
    }

    fn supports_threading(&self) -> bool {
        true // Incident threads via notes
    }

    fn supports_interactive(&self) -> bool {
        false // No interactive UI in webhooks
    }

    fn supports_files(&self) -> bool {
        false // No file attachments in webhooks
    }
}
```

### 8.2 Platform Capabilities

```rust
pub fn get_platform_capabilities(platform: &str) -> PlatformCapabilities {
    match platform.to_lowercase().as_str() {
        "pagerduty" => PlatformCapabilities {
            threading: true,      // Incident notes form threads
            interactive: false,   // No interactive components
            files: false,         // No file attachments
            reactions: false,     // No reaction support
            rich_text: true,      // Markdown in notes
            approvals: false,     // No approval workflow
        },
        // ... other platforms
    }
}
```

---

## 9. Registry Integration

### 9.1 Register PagerDuty Platform

```rust
impl PlatformRegistry {
    pub fn register_defaults(&mut self) {
        // ... existing platforms ...

        // PagerDuty
        self.register("pagerduty", Box::new(|config| {
            let cfg: PagerDutyConfig = serde_json::from_value(config)
                .map_err(|e| PlatformError::ParseError(format!("Invalid PagerDuty config: {}", e)))?;
            Ok(Box::new(PagerDutyPlatform::new(cfg)?))
        }));
    }
}
```

### 9.2 TypedPlatformConfig Enum

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypedPlatformConfig {
    Slack(SlackConfig),
    Discord(DiscordConfig),
    Telegram(TelegramConfig),
    WhatsApp(WhatsAppConfig),
    Teams(TeamsConfig),
    GitHub(GitHubConfig),
    GitLab(GitLabConfig),
    Bitbucket(BitbucketConfig),
    Jira(JiraConfig),
    PagerDuty(PagerDutyConfig),  // Add PagerDuty
}
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> PagerDutyConfig {
        PagerDutyConfig {
            webhook_secret: "test-secret-123".to_string(),
            bot_name: "testbot".to_string(),
            event_types: None,
            allowed_services: None,
            allowed_teams: None,
            min_priority: None,
            min_urgency: None,
            api_token: None,
        }
    }

    #[test]
    fn test_pagerduty_platform_new() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config);
        assert!(platform.is_ok());
    }

    #[test]
    fn test_signature_verification() {
        let config = create_test_config();
        let platform = PagerDutyPlatform::new(config).unwrap();

        let payload = b"test payload";
        let signature = "v1=invalid";

        let result = platform.verify_signature(payload, signature).await;
        assert!(!result);
    }

    #[test]
    fn test_event_filtering_by_type() {
        let mut config = create_test_config();
        config.event_types = Some(vec![
            "incident.triggered".to_string(),
            "incident.acknowledged".to_string(),
        ]);

        let platform = PagerDutyPlatform::new(config).unwrap();

        let event = create_test_event("incident.triggered");
        assert!(platform.should_process_event(&event));

        let event = create_test_event("incident.escalated");
        assert!(!platform.should_process_event(&event));
    }

    #[test]
    fn test_priority_filtering() {
        let mut config = create_test_config();
        config.min_priority = Some("P2".to_string());

        let platform = PagerDutyPlatform::new(config).unwrap();

        // P1 should pass (higher priority)
        assert!(is_priority_sufficient("P1", "P2"));

        // P2 should pass (equal)
        assert!(is_priority_sufficient("P2", "P2"));

        // P3 should fail (lower priority)
        assert!(!is_priority_sufficient("P3", "P2"));
    }

    #[test]
    fn test_service_filtering() {
        let mut config = create_test_config();
        config.allowed_services = Some(vec!["PXYZ123".to_string()]);

        let platform = PagerDutyPlatform::new(config).unwrap();

        let mut event = create_test_event("incident.triggered");
        event.data.service.id = "PXYZ123".to_string();
        assert!(platform.should_process_event(&event));

        event.data.service.id = "PXYZ999".to_string();
        assert!(!platform.should_process_event(&event));
    }
}
```

### 10.2 Integration Tests

```rust
#[tokio::test]
async fn test_parse_incident_triggered_webhook() {
    let config = create_test_config();
    let platform = PagerDutyPlatform::new(config).unwrap();

    let payload = r#"{
        "event": {
            "id": "01DRF6BV56ABC",
            "event_type": "incident.triggered",
            "resource_type": "incident",
            "occurred_at": "2025-12-23T10:30:00Z",
            "data": {
                "id": "Q2KURS8RXYZ123",
                "type": "incident",
                "status": "triggered",
                "title": "High CPU usage",
                "service": {
                    "id": "PXYZ123",
                    "summary": "Production API"
                },
                "urgency": "high"
            }
        }
    }"#;

    let mut headers = HashMap::new();
    headers.insert("x-pagerduty-signature".to_string(), "v1=abc123".to_string());

    let result = platform.parse_message(payload.as_bytes(), &headers).await;
    assert!(result.is_ok());

    let msg = result.unwrap();
    assert_eq!(msg.platform, "pagerduty");
    assert_eq!(msg.text, "High CPU usage");
    assert_eq!(msg.metadata.get("status").unwrap().as_str().unwrap(), "triggered");
}
```

---

## 11. Error Handling

### 11.1 Platform-Specific Errors

```rust
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unsupported message type")]
    UnsupportedMessageType,

    #[error("Invalid event type: {0}")]
    InvalidEventType(String),

    #[error("Incident not found: {0}")]
    IncidentNotFound(String),
}
```

### 11.2 Error Recovery

```rust
async fn parse_message(
    &self,
    raw: &[u8],
    headers: &HashMap<String, String>,
) -> Result<TriggerMessage, PlatformError> {
    // Verify signature
    if let Some(signature) = headers.get("x-pagerduty-signature") {
        if !self.verify_signature(raw, signature).await {
            warn!("Invalid PagerDuty signature");
            return Err(PlatformError::InvalidSignature(
                "Signature verification failed".to_string(),
            ));
        }
    } else {
        warn!("Missing x-pagerduty-signature header");
        return Err(PlatformError::InvalidSignature(
            "Missing signature header".to_string(),
        ));
    }

    // Parse webhook payload
    let webhook: PagerDutyWebhook = serde_json::from_slice(raw)
        .map_err(|e| {
            error!("Failed to parse PagerDuty webhook: {}", e);
            PlatformError::ParseError(format!("Invalid webhook payload: {}", e))
        })?;

    // Check if event should be processed
    if !self.should_process_event(&webhook.event) {
        debug!("Event filtered out by configuration");
        return Err(PlatformError::UnsupportedMessageType);
    }

    // Convert to TriggerMessage
    self.parse_pagerduty_event(webhook).await
}
```

---

## 12. Security Considerations

### 12.1 Webhook Secret Management

- Store webhook secret in environment variables or secrets management system
- Never commit webhook secrets to version control
- Rotate webhook secrets periodically
- Use different secrets for different environments (dev, staging, production)

### 12.2 Signature Verification

- Always verify `x-pagerduty-signature` header before processing events
- Reject requests with missing or invalid signatures
- Use constant-time comparison to prevent timing attacks
- Log all signature verification failures for security monitoring

### 12.3 API Token Security

- API tokens have full account access - protect them carefully
- Use service accounts with minimal required permissions
- Rotate API tokens regularly
- Monitor API usage for anomalies

### 12.4 Filtering and Validation

- Validate all event types before processing
- Filter by allowed services and teams
- Sanitize all user-provided data (incident titles, notes)
- Rate limit API requests to prevent abuse

---

## 13. Performance Considerations

### 13.1 Webhook Response Time

- PagerDuty expects webhook responses within 5 seconds
- Process events asynchronously if possible
- Return 200 OK immediately, process in background
- Use queues for complex processing workflows

### 13.2 API Rate Limits

PagerDuty API rate limits:
- REST API: 960 requests per minute per API key
- Events API: No documented limit (designed for high throughput)

Implementation:
```rust
use tokio::time::{sleep, Duration};

async fn rate_limited_api_call<F, Fut, T>(
    &self,
    operation: F,
) -> Result<T, PlatformError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, PlatformError>>,
{
    // Simple rate limiting: max 15 requests per second
    const RATE_LIMIT_DELAY: Duration = Duration::from_millis(67);

    let result = operation().await;

    if let Err(PlatformError::RateLimitExceeded) = result {
        warn!("Rate limit hit, backing off");
        sleep(Duration::from_secs(1)).await;
    } else {
        sleep(RATE_LIMIT_DELAY).await;
    }

    result
}
```

---

## 14. Monitoring and Observability

### 14.1 Metrics to Track

```rust
use prometheus::{Counter, Histogram, IntGauge};

lazy_static! {
    static ref WEBHOOK_RECEIVED: Counter = register_counter!(
        "pagerduty_webhooks_received_total",
        "Total number of PagerDuty webhooks received"
    ).unwrap();

    static ref WEBHOOK_PROCESSED: Counter = register_counter!(
        "pagerduty_webhooks_processed_total",
        "Total number of PagerDuty webhooks processed"
    ).unwrap();

    static ref WEBHOOK_ERRORS: Counter = register_counter!(
        "pagerduty_webhooks_errors_total",
        "Total number of PagerDuty webhook processing errors"
    ).unwrap();

    static ref WEBHOOK_LATENCY: Histogram = register_histogram!(
        "pagerduty_webhook_processing_duration_seconds",
        "Time spent processing PagerDuty webhooks"
    ).unwrap();

    static ref ACTIVE_INCIDENTS: IntGauge = register_int_gauge!(
        "pagerduty_active_incidents",
        "Number of active incidents being tracked"
    ).unwrap();
}
```

### 14.2 Logging Strategy

```rust
use tracing::{debug, info, warn, error};

// Log all webhook events
info!(
    event_type = %event.event_type,
    incident_id = %incident.id,
    service = %incident.service.summary,
    "Processing PagerDuty webhook"
);

// Log filtering decisions
debug!(
    event_type = %event.event_type,
    reason = "event_type_filtered",
    "Skipping event"
);

// Log API calls
info!(
    incident_id = %incident_id,
    action = "add_note",
    "Adding note to PagerDuty incident"
);

// Log errors with context
error!(
    error = %e,
    incident_id = %incident_id,
    "Failed to update PagerDuty incident"
);
```

---

## 15. Future Enhancements

### 15.1 Phase 2 Features

- Support for additional event types (annotated, delegated, priority_updated, etc.)
- Custom field extraction and filtering
- Webhook subscription management via API
- Automatic incident correlation
- Multi-account support

### 15.2 Advanced Workflows

- Automated incident routing based on service/team
- Escalation policy integration
- Custom status page updates
- Integration with ITSM tools
- Automated runbook execution

### 15.3 Machine Learning Integration

- Incident prediction and prevention
- Anomaly detection in incident patterns
- Auto-resolution suggestions
- Intelligent alert grouping

---

## 16. References

### 16.1 PagerDuty Documentation

- [Webhooks Overview](https://support.pagerduty.com/main/docs/webhooks)
- [V3 Webhooks API](https://developer.pagerduty.com/docs/88922dc5e1ad1-overview-v2-webhooks)
- [Webhook Signature Verification](https://developer.pagerduty.com/docs/ZG9jOjExMDI5NTkz-verifying-signatures)
- [REST API Reference](https://developer.pagerduty.com/api-reference/9d0b4b12e36f9-list-incidents)
- [Events API v2](https://developer.pagerduty.com/docs/ZG9jOjExMDI5NTgw-events-api-v2-overview)

### 16.2 Related AOF Documentation

- [Trigger Platform Architecture](../architecture/TRIGGERS.md)
- [TriggerPlatform Trait](../../crates/aof-triggers/src/trait.rs)
- [Existing Platform Implementations](../../crates/aof-triggers/src/platforms/)
- [Webhook Handler](../../crates/aof-triggers/src/webhook.rs)

### 16.3 Implementation Checklist

- [ ] Create `pagerduty.rs` in `aof-triggers/src/platforms/`
- [ ] Implement `PagerDutyConfig` struct
- [ ] Implement `PagerDutyPlatform` struct
- [ ] Implement webhook signature verification
- [ ] Implement event parsing and filtering
- [ ] Implement `TriggerPlatform` trait
- [ ] Add to `TypedPlatformConfig` enum
- [ ] Register in `PlatformRegistry`
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update platform capabilities
- [ ] Add user documentation
- [ ] Add example YAML resources
- [ ] Test with real PagerDuty webhooks

---

## Appendix A: Complete Code Structure

```
aof/
├── crates/
│   └── aof-triggers/
│       └── src/
│           ├── platforms/
│           │   ├── mod.rs           (add pagerduty module)
│           │   └── pagerduty.rs     (new file)
│           ├── trait.rs              (TriggerPlatform trait)
│           └── response.rs           (TriggerResponse)
└── docs/
    ├── internal/
    │   └── design/
    │       └── PAGERDUTY-TRIGGER-SPEC.md  (this file)
    └── user/
        └── triggers/
            └── pagerduty.md          (user documentation)
```

## Appendix B: Environment Variables Reference

```bash
# Required
export PAGERDUTY_WEBHOOK_SECRET="abc123def456..."

# Optional
export PAGERDUTY_API_TOKEN="u+abc123..."
export PAGERDUTY_ALLOWED_SERVICES="PXYZ123,PABC456"
export PAGERDUTY_ALLOWED_TEAMS="P456DEF,P789GHI"
export PAGERDUTY_MIN_PRIORITY="P2"
export PAGERDUTY_MIN_URGENCY="high"
export PAGERDUTY_BOT_NAME="aof-incident-bot"
```

---

**Document Version**: 1.0
**Last Updated**: 2025-12-23
**Status**: Design Specification (Ready for Implementation)
**Author**: SPARC Specification Agent
**Review Status**: Pending
