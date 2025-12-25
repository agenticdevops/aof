# ServiceNow API Integration Research Report

**Research Date**: 2025-12-25
**Researcher**: AOF Hive Mind Research Agent
**Objective**: Comprehensive analysis of ServiceNow APIs for AOF integration

---

## Executive Summary

ServiceNow provides a robust REST API ecosystem for ITSM operations including incident management, CMDB, and event management. This report covers authentication methods, key endpoints, rate limits, and Rust integration recommendations for implementing a ServiceNow integration in AOF.

**Key Findings**:
- ✅ Multiple authentication methods supported (Basic, OAuth 2.0, API Keys, Certificate-based)
- ✅ Comprehensive Table API for CRUD operations on incidents and other ITSM tables
- ✅ CMDB API with Identification and Reconciliation Engine (IRE)
- ✅ Event Management API for automated incident creation
- ⚠️ No official Rust SDK (community options available)
- ⚠️ Rate limiting per user with configurable rules

---

## 1. ServiceNow REST API Overview

### 1.1 API Architecture

ServiceNow stores all data in tables and provides the **Table API** as the primary interface for CRUD operations. The base URL pattern is:

```
https://{instance}.service-now.com/api/now/table/{table_name}
```

### 1.2 Key API Categories

| API Type | Purpose | Base Endpoint |
|----------|---------|---------------|
| Table API | CRUD operations on any ServiceNow table | `/api/now/table/{table}` |
| CMDB API | Configuration Management Database operations | `/api/now/cmdb/instance/{ci_class}` |
| Event Management | Event ingestion and alert creation | Event table: `em_event` |
| Import Set API | Bulk data import with staging tables | `/api/now/import/{import_set}` |
| Scripted REST API | Custom API endpoints | User-defined paths |

---

## 2. Authentication Methods

### 2.1 Basic Authentication (Default)

**Method**: HTTP Basic Auth with username and password
**Headers**: `Authorization: Basic {base64(username:password)}`

**Pros**:
- Simple to implement
- No learning curve
- Works immediately

**Cons**:
- Credentials sent with every request
- Less secure over unencrypted channels
- Not recommended for production

**Example**:
```bash
curl -u username:password \
  https://instance.service-now.com/api/now/table/incident
```

### 2.2 OAuth 2.0 (Recommended)

**Method**: OAuth 2.0 Client Credentials or Authorization Code flow
**Token Types**: Access token (short-lived) + Refresh token (long-lived)

**Flow**:
1. Obtain Client ID and Client Secret from ServiceNow
2. Request access token: `POST /oauth_token.do`
3. Use access token in header: `Authorization: Bearer {token}`
4. Refresh when expired using refresh token

**Pros**:
- More secure than Basic Auth
- Token expiration and refresh
- Granular scope control
- Industry standard

**Cons**:
- More complex implementation
- Requires initial setup in ServiceNow

**Token Request Example**:
```bash
curl -X POST \
  https://instance.service-now.com/oauth_token.do \
  -d "grant_type=client_credentials" \
  -d "client_id={client_id}" \
  -d "client_secret={client_secret}"
```

### 2.3 API Key Authentication

**Method**: Custom API key in header or query parameter
**Implementation**: Via Scripted REST API with custom validation

**Pros**:
- Simple token-based auth
- Can be scoped to specific APIs
- Easy rotation

**Cons**:
- Requires custom implementation
- Not built-in for all APIs

### 2.4 Certificate-Based (Mutual TLS)

**Method**: Client certificate for authentication
**Use Case**: Machine-to-machine integrations requiring highest security

**Pros**:
- Highest security level
- No credentials in transit
- Perfect for automated systems

**Cons**:
- Complex certificate management
- Requires PKI infrastructure

### 2.5 Multi-Factor Authentication (MFA)

**Method**: Username + Password + MFA code
**Use Case**: Interactive user authentication

**Note**: Not suitable for automated integrations

### 2.6 Recommendation for AOF

**Primary**: OAuth 2.0 with client credentials flow
**Fallback**: Basic Auth for development/testing
**Future**: Certificate-based for high-security deployments

---

## 3. Incident Management API (Table API)

### 3.1 Base Endpoint

```
https://{instance}.service-now.com/api/now/table/incident
```

### 3.2 HTTP Methods

| Method | Operation | Description |
|--------|-----------|-------------|
| GET | Read | Retrieve incidents (query with filters) |
| POST | Create | Create new incident |
| PATCH | Update | Update existing incident |
| PUT | Replace | Replace entire incident record |
| DELETE | Delete | Delete incident |

### 3.3 Create Incident (POST)

**Endpoint**: `POST /api/now/table/incident`

**Required Fields**:
- `short_description` - Brief summary
- `caller_id` - User reporting the incident

**Common Optional Fields**:
```json
{
  "short_description": "User cannot access VPN",
  "description": "User reports unable to connect to corporate VPN from home",
  "urgency": "2",
  "impact": "2",
  "priority": "3",
  "category": "Network",
  "subcategory": "VPN",
  "assignment_group": "Network Team",
  "caller_id": "abraham.lincoln",
  "state": "1",
  "contact_type": "email"
}
```

