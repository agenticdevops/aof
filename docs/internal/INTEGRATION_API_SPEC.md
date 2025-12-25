# Platform Integration API Specifications
## New Relic, Splunk, and ServiceNow

**Version:** 1.0
**Date:** 2025-12-25
**Purpose:** Detailed API specifications for implementing built-in tools

---

## 1. New Relic Integration

### 1.1 Authentication

**Method:** API Key
**Header:** `X-Api-Key: {api_key}`
**Key Types:**
- User API Key (personal access)
- REST API Key (deprecated, still supported)

**Regions:**
- US: `https://api.newrelic.com`
- EU: `https://api.eu.newrelic.com`

### 1.2 Tool: newrelic_nrql_query

**Description:** Execute NRQL queries for metrics, events, and logs.

**API Endpoint:**
```
POST https://api.newrelic.com/graphql
```

**Request Body (GraphQL):**
```graphql
{
  actor {
    account(id: ACCOUNT_ID) {
      nrql(query: "SELECT * FROM Transaction LIMIT 100") {
        results
      }
    }
  }
}
```

**Alternatively (REST API v1):**
```
POST https://insights-api.newrelic.com/v1/accounts/{accountId}/query
Content-Type: application/json
X-Query-Key: {query_key}

{
  "nrql": "SELECT average(duration) FROM Transaction WHERE appName = 'MyApp' SINCE 1 hour ago"
}
```

**Tool Parameters:**
```yaml
account_id:
  type: string
  description: New Relic account ID (numeric)
  required: true

query:
  type: string
  description: NRQL query string
  required: true
  example: "SELECT * FROM Transaction WHERE appName = 'MyApp' SINCE 1 hour ago"

api_key:
  type: string
  description: New Relic API key (User API Key)
  required: true

region:
  type: string
  description: API region (us or eu)
  default: us
  enum: [us, eu]
```

**Response Format:**
```json
{
  "results": [
    {
      "timestamp": 1234567890000,
      "average.duration": 0.456
    }
  ],
  "metadata": {
    "contents": [
      {"alias": "timestamp", "type": "timestamp"},
      {"alias": "average.duration", "type": "numeric"}
    ],
    "eventTypes": ["Transaction"],
    "facets": []
  }
}
```

**Error Handling:**
- 400: Invalid NRQL syntax → Return detailed syntax error
- 401: Invalid API key → "Authentication failed. Check API key."
- 403: Account access denied → "Account {id} access denied."
- 429: Rate limit exceeded → "Rate limited. Retry after {seconds}s"

### 1.3 Tool: newrelic_alert_list

**Description:** List alert violations (incidents).

**API Endpoint:**
```
GET https://api.newrelic.com/v2/alerts_violations.json
```

**Request Headers:**
```
X-Api-Key: {api_key}
Content-Type: application/json
```

**Query Parameters:**
```
only_open: boolean (default: true)
start_date: ISO8601 timestamp
end_date: ISO8601 timestamp
```

**Tool Parameters:**
```yaml
api_key:
  type: string
  required: true

only_open:
  type: boolean
  description: Only return open violations
  default: true

start_date:
  type: string
  description: Start time (ISO 8601)
  example: "2025-01-01T00:00:00Z"

end_date:
  type: string
  description: End time (ISO 8601)
```

**Response:**
```json
{
  "violations": [
    {
      "id": 123456,
      "label": "CPU usage > 90%",
      "duration": 3600,
      "opened_at": "2025-01-01T12:00:00Z",
      "closed_at": null,
      "policy_name": "Production Policy",
      "condition_name": "High CPU",
      "priority": "critical"
    }
  ]
}
```

### 1.4 Tool: newrelic_entity_search

**Description:** Search APM entities (applications, hosts, containers).

**API Endpoint (GraphQL):**
```
POST https://api.newrelic.com/graphql
```