**Response**: Returns created incident with `sys_id` and all fields

### 3.4 Query Incidents (GET)

**Endpoint**: `GET /api/now/table/incident?{query_params}`

**Common Query Parameters**:
- `sysparm_query` - Encoded query (e.g., `state=1^priority=1`)
- `sysparm_limit` - Max records to return (default: 10,000)
- `sysparm_offset` - Pagination offset
- `sysparm_fields` - Comma-separated field list
- `sysparm_display_value` - Return display values (true/false/all)

**Example - Get high-priority open incidents**:
```bash
GET /api/now/table/incident?sysparm_query=state=1^priority=1&sysparm_limit=50
```

### 3.5 Update Incident (PATCH)

**Endpoint**: `PATCH /api/now/table/incident/{sys_id}`

**Example - Resolve incident**:
```json
{
  "state": "6",
  "close_code": "Solved (Permanently)",
  "close_notes": "VPN credentials reset and user confirmed access restored",
  "resolution_code": "Password reset"
}
```

### 3.6 Comments and Work Notes

**Table**: `sys_journal_field`

**Add Comment via Incident Update**:
```json
{
  "comments": "Contacted user via email. Awaiting response.",
  "work_notes": "Internal note: Escalating to network team."
}
```

**Query Comments**:
```bash
GET /api/now/table/sys_journal_field?sysparm_query=element_id={incident_sys_id}
```

### 3.7 State Values

| State | Value | Description |
|-------|-------|-------------|
| New | 1 | Newly created |
| In Progress | 2 | Being worked on |
| On Hold | 3 | Waiting for info |
| Resolved | 6 | Solution provided |
| Closed | 7 | Closed permanently |
| Canceled | 8 | No longer needed |

---

## 4. CMDB API (Configuration Management)

### 4.1 Overview

The Configuration Management Database (CMDB) is a hierarchical structure storing Configuration Items (CIs) and their relationships. The CMDB API provides access to configuration data.

### 4.2 CMDB Structure

```
cmdb (root)
└── cmdb_ci (Configuration Items)
    ├── cmdb_ci_computer (Computers)
    │   ├── cmdb_ci_server (Servers)
    │   └── cmdb_ci_win_server (Windows Servers)
    ├── cmdb_ci_network_adapter (Network Devices)
    ├── cmdb_ci_service (Services)
    ├── cmdb_ci_app_server (Application Servers)
    ├── cmdb_ci_database (Databases)
    └── cmdb_ci_appl (Applications)
```

### 4.3 CMDB Instance API

**Base Endpoint**: `/api/now/cmdb/instance/{ci_class}`

**Example - Get all servers**:
```bash
GET /api/now/cmdb/instance/cmdb_ci_server
```

**Example - Get specific server by name**:
```bash
GET /api/now/cmdb/instance/cmdb_ci_server?sysparm_query=name=prod-web-01
```

### 4.4 CI Relationships

**Table**: `cmdb_rel_ci`

**Query Relationships**:
```bash
GET /api/now/table/cmdb_rel_ci?sysparm_query=parent={ci_sys_id}
```

**Example Relationships**:
- Server → Runs On → Application
- Application → Depends On → Database
- Service → Consists Of → Multiple CIs

### 4.5 Identification and Reconciliation Engine (IRE)

**Important**: The Table API bypasses IRE, which can lead to duplicate CIs. For CMDB operations, consider:

1. **Use CMDB Instance API** instead of Table API for CI operations
2. **Implement de-duplication logic** if using Table API
3. **Check for existing CIs** before creating new ones

**IRE Process**:
1. **Identification**: Matches incoming CI data to existing CIs using identification rules
2. **Reconciliation**: Merges data from multiple sources into single CI record

### 4.6 API Discovery Integration

ServiceNow v1.49+ includes new data model for APIs:

**Tables**:
- `cmdb_ci_api_frontend` - Client-facing API endpoints
- `cmdb_ci_api_backend` - Backend services providing responses

**Use Case**: Track API inventory in CMDB for API gateway management

---

## 5. Event Management API

### 5.1 Overview

Event Management receives events from monitoring tools and creates alerts/incidents based on rules.

### 5.2 Event Ingestion Methods

| Method | Use Case | Complexity |
|--------|----------|------------|
| REST API | Programmatic event creation | Medium |
| SNMP Traps | Network device monitoring | Low |
| Email | Legacy systems | Low (not recommended) |
| MID Server | Secure inbound connections | High |
| Web Services | SOAP/REST integration | Medium |

### 5.3 Event Table

**Table**: `em_event`

**Create Event (POST)**:
```bash
POST /api/now/table/em_event
```

**Event Payload**:
```json
{
  "source": "Prometheus",
  "node": "prod-web-01.example.com",
  "type": "CPU",
  "severity": "2",
  "description": "CPU utilization above 90% for 5 minutes",
  "message_key": "cpu_high_prod_web_01",
  "metric_name": "cpu.utilization",
  "resource": "cpu",
  "time_of_event": "2025-12-25 10:30:00"
}
```