**GraphQL Query:**
```graphql
{
  actor {
    entitySearch(query: "type = 'APPLICATION' AND name LIKE 'MyApp'") {
      results {
        entities {
          name
          entityType
          guid
          ... on ApmApplicationEntity {
            applicationId
            language
          }
        }
      }
    }
  }
}
```

**Tool Parameters:**
```yaml
api_key:
  type: string
  required: true

query:
  type: string
  description: Entity search query
  required: true
  example: "type = 'APPLICATION' AND name LIKE 'MyApp'"

entity_type:
  type: string
  description: Filter by entity type
  enum: [APPLICATION, HOST, CONTAINER, SERVICE]
```

### 1.5 Tool: newrelic_metric_data

**Description:** Get metric timeseries data for applications.

**API Endpoint:**
```
GET https://api.newrelic.com/v2/applications/{application_id}/metrics/data.json
```

**Query Parameters:**
```
names[]: Metric names (e.g., "Apdex", "HttpDispatcher")
values[]: Metric values (e.g., "score", "average_response_time")
from: ISO8601 start time
to: ISO8601 end time
period: Aggregation period in seconds (default: 60)
```

**Tool Parameters:**
```yaml
api_key:
  type: string
  required: true

application_id:
  type: integer
  description: APM application ID
  required: true

names:
  type: array
  description: Metric names to retrieve
  items:
    type: string
  example: ["Apdex", "HttpDispatcher"]

values:
  type: array
  description: Metric values to retrieve
  items:
    type: string
  example: ["score", "average_response_time"]

from:
  type: string
  description: Start time (ISO 8601)
  required: true

to:
  type: string
  description: End time (ISO 8601)
  required: true

period:
  type: integer
  description: Aggregation period (seconds)
  default: 60
```

---

## 2. Splunk Integration

### 2.1 Authentication

**Method:** Bearer Token or Basic Auth
**Header:** `Authorization: Bearer {token}` or `Authorization: Basic {base64(user:pass)}`

**Token Generation:**
```bash
# Generate token via CLI
splunk create token -name aof-integration -audience api
```

**Endpoints:**
- Splunk Enterprise: `https://{host}:8089`
- Splunk Cloud: `https://{stack}.splunkcloud.com:8089`

### 2.2 Tool: splunk_search

**Description:** Execute SPL (Splunk Processing Language) search.

**API Flow (Asynchronous):**

1. **Create Search Job:**
```
POST https://{host}:8089/services/search/jobs
Authorization: Bearer {token}
Content-Type: application/x-www-form-urlencoded

search=search index=main error | stats count by host
earliest_time=-1h
latest_time=now
```

**Response:**
```xml
<response>
  <sid>1234567890.12345</sid>
</response>
```

2. **Poll Job Status:**
```
GET https://{host}:8089/services/search/jobs/{sid}
Authorization: Bearer {token}
```

**Response:**
```xml
<entry>
  <content>
    <s:dict>
      <s:key name="isDone">1</s:key>
      <s:key name="resultCount">42</s:key>
    </s:dict>
  </content>
</entry>
```

3. **Get Results:**
```
GET https://{host}:8089/services/search/jobs/{sid}/results
Authorization: Bearer {token}
Accept: application/json
```

**Tool Parameters:**
```yaml
endpoint:
  type: string
  description: Splunk API endpoint
  required: true
  example: "https://splunk.example.com:8089"

auth_token:
  type: string
  description: Authentication token (Bearer token or base64 user:pass)
  required: true

search:
  type: string
  description: SPL search query
  required: true
  example: "search index=main error | stats count by host"

earliest_time:
  type: string
  description: Start time (-1h, -30m, -7d, ISO8601, or epoch)
  default: "-1h"

latest_time:
  type: string
  description: End time (now, ISO8601, or epoch)
  default: "now"

max_count:
  type: integer
  description: Maximum results to return
  default: 10000

output_mode:
  type: string
  description: Result format
  default: "json"
  enum: [json, xml, csv]
```

**Implementation Notes:**
- Use polling with exponential backoff (start 1s, max 5s)
- Default max attempts: 60 (5 minutes total timeout)
- Handle `isDone=0` by continuing to poll
- Support `output.csv` and `output.json` result formats