### 5.4 Event Processing Flow

```
External Event
    ↓
Event Transform Rules (em_event table)
    ↓
Event Transform Maps (map to alert fields)
    ↓
Alert Creation (em_alert table)
    ↓
Alert Action Rules
    ↓
Incident Creation (incident table)
```

### 5.5 Alert Action Rules

Alert Action Rules automatically:
- Create incidents from alerts
- Attach knowledge articles
- Trigger orchestration workflows
- Send notifications
- Launch remediation scripts

**Configuration**: Navigate to Event Management → Alert Action Rules

### 5.6 Direct Incident Creation

For simple use cases, skip Event Management and create incidents directly:

```bash
POST /api/now/table/incident
```

**When to use Event Management**:
- ✅ Need de-duplication of events
- ✅ Multiple monitoring sources
- ✅ Complex alert correlation
- ✅ Automated remediation workflows

**When to create incidents directly**:
- ✅ Single source of truth
- ✅ One-to-one event-to-incident mapping
- ✅ Simpler integration

---

## 6. Rate Limits and Best Practices

### 6.1 Rate Limit Architecture

**Key Concepts**:
- **Semaphores**: Internal workers that process concurrent requests
- **Per-User Tracking**: Each user has separate rate limit counters
- **Node-Based Limits**: Multi-node instances have higher capacity

**Default Limits** (varies by license tier):
- Concurrent transactions: ~166 (16 active + 150 queued)
- 167th transaction: HTTP 429 error
- Per-hour limits: Configurable via Rate Limit Rules

### 6.2 Rate Limit Response

**HTTP Status**: `429 Too Many Requests`

**Response Headers**:
- `X-RateLimit-Limit` - Max requests allowed
- `X-RateLimit-Remaining` - Requests remaining
- `X-RateLimit-Reset` - Time when limit resets (epoch)
- `Retry-After` - Seconds to wait before retrying

### 6.3 Rate Limit Rules

**Configuration**: System Settings → Rate Limit Rules

**Rule Types**:
- User-specific limits
- Role-based limits
- Global limits
- API endpoint-specific limits

**Example Rule**:
- User: `integration_user`
- Max Requests: 1000 per hour
- Action: Reject with 429 error

### 6.4 Monitoring Rate Limits

**Navigate to**: System Logs → Rate Limit Violations

**View**:
- Which users exceeded limits
- Which rules were triggered
- Timestamp and frequency
- Request details

### 6.5 Best Practices for Integration

#### 6.5.1 Implement Exponential Backoff

```rust
async fn retry_with_backoff<F, T>(
    mut f: F,
    max_retries: u32
) -> Result<T, Error>
where
    F: FnMut() -> Result<T, Error>
{
    let mut retry_count = 0;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if e.is_rate_limit() && retry_count < max_retries => {
                let wait_seconds = 2_u64.pow(retry_count);
                tokio::time::sleep(Duration::from_secs(wait_seconds)).await;
                retry_count += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

#### 6.5.2 Batch Requests

Instead of:
```rust
// ❌ Multiple individual requests
for incident in incidents {
    create_incident(incident).await?;
}
```

Use:
```rust
// ✅ Batch import
let batch_payload = incidents.into_iter()
    .map(|i| json!(i))
    .collect::<Vec<_>>();

import_batch(batch_payload).await?;
```

#### 6.5.3 Prioritize Critical Requests

```rust
enum RequestPriority {
    Critical,   // P1 incidents
    High,       // P2 incidents
    Normal,     // P3-P5 incidents
    Low,        // Bulk operations
}

// Process critical requests first
critical_queue.process().await?;
normal_queue.process_with_delay(500).await?;
```

#### 6.5.4 Monitor and Alert

```rust
struct RateLimitMonitor {
    remaining: AtomicU32,
    reset_time: AtomicU64,
}

impl RateLimitMonitor {
    fn check_capacity(&self) -> bool {
        let remaining = self.remaining.load(Ordering::Relaxed);
        let threshold = 100; // Alert when < 100 requests remaining

        if remaining < threshold {
            log::warn!("Rate limit approaching: {} requests remaining", remaining);
            // Send alert to monitoring system
        }

        remaining > 0
    }
}
```

#### 6.5.5 Use Dedicated Service Account

- Create dedicated integration user
- Apply specific rate limit rules
- Monitor usage separately
- Avoid sharing credentials

### 6.6 Error Handling Best Practices

#### 6.6.1 HTTP Status Code Handling

```rust
match response.status() {
    200..=299 => { /* Success */ },
    400 => { /* Bad request - fix payload */ },
    401 => { /* Unauthorized - refresh token */ },
    403 => { /* Forbidden - check permissions */ },
    404 => { /* Not found - verify sys_id */ },
    408 => { /* Timeout - retry with backoff */ },
    429 => { /* Rate limit - backoff and retry */ },
    500..=599 => { /* Server error - retry with backoff */ },
    _ => { /* Unknown error - log and alert */ },
}
```

#### 6.6.2 Input Validation

```rust
fn validate_incident_payload(payload: &IncidentPayload) -> Result<(), ValidationError> {
    if payload.short_description.is_empty() {
        return Err(ValidationError::MissingField("short_description"));
    }

    if payload.caller_id.is_empty() {
        return Err(ValidationError::MissingField("caller_id"));
    }

    if payload.urgency < 1 || payload.urgency > 3 {
        return Err(ValidationError::InvalidValue("urgency must be 1-3"));
    }

    Ok(())
}
```

#### 6.6.3 Detailed Logging

```rust
async fn create_incident(payload: IncidentPayload) -> Result<Incident, Error> {
    log::info!("Creating incident: {}", payload.short_description);
    log::debug!("Payload: {:?}", payload);

    match client.post("/api/now/table/incident")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            log::info!("Incident created: sys_id={}", response.sys_id);
            Ok(response)
        }
        Err(e) => {
            log::error!("Failed to create incident: {:?}", e);
            log::error!("Payload: {:?}", payload);
            Err(e)
        }
    }
}
```

#### 6.6.4 Graceful Degradation

```rust
async fn sync_to_servicenow(incidents: Vec<Incident>) -> Result<SyncReport, Error> {
    let mut report = SyncReport::default();

    for incident in incidents {
        match create_incident(incident.clone()).await {
            Ok(result) => {
                report.successful += 1;
                report.created_sys_ids.push(result.sys_id);
            }
            Err(e) if e.is_transient() => {
                // Queue for retry
                report.queued_for_retry += 1;
                retry_queue.push(incident).await?;
            }
            Err(e) => {
                // Permanent failure - log and continue
                report.failed += 1;
                log::error!("Permanent failure for incident: {:?}", e);
            }
        }
    }

    Ok(report)
}
```

---

## 7. Rust Integration Recommendations

### 7.1 Current State

**Official SDK**: ❌ None (JavaScript/TypeScript only)
**Community Crates**: ⚠️ Limited (`srapic-rs` - ServiceNow REST API Client)
**Recommendation**: Build custom client using `reqwest` + `serde`

### 7.2 Available Rust Libraries

#### 7.2.1 srapic-rs

**Repository**: https://github.com/luisnuxx/srapic-rs
**Status**: Community-maintained, limited documentation
**Pros**: Basic REST API coverage
**Cons**: Minimal maintenance, limited features

#### 7.2.2 AWS AppFlow SDK

**Use Case**: If using AWS ecosystem for data transfer
**Crate**: `aws-sdk-appflow`
**Supports**: ServiceNow as data source/destination
**Limitation**: Requires AWS infrastructure

### 7.3 Recommended Architecture for AOF

Build a native Rust client using standard HTTP libraries:

#### 7.3.1 Core Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
url = "2.5"
base64 = "0.22"
```

#### 7.3.2 Client Structure

```rust
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct ServiceNowClient {
    base_url: String,
    client: Client,
    auth: AuthMethod,
}

pub enum AuthMethod {
    Basic { username: String, password: String },
    OAuth { access_token: String, refresh_token: String },
    ApiKey { key: String },
}

impl ServiceNowClient {
    pub fn new(instance: &str, auth: AuthMethod) -> Result<Self> {
        let base_url = format!("https://{}.service-now.com", instance);
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { base_url, client, auth })
    }

    async fn request<T: Serialize>(&self, method: Method, path: &str, body: Option<T>) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, &url);

        // Add authentication
        req = match &self.auth {
            AuthMethod::Basic { username, password } => {
                req.basic_auth(username, Some(password))
            }
            AuthMethod::OAuth { access_token, .. } => {
                req.bearer_auth(access_token)
            }
            AuthMethod::ApiKey { key } => {
                req.header("X-API-Key", key)
            }
        };

        // Add body if present
        if let Some(body) = body {
            req = req.json(&body);
        }

        req.send().await.context("Failed to send request")
    }
}
```