**Error Handling:**
- 400: Invalid SPL syntax → Parse `<message>` from XML response
- 401: Authentication failed → "Invalid token or credentials"
- 403: Insufficient permissions → "Insufficient search permissions"
- Self-signed cert: Use `.danger_accept_invalid_certs(true)` with opt-in config

### 2.3 Tool: splunk_saved_search_run

**Description:** Run a saved search by name.

**API Endpoint:**
```
POST https://{host}:8089/services/saved/searches/{name}/dispatch
Authorization: Bearer {token}
```

**Tool Parameters:**
```yaml
endpoint:
  type: string
  required: true

auth_token:
  type: string
  required: true

search_name:
  type: string
  description: Name of saved search
  required: true

args:
  type: object
  description: Search arguments (dispatch.earliest_time, etc.)
  example:
    dispatch.earliest_time: "-1h"
    dispatch.latest_time: "now"
```

### 2.4 Tool: splunk_alert_list

**Description:** List triggered alerts.

**API Endpoint:**
```
GET https://{host}:8089/services/alerts/fired_alerts
Authorization: Bearer {token}
```

**Tool Parameters:**
```yaml
endpoint:
  type: string
  required: true

auth_token:
  type: string
  required: true

earliest_time:
  type: string
  default: "-24h"

count:
  type: integer
  description: Maximum alerts to return
  default: 100
```

### 2.5 Tool: splunk_event_submit

**Description:** Submit events to HTTP Event Collector (HEC).

**API Endpoint:**
```
POST https://{host}:8088/services/collector/event
Authorization: Splunk {hec_token}
Content-Type: application/json

{
  "event": "Agent completed RCA analysis",
  "source": "aof-agent",
  "sourcetype": "aof:rca",
  "index": "main",
  "time": 1234567890,
  "fields": {
    "severity": "high",
    "component": "api-service"
  }
}
```

**Tool Parameters:**
```yaml
endpoint:
  type: string
  description: HEC endpoint (port 8088, not 8089)
  required: true

hec_token:
  type: string
  description: HTTP Event Collector token
  required: true

event:
  type: string
  description: Event message
  required: true

source:
  type: string
  default: "aof-agent"

sourcetype:
  type: string
  default: "aof:event"

index:
  type: string
  default: "main"

fields:
  type: object
  description: Additional indexed fields
```

---

## 3. ServiceNow Integration

### 3.1 Authentication

**Method:** Basic Auth (username:password) or OAuth 2.0
**Header:** `Authorization: Basic {base64(username:password)}`
**Content-Type:** `application/json`
**Accept:** `application/json`

**Instance URL:**
```
https://{instance}.service-now.com
```

### 3.2 Tool: servicenow_incident_create

**Description:** Create incident ticket.

**API Endpoint:**
```
POST https://{instance}.service-now.com/api/now/table/incident
Authorization: Basic {base64_credentials}
Content-Type: application/json
Accept: application/json

{
  "short_description": "High CPU on api-service-prod",
  "description": "Detailed RCA: ...",
  "urgency": "1",
  "impact": "2",
  "assignment_group": "SRE Team",
  "caller_id": "aof-agent"
}
```

**Tool Parameters:**
```yaml
instance:
  type: string
  description: ServiceNow instance name (e.g., dev12345)
  required: true

username:
  type: string
  required: true

password:
  type: string
  required: true

short_description:
  type: string
  description: Brief incident summary
  required: true
  max_length: 160

description:
  type: string
  description: Detailed incident description

urgency:
  type: string
  description: Urgency level
  enum: ["1", "2", "3"]  # 1=High, 2=Medium, 3=Low
  default: "2"

impact:
  type: string
  description: Business impact
  enum: ["1", "2", "3"]  # 1=High, 2=Medium, 3=Low
  default: "2"

assignment_group:
  type: string
  description: Group to assign incident to

caller_id:
  type: string
  description: User who reported the incident
  default: "aof-agent"

category:
  type: string
  description: Incident category
  example: "Infrastructure"

subcategory:
  type: string
  description: Incident subcategory
  example: "Performance"
```