#### 7.3.3 Incident API Implementation

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Incident {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys_id: Option<String>,
    pub short_description: String,
    pub description: Option<String>,
    pub caller_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urgency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignment_group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceNowResponse<T> {
    pub result: T,
}

impl ServiceNowClient {
    pub async fn create_incident(&self, incident: &Incident) -> Result<Incident> {
        let response = self.request(
            Method::POST,
            "/api/now/table/incident",
            Some(incident)
        ).await?;

        match response.status() {
            StatusCode::CREATED => {
                let result: ServiceNowResponse<Incident> = response.json().await?;
                Ok(result.result)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(anyhow::anyhow!("Rate limit exceeded"))
            }
            status => {
                let error_text = response.text().await?;
                Err(anyhow::anyhow!("Request failed with status {}: {}", status, error_text))
            }
        }
    }

    pub async fn get_incident(&self, sys_id: &str) -> Result<Incident> {
        let path = format!("/api/now/table/incident/{}", sys_id);
        let response = self.request::<()>(Method::GET, &path, None).await?;

        let result: ServiceNowResponse<Incident> = response.json().await?;
        Ok(result.result)
    }

    pub async fn query_incidents(&self, query: &str, limit: Option<u32>) -> Result<Vec<Incident>> {
        let mut path = format!("/api/now/table/incident?sysparm_query={}", query);
        if let Some(limit) = limit {
            path.push_str(&format!("&sysparm_limit={}", limit));
        }

        let response = self.request::<()>(Method::GET, &path, None).await?;

        #[derive(Deserialize)]
        struct MultiResult {
            result: Vec<Incident>,
        }

        let result: MultiResult = response.json().await?;
        Ok(result.result)
    }

    pub async fn update_incident(&self, sys_id: &str, updates: &Incident) -> Result<Incident> {
        let path = format!("/api/now/table/incident/{}", sys_id);
        let response = self.request(Method::PATCH, &path, Some(updates)).await?;

        let result: ServiceNowResponse<Incident> = response.json().await?;
        Ok(result.result)
    }
}
```

#### 7.3.4 OAuth Token Management

```rust
#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

impl ServiceNowClient {
    pub async fn obtain_oauth_token(
        instance: &str,
        client_id: &str,
        client_secret: &str
    ) -> Result<AuthMethod> {
        let client = Client::new();
        let url = format!("https://{}.service-now.com/oauth_token.do", instance);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let response = client.post(&url)
            .form(&params)
            .send()
            .await?;

        let token_data: OAuthTokenResponse = response.json().await?;

        Ok(AuthMethod::OAuth {
            access_token: token_data.access_token,
            refresh_token: token_data.refresh_token.unwrap_or_default(),
        })
    }

    pub async fn refresh_oauth_token(&mut self) -> Result<()> {
        if let AuthMethod::OAuth { ref refresh_token, .. } = self.auth {
            let url = format!("{}/oauth_token.do", self.base_url);

            let params = [
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ];

            let response = self.client.post(&url)
                .form(&params)
                .send()
                .await?;

            let token_data: OAuthTokenResponse = response.json().await?;

            self.auth = AuthMethod::OAuth {
                access_token: token_data.access_token,
                refresh_token: token_data.refresh_token.unwrap_or_default(),
            };
        }

        Ok(())
    }
}
```

#### 7.3.5 Rate Limit Handling

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct RateLimitedClient {
    client: ServiceNowClient,
    semaphore: Arc<Semaphore>,
    rate_limit_info: Arc<RwLock<RateLimitInfo>>,
}

#[derive(Default)]
struct RateLimitInfo {
    remaining: u32,
    reset_time: Option<u64>,
}

impl RateLimitedClient {
    pub fn new(client: ServiceNowClient, max_concurrent: usize) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            rate_limit_info: Arc::new(RwLock::new(RateLimitInfo::default())),
        }
    }

    pub async fn create_incident_with_retry(&self, incident: &Incident) -> Result<Incident> {
        let _permit = self.semaphore.acquire().await?;

        for attempt in 0..5 {
            match self.client.create_incident(incident).await {
                Ok(result) => return Ok(result),
                Err(e) if e.to_string().contains("Rate limit") => {
                    let wait_seconds = 2_u64.pow(attempt);
                    tracing::warn!("Rate limited. Waiting {} seconds...", wait_seconds);
                    tokio::time::sleep(Duration::from_secs(wait_seconds)).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(anyhow::anyhow!("Max retries exceeded"))
    }
}
```

### 7.4 Integration with AOF Architecture

#### 7.4.1 Tool Implementation

```rust
// In aof-servicenow crate

use aof_core::{Tool, ToolInput, ToolResult};
use async_trait::async_trait;

pub struct ServiceNowIncidentTool {
    client: ServiceNowClient,
}

#[async_trait]
impl Tool for ServiceNowIncidentTool {
    async fn execute(&self, input: ToolInput) -> ToolResult {
        let action = input.get("action").context("Missing action")?;

        match action.as_str() {
            "create" => {
                let incident = serde_json::from_value(input)?;
                let result = self.client.create_incident(&incident).await?;
                ToolResult::success(serde_json::to_value(result)?)
            }
            "get" => {
                let sys_id = input.get("sys_id").context("Missing sys_id")?;
                let result = self.client.get_incident(sys_id).await?;
                ToolResult::success(serde_json::to_value(result)?)
            }
            "query" => {
                let query = input.get("query").context("Missing query")?;
                let limit = input.get("limit").and_then(|v| v.as_u64().map(|n| n as u32));
                let result = self.client.query_incidents(query, limit).await?;
                ToolResult::success(serde_json::to_value(result)?)
            }
            "update" => {
                let sys_id = input.get("sys_id").context("Missing sys_id")?;
                let updates = serde_json::from_value(input)?;
                let result = self.client.update_incident(sys_id, &updates).await?;
                ToolResult::success(serde_json::to_value(result)?)
            }
            _ => ToolResult::error(format!("Unknown action: {}", action)),
        }
    }
}
```

#### 7.4.2 Agent Configuration

```yaml
# servicenow-incident-agent.yaml
name: servicenow-incident-handler
provider: google:gemini-2.5-flash
system_prompt: |
  You are an incident management agent for ServiceNow.
  You can create, query, update, and resolve incidents.

tools:
  - name: servicenow_incident
    type: custom
    config:
      instance: "${SERVICENOW_INSTANCE}"
      auth:
        type: oauth
        client_id: "${SERVICENOW_CLIENT_ID}"
        client_secret: "${SERVICENOW_CLIENT_SECRET}"

instructions: |
  When handling incidents:
  1. Validate all required fields before creation
  2. Use appropriate priority based on urgency and impact
  3. Add detailed descriptions for context
  4. Update incidents with progress notes
  5. Resolve with clear resolution notes
```

---

## 8. Sample API Calls

### 8.1 Authentication

#### Basic Auth
```bash
curl -u admin:password \
  -H "Accept: application/json" \
  https://dev12345.service-now.com/api/now/table/incident
```

#### OAuth Token Request
```bash
curl -X POST \
  https://dev12345.service-now.com/oauth_token.do \
  -d "grant_type=client_credentials" \
  -d "client_id=abc123" \
  -d "client_secret=secret456"
```

### 8.2 Incident Operations

#### Create Incident
```bash
curl -X POST \
  https://dev12345.service-now.com/api/now/table/incident \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "short_description": "Database connection timeout",
    "description": "Production database connection pool exhausted",
    "caller_id": "admin",
    "urgency": "1",
    "impact": "1",
    "category": "Database",
    "assignment_group": "Database Team"
  }'
```

#### Query Incidents
```bash
# Get all P1 incidents created today
curl -X GET \
  "https://dev12345.service-now.com/api/now/table/incident?sysparm_query=priority=1^sys_created_onONToday@javascript:gs.beginningOfToday()@javascript:gs.endOfToday()" \
  -H "Authorization: Bearer {token}"
```

#### Update Incident
```bash
curl -X PATCH \
  https://dev12345.service-now.com/api/now/table/incident/{sys_id} \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "state": "2",
    "work_notes": "Investigation in progress. Root cause identified."
  }'
```

#### Resolve Incident
```bash
curl -X PATCH \
  https://dev12345.service-now.com/api/now/table/incident/{sys_id} \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "state": "6",
    "close_code": "Solved (Permanently)",
    "close_notes": "Database connection pool size increased from 50 to 100",
    "resolution_code": "Configuration change"
  }'
```

### 8.3 CMDB Operations

#### Get Server Configuration Items
```bash
curl -X GET \
  "https://dev12345.service-now.com/api/now/cmdb/instance/cmdb_ci_server?sysparm_query=operational_status=1" \
  -H "Authorization: Bearer {token}"
```

#### Query CI Relationships
```bash
curl -X GET \
  "https://dev12345.service-now.com/api/now/table/cmdb_rel_ci?sysparm_query=parent={server_sys_id}" \
  -H "Authorization: Bearer {token}"
```

### 8.4 Event Management

#### Create Event
```bash
curl -X POST \
  https://dev12345.service-now.com/api/now/table/em_event \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "source": "Datadog",
    "node": "prod-api-01.example.com",
    "type": "Memory",
    "severity": "3",
    "description": "Memory utilization above 95%",
    "message_key": "memory_high_prod_api_01",
    "metric_name": "system.mem.used",
    "resource": "memory"
  }'
```

---

## 9. Integration Roadmap for AOF

### 9.1 Phase 1: Core Incident Management (MVP)

**Deliverables**:
- ✅ ServiceNow client library (`aof-servicenow` crate)
- ✅ Basic Auth + OAuth 2.0 support
- ✅ CRUD operations for incidents
- ✅ Rate limit handling with exponential backoff
- ✅ Tool implementation for AOF
- ✅ Example agent configuration

**Timeline**: 1-2 weeks

### 9.2 Phase 2: CMDB Integration

**Deliverables**:
- ✅ CMDB API client
- ✅ CI query and relationship mapping
- ✅ IRE-aware CI creation/update
- ✅ Integration with AOF context system

**Timeline**: 2-3 weeks

### 9.3 Phase 3: Event Management

**Deliverables**:
- ✅ Event ingestion API
- ✅ Alert creation and correlation
- ✅ Automated incident creation from events
- ✅ Integration with monitoring tools (Datadog, Grafana)

**Timeline**: 2-3 weeks

### 9.4 Phase 4: Advanced Features

**Deliverables**:
- ✅ Change management API
- ✅ Problem management API
- ✅ Knowledge base integration
- ✅ Attachment handling
- ✅ Webhook support for real-time updates

**Timeline**: 3-4 weeks

---

## 10. Security Considerations

### 10.1 Credential Management

**Recommendations**:
- ✅ Store credentials in environment variables or secrets manager
- ✅ Never commit credentials to version control
- ✅ Rotate OAuth tokens regularly
- ✅ Use dedicated service accounts with minimal permissions

**Example `.env` file**:
```bash
SERVICENOW_INSTANCE=dev12345
SERVICENOW_CLIENT_ID=abc123
SERVICENOW_CLIENT_SECRET=secret456
# OR for Basic Auth (dev only)
SERVICENOW_USERNAME=integration_user
SERVICENOW_PASSWORD=secure_password
```

### 10.2 Permission Management

**ServiceNow Roles Required**:
- `rest_api_explorer` - Access REST API Explorer
- `web_service_admin` - Manage web services
- `itil` - ITSM operations (incident, change, problem)
- `cmdb_read` / `cmdb_write` - CMDB access
- `evt_mgmt_operator` - Event Management operations

**Best Practice**: Create custom role with minimal required permissions

### 10.3 Network Security

**Recommendations**:
- ✅ Use HTTPS only (TLS 1.2+)
- ✅ Implement certificate pinning for production
- ✅ Whitelist IP addresses if possible
- ✅ Use VPN or private network for sensitive environments

### 10.4 Data Protection

**Considerations**:
- ✅ Sanitize sensitive data before logging
- ✅ Encrypt tokens at rest
- ✅ Implement request/response size limits
- ✅ Validate all inputs to prevent injection attacks

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn test_create_incident() {
        let _m = mock("POST", "/api/now/table/incident")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"result": {"sys_id": "123", "short_description": "Test"}}"#)
            .create();

        let client = ServiceNowClient::new("mock", AuthMethod::Basic {
            username: "test".to_string(),
            password: "test".to_string(),
        }).unwrap();

        let incident = Incident {
            sys_id: None,
            short_description: "Test".to_string(),
            description: None,
            caller_id: "admin".to_string(),
            urgency: None,
            impact: None,
            priority: None,
            state: None,
            assignment_group: None,
        };

        let result = client.create_incident(&incident).await.unwrap();
        assert_eq!(result.sys_id.unwrap(), "123");
    }
}
```

### 11.2 Integration Tests

**Requirements**:
- ServiceNow developer instance (free)
- Test data setup script
- Cleanup after tests

```rust
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_real_servicenow_integration() {
    let instance = std::env::var("SERVICENOW_INSTANCE").unwrap();
    let username = std::env::var("SERVICENOW_USERNAME").unwrap();
    let password = std::env::var("SERVICENOW_PASSWORD").unwrap();

    let client = ServiceNowClient::new(&instance, AuthMethod::Basic {
        username,
        password,
    }).unwrap();

    // Create test incident
    let incident = Incident {
        short_description: format!("Test incident {}", Uuid::new_v4()),
        caller_id: "admin".to_string(),
        ..Default::default()
    };

    let created = client.create_incident(&incident).await.unwrap();
    let sys_id = created.sys_id.unwrap();

    // Verify creation
    let fetched = client.get_incident(&sys_id).await.unwrap();
    assert_eq!(fetched.short_description, incident.short_description);

    // Cleanup
    client.delete_incident(&sys_id).await.unwrap();
}
```

---

## 12. Documentation Requirements

### 12.1 User Documentation

**Files to create**:
- `/docs/tools/servicenow.md` - User-facing integration guide
- `/docs/examples/servicenow-incident.md` - Example configurations
- `/docs/tutorials/servicenow-setup.md` - Setup tutorial

**Content**:
- Authentication setup
- Agent configuration examples
- Common use cases
- Troubleshooting guide

### 12.2 Developer Documentation

**Files to create**:
- `/docs/internal/servicenow-architecture.md` - Technical design
- `/docs/reference/servicenow-api.md` - API reference
- `/crates/aof-servicenow/README.md` - Crate documentation

**Content**:
- Architecture diagrams
- API client implementation details
- Extension points
- Testing guidelines

---

## 13. Monitoring and Observability

### 13.1 Metrics to Track

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct ServiceNowMetrics {
    requests_total: Counter,
    requests_failed: Counter,
    request_duration: Histogram,
    rate_limit_hits: Counter,
}

impl ServiceNowMetrics {
    pub fn new(registry: &Registry) -> Self {
        // Define metrics
        Self {
            requests_total: Counter::new("servicenow_requests_total", "Total requests").unwrap(),
            requests_failed: Counter::new("servicenow_requests_failed", "Failed requests").unwrap(),
            request_duration: Histogram::new("servicenow_request_duration_seconds", "Request duration").unwrap(),
            rate_limit_hits: Counter::new("servicenow_rate_limit_hits", "Rate limit hits").unwrap(),
        }
    }

    pub async fn track_request<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>
    {
        let timer = self.request_duration.start_timer();
        self.requests_total.inc();

        match f.await {
            Ok(result) => {
                timer.observe_duration();
                Ok(result)
            }
            Err(e) => {
                self.requests_failed.inc();
                if e.to_string().contains("Rate limit") {
                    self.rate_limit_hits.inc();
                }
                Err(e)
            }
        }
    }
}
```

### 13.2 Logging

```rust
use tracing::{info, warn, error, debug};

impl ServiceNowClient {
    pub async fn create_incident(&self, incident: &Incident) -> Result<Incident> {
        debug!("Creating incident: {:?}", incident);

        let start = Instant::now();
        let result = self.request(Method::POST, "/api/now/table/incident", Some(incident)).await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                info!(
                    duration_ms = elapsed.as_millis(),
                    status = response.status().as_u16(),
                    "Incident created successfully"
                );
                // Process response...
            }
            Err(e) => {
                error!(
                    duration_ms = elapsed.as_millis(),
                    error = %e,
                    "Failed to create incident"
                );
                return Err(e);
            }
        }

        // ...
    }
}
```

---

## 14. Key Takeaways

### ✅ Strengths

1. **Mature API ecosystem** - Comprehensive REST APIs for all ITSM operations
2. **Flexible authentication** - Multiple auth methods for different use cases
3. **Rich data model** - Extensive tables and relationships via CMDB
4. **Event-driven** - Event Management enables automated incident creation
5. **Enterprise-grade** - Rate limiting, monitoring, and security features

### ⚠️ Challenges

1. **No official Rust SDK** - Requires custom client implementation
2. **Complex rate limiting** - Per-user tracking with semaphores
3. **IRE complexity** - CMDB operations need careful handling to avoid duplicates
4. **Authentication management** - OAuth token refresh logic required
5. **Documentation fragmentation** - Information spread across multiple sources