**Response:**
```json
{
  "result": {
    "sys_id": "abc123def456",
    "number": "INC0012345",
    "state": "1",
    "short_description": "High CPU on api-service-prod",
    "sys_created_on": "2025-01-01 12:00:00",
    "sys_created_by": "aof-agent"
  }
}
```

**Priority Calculation:**
ServiceNow auto-calculates priority from urgency + impact:
- Urgency 1 + Impact 1 = Priority 1 (Critical)
- Urgency 1 + Impact 2 = Priority 2 (High)
- Urgency 2 + Impact 2 = Priority 3 (Medium)
- Urgency 3 + Impact 3 = Priority 5 (Planning)

### 3.3 Tool: servicenow_incident_update

**Description:** Update existing incident.

**API Endpoint:**
```
PATCH https://{instance}.service-now.com/api/now/table/incident/{sys_id}
Authorization: Basic {base64_credentials}
Content-Type: application/json

{
  "state": "6",  # Resolved
  "work_notes": "Agent completed RCA. Root cause: Memory leak in v2.3.1",
  "close_code": "Solved (Permanently)",
  "close_notes": "Deployed fix in v2.3.2"
}
```

**Tool Parameters:**
```yaml
instance:
  type: string
  required: true

username:
  type: string
  required: true

password:
  type: string
  required: true

sys_id:
  type: string
  description: Incident sys_id (from create response)
  required: true

state:
  type: string
  description: Incident state
  enum: ["1", "2", "3", "6", "7"]  # New, In Progress, On Hold, Resolved, Closed

work_notes:
  type: string
  description: Add work notes (visible to technicians)

close_code:
  type: string
  description: Closure code (required if state=6 or 7)
  example: "Solved (Permanently)"

close_notes:
  type: string
  description: Closure notes (required if state=6 or 7)
```

### 3.4 Tool: servicenow_incident_query

**Description:** Query incidents with filters.

**API Endpoint:**
```
GET https://{instance}.service-now.com/api/now/table/incident
  ?sysparm_query=active=true^priority=1
  &sysparm_limit=10
  &sysparm_fields=number,short_description,state,priority,sys_created_on
  &sysparm_display_value=true
Authorization: Basic {base64_credentials}
```

**Encoded Query Syntax:**
- `^` = AND
- `^OR` = OR
- `=` = equals
- `!=` = not equals
- `LIKE` = contains
- `STARTSWITH` = starts with

**Examples:**
- `active=true^priority=1` → Active AND Priority=1
- `state=1^ORstate=2` → State=New OR State=In Progress
- `short_descriptionLIKEapi-service` → Contains "api-service"

**Tool Parameters:**
```yaml
instance:
  type: string
  required: true

username:
  type: string
  required: true

password:
  type: string
  required: true

query:
  type: string
  description: Encoded query string
  required: true
  example: "active=true^priority=1^assignment_group=SRE Team"

limit:
  type: integer
  description: Maximum records to return
  default: 100
  maximum: 10000

fields:
  type: array
  description: Fields to return
  items:
    type: string
  example: ["number", "short_description", "state", "priority"]

display_value:
  type: boolean
  description: Return display values instead of sys_ids
  default: true
```

### 3.5 Tool: servicenow_cmdb_query

**Description:** Query Configuration Items (CMDB).

**API Endpoint:**
```
GET https://{instance}.service-now.com/api/now/table/cmdb_ci_server
  ?sysparm_query=name=api-service-prod-1
Authorization: Basic {base64_credentials}
```

**Tool Parameters:**
```yaml
instance:
  type: string
  required: true

username:
  type: string
  required: true

password:
  type: string
  required: true

ci_class:
  type: string
  description: CMDB CI class table name
  required: true
  enum:
    - cmdb_ci_server
    - cmdb_ci_vm_instance
    - cmdb_ci_app_server
    - cmdb_ci_database
    - cmdb_ci_service
  example: "cmdb_ci_server"

query:
  type: string
  description: Encoded query
  example: "name=api-service-prod-1"

limit:
  type: integer
  default: 100
```