### 🎯 Recommendations

1. **Start with OAuth 2.0** for authentication (production-ready from day 1)
2. **Build rate limit handling early** to avoid production issues
3. **Use Table API for incidents** (simpler than Event Management for basic use cases)
4. **Implement comprehensive logging** for troubleshooting
5. **Create integration tests** with real ServiceNow instance
6. **Document extensively** for future maintainers

---

## Sources

### Authentication
- [ServiceNow REST API Authentication Methods](https://www.servicenow.com/community/developer-forum/rest-api-authentication-methods/m-p/1475577)
- [ServiceNow Basic Authentication Requirements](https://support.servicenow.com/kb?id=kb_article_view&sysparm_article=KB0793963)
- [ServiceNow REST API Documentation](https://www.servicenow.com/docs/bundle/zurich-api-reference/page/integrate/inbound-rest/concept/c_RESTAPI.html)
- [Enhancing API Security Practices in ServiceNow](https://www.reco.ai/hub/enhancing-api-security-practices-in-servicenow)

### Table API & Incident Management
- [ServiceNow Table API Documentation](https://www.servicenow.com/docs/bundle/yokohama-api-reference/page/integrate/inbound-rest/concept/c_TableAPI.html)
- [Creating ServiceNow Incidents via REST API](https://vexpose.blog/2021/04/20/creating-servicenow-incidents-via-rest-api/)
- [How to Retrieve Incidents Using the ServiceNow API](https://www.merge.dev/blog/get-incidents-servicenow-api)
- [REST API Integration using Table API](https://medium.com/@tanawadeashish10/rest-api-integration-using-table-api-in-servicenow-1e769010b1a1)

### CMDB API
- [ServiceNow CMDB REST API Tutorial](https://vexpose.blog/2022/10/28/servicenow-cmdb-rest-api-tutorial/)
- [CMDB Instance API Documentation](https://www.servicenow.com/docs/bundle/zurich-api-reference/page/integrate/inbound-rest/concept/cmdb-instance-api.html)
- [ServiceNow CMDB API Integration Guide](https://virima.com/blog/the-power-of-servicenow-cmdb-api-integrating-with-external-applications)
- [ServiceNow CMDB Inbound API](https://www.conware.eu/blog/servicenow-cmdb-inbound-api)

### Event Management
- [ServiceNow Event Management Overview](https://www.servicenowelite.com/blog/2015/8/19/servicenow-event-management)
- [Technical Insight into ServiceNow Event Management](https://www.emergys.com/blog/technical-insight-into-servicenow-event-management/)

### Rate Limits & Best Practices
- [Understanding ServiceNow REST API Rate Limits](https://www.servicenow.com/community/developer-articles/understanding-servicenow-rest-api-rate-limits-key-concepts-amp/ta-p/3407367)
- [Inbound REST API Rate Limiting](https://www.servicenow.com/docs/bundle/yokohama-api-reference/page/integrate/inbound-rest/concept/inbound-REST-API-rate-limiting.html)
- [ServiceNow Best Practices for Integration Error Handling](https://www.perspectium.com/blog/servicenow-best-practices-for-integration-error-handling/)
- [Handling HTTP Errors in ServiceNow Integrations](https://www.servicenow.com/community/api-insights-articles/handling-http-errors-in-servicenow-integrations-a-comprehensive/ta-p/3066241)

### Rust Integration
- [srapic-rs - ServiceNow REST API Client for Rust](https://github.com/luisnuxx/srapic-rs)
- [ServiceNow SDK Documentation](https://www.servicenow.com/docs/bundle/zurich-application-development/page/build/servicenow-sdk/concept/servicenow-sdk-landing.html)
- [AWS AppFlow Rust SDK](https://lib.rs/crates/aws-sdk-appflow)

---

**End of Report**