### 3.6 Tool: servicenow_change_create

**Description:** Create change request.

**API Endpoint:**
```
POST https://{instance}.service-now.com/api/now/table/change_request
Authorization: Basic {base64_credentials}
Content-Type: application/json

{
  "short_description": "Deploy fix for memory leak",
  "description": "Deploy v2.3.2 to fix memory leak in api-service",
  "type": "standard",
  "risk": "3",  # 1=High, 2=Medium, 3=Low, 4=Very Low
  "impact": "2",
  "priority": "2",
  "assignment_group": "SRE Team",
  "start_date": "2025-01-02 00:00:00",
  "end_date": "2025-01-02 02:00:00",
  "justification": "Fix critical production issue"
}
```

**Tool Parameters:**
```yaml
instance:
  type: string
  required: true

username:
  type: string
  required: true

password:
  type: string
  required: true

short_description:
  type: string
  required: true

description:
  type: string

type:
  type: string
  description: Change type
  enum: ["standard", "emergency", "normal"]
  default: "standard"

risk:
  type: string
  description: Risk assessment
  enum: ["1", "2", "3", "4"]  # High, Medium, Low, Very Low
  default: "3"

impact:
  type: string
  enum: ["1", "2", "3"]
  default: "2"

start_date:
  type: string
  description: Planned start (YYYY-MM-DD HH:MM:SS)
  required: true

end_date:
  type: string
  description: Planned end (YYYY-MM-DD HH:MM:SS)
  required: true

assignment_group:
  type: string

justification:
  type: string
  description: Change justification
```

---

## 4. Common Patterns

### 4.1 HTTP Client Configuration

**All tools should use:**
```rust
reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .default_headers(headers)
    .build()
```

**For Splunk self-signed certs (opt-in):**
```rust
.danger_accept_invalid_certs(config.allow_self_signed_certs)
```

### 4.2 Error Response Handling

**Standard pattern:**
```rust
match response.status().as_u16() {
    200 | 201 => { /* success */ },
    400 => ToolResult::error("Invalid request: {parse_error_message}"),
    401 => ToolResult::error("Authentication failed. Check credentials."),
    403 => ToolResult::error("Insufficient permissions for this operation."),
    404 => ToolResult::error("Resource not found: {resource_id}"),
    429 => ToolResult::error("Rate limited. Retry after {retry_after}s"),
    500..=599 => ToolResult::error("Server error ({}): {message}"),
    _ => ToolResult::error("Unexpected status {}: {body}"),
}
```

### 4.3 Time Parsing

**Support multiple formats:**
- Unix timestamp: `1234567890`
- ISO 8601: `2025-01-01T12:00:00Z`
- Relative: `-1h`, `-30m`, `-7d`
- Special: `now`

**Implementation:**
```rust
fn parse_time(s: &str) -> Result<i64, String> {
    // Try Unix timestamp
    if let Ok(ts) = s.parse::<i64>() { return Ok(ts); }

    // Try ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp());
    }

    // Try relative time
    if s.starts_with('-') {
        if let Some(seconds) = parse_relative(&s[1..]) {
            return Ok(chrono::Utc::now().timestamp() - seconds);
        }
    }

    // Special case: now
    if s == "now" {
        return Ok(chrono::Utc::now().timestamp());
    }

    Err(format!("Invalid time format: {}", s))
}
```

### 4.4 Async Polling Pattern (Splunk)

**Template:**
```rust
async fn poll_operation(
    client: &Client,
    operation_id: &str,
    max_attempts: u32,
    initial_interval_ms: u64,
) -> AofResult<serde_json::Value> {
    let mut interval = initial_interval_ms;

    for attempt in 1..=max_attempts {
        let status = check_status(client, operation_id).await?;

        if status.is_complete {
            return get_results(client, operation_id).await;
        }

        if status.is_failed {
            return Err(AofError::tool(format!("Operation failed: {}", status.error)));
        }

        tokio::time::sleep(Duration::from_millis(interval)).await;

        // Exponential backoff
        interval = std::cmp::min(interval * 2, 5000);
    }

    Err(AofError::tool(format!(
        "Operation timed out after {} attempts",
        max_attempts
    )))
}
```

---

## 5. Testing Requirements

### 5.1 Unit Tests (Per Tool)

**Use `mockito` for HTTP mocking:**
```rust
#[tokio::test]
async fn test_newrelic_query_success() {
    let mut server = mockito::Server::new();
    let mock = server.mock("POST", "/graphql")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": {"actor": {"account": {"nrql": {"results": []}}}}}"#)
        .create();

    let tool = NewRelicNRQLQueryTool::new();
    let input = ToolInput::new(serde_json::json!({
        "endpoint": server.url(),
        "account_id": "123",
        "query": "SELECT * FROM Transaction",
        "api_key": "test-key"
    }));

    let result = tool.execute(input).await.unwrap();
    assert!(result.success);
    mock.assert();
}
```

### 5.2 Integration Tests (Optional, requires real credentials)

**Skip by default, run with env var:**
```rust
#[tokio::test]
#[ignore]  // Skip in CI, run manually with: cargo test -- --ignored
async fn test_newrelic_query_real_api() {
    let api_key = std::env::var("NEWRELIC_API_KEY")
        .expect("Set NEWRELIC_API_KEY for integration tests");

    let tool = NewRelicNRQLQueryTool::new();
    // ... test with real API
}
```

---

## 6. Documentation Requirements

### 6.1 Tool Module Documentation

**Each `{platform}.rs` file must have:**
```rust
//! {Platform} Tools
//!
//! Tools for querying and interacting with {Platform}'s API.
//!
//! ## Available Tools
//!
//! - `{tool_name}` - {description}
//! - ...
//!
//! ## Prerequisites
//!
//! - Requires `{feature}` feature flag
//! - Valid API credentials
//!
//! ## Authentication
//!
//! {Authentication details, headers, etc.}
//!
//! ## Example
//!
//! ```yaml
//! tools:
//!   - name: {tool_name}
//!     config:
//!       api_key: "{{ secrets.api_key }}"
//! ```
```

### 6.2 User Documentation

**Create `docs/tools/{platform}.md` with:**
- Overview of the platform
- Available tools list with descriptions
- Authentication setup guide
- Example agent YAML configurations
- Common use cases (RCA, monitoring, incident management)
- Troubleshooting section

---

## 7. Security Considerations

### 7.1 Credential Handling

**NEVER:**
- Log credentials (API keys, passwords)
- Store credentials in code or examples
- Return credentials in error messages

**ALWAYS:**
- Accept credentials as tool arguments
- Document use of Context for secrets management
- Sanitize error messages to remove sensitive data

### 7.2 HTTPS Enforcement

**Default to HTTPS, only allow HTTP with explicit opt-in:**
```rust
if endpoint.starts_with("http://") && !config.allow_insecure {
    return Err(AofError::security(
        "HTTP endpoint not allowed. Use HTTPS or set allow_insecure=true"
    ));
}
```

### 7.3 Certificate Validation

**Self-signed certificates must be opt-in:**
```yaml
tools:
  - name: splunk_search
    config:
      endpoint: "https://splunk.local:8089"
      allow_self_signed_certs: true  # Explicit opt-in
```

---

## Summary

This specification provides:
- ✅ Complete API endpoint documentation for 15+ tools
- ✅ Request/response formats with examples
- ✅ Tool parameter schemas (YAML-compatible)
- ✅ Authentication patterns for each platform
- ✅ Error handling guidelines
- ✅ Testing templates
- ✅ Security best practices

**Implementation teams can use this as a reference to build consistent, well-tested integrations.**
