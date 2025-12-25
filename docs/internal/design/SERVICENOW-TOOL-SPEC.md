# ServiceNow Tool Specification

## 1. Overview

The ServiceNow Tool provides integration with ServiceNow's IT Service Management (ITSM) platform, enabling agents to create, query, and update incidents, manage CMDB entries, and interact with change requests programmatically. This tool follows the existing tool pattern in `aof-tools` and provides comprehensive access to ServiceNow's ITSM capabilities.

### ServiceNow Platform Capabilities

ServiceNow is a leading enterprise ITSM platform that provides:

- **Incident Management**: Create, update, and resolve incidents
- **Problem Management**: Track and resolve underlying issues
- **Change Management**: Request and approve changes
- **CMDB**: Configuration Management Database for asset tracking
- **Service Catalog**: Request and provision services
- **Knowledge Base**: Self-service documentation
- **Event Management**: Automated incident creation from events
- **Workflows**: Automated business processes

### API Architecture

ServiceNow uses a REST API (Table API) for CRUD operations:
- **Base URL**: `https://{instance}.service-now.com`
- **Table API**: `/api/now/table/{table_name}`
- **CMDB API**: `/api/now/cmdb/instance/{class}`

## 2. Tool Operations

### 2.1 Create Incident (`servicenow_incident_create`)

Create a new incident in ServiceNow.

**Purpose**: Automate incident creation from AOF agent workflows.

**API Endpoint**: `POST /api/now/table/incident`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password (or OAuth token)
- `short_description` (required): Brief incident summary (max 160 chars)
- `description` (optional): Detailed description
- `caller_id` (optional): User who reported the incident
- `category` (optional): Incident category
- `subcategory` (optional): Incident subcategory
- `urgency` (optional): 1 (High), 2 (Medium), 3 (Low)
- `impact` (optional): 1 (High), 2 (Medium), 3 (Low)
- `assignment_group` (optional): Assignment group sys_id or name
- `assigned_to` (optional): Assigned user sys_id or name
- `cmdb_ci` (optional): Configuration Item sys_id

**Request Format**:
```json
{
  "short_description": "Database connection timeout",
  "description": "Production database experiencing intermittent connection timeouts affecting order processing",
  "caller_id": "admin",
  "urgency": "1",
  "impact": "1",
  "category": "Database",
  "assignment_group": "Database Team"
}
```

**Response Format**:
```json
{
  "success": true,
  "data": {
    "sys_id": "abc123def456",
    "number": "INC0012345",
    "short_description": "Database connection timeout",
    "state": "New",
    "priority": "1 - Critical",
    "sys_created_on": "2025-12-25 07:30:00"
  }
}
```

### 2.2 Query Incidents (`servicenow_incident_query`)

Query incidents with filters.

**Purpose**: Retrieve incident information for analysis and reporting.

**API Endpoint**: `GET /api/now/table/incident`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password
- `query` (optional): Encoded query string (e.g., `priority=1^state=1`)
- `fields` (optional): Comma-separated fields to return
- `limit` (optional): Max results (default: 50)
- `offset` (optional): Pagination offset

**Encoded Query Syntax**:
```
# High priority active incidents
priority=1^state!=6

# Incidents assigned to a group
assignment_group.name=Database Team^state=2

# Created in last 24 hours
sys_created_on>javascript:gs.daysAgo(1)

# Multiple conditions with OR
priority=1^ORpriority=2
```

### 2.3 Update Incident (`servicenow_incident_update`)

Update an existing incident.

**Purpose**: Modify incident fields, add comments, or change status.

**API Endpoint**: `PATCH /api/now/table/incident/{sys_id}`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password
- `sys_id` (required): Incident sys_id
- `fields` (required): Object of fields to update

**Common Update Fields**:
```json
{
  "state": "2",  // In Progress
  "work_notes": "Investigating the issue",
  "assigned_to": "john.doe",
  "close_code": "Solved (Permanently)",
  "close_notes": "Root cause identified and fixed"
}
```

### 2.4 Get Incident (`servicenow_incident_get`)

Get a single incident by sys_id or number.

**Purpose**: Retrieve detailed incident information.

**API Endpoint**: `GET /api/now/table/incident/{sys_id}`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password
- `identifier` (required): Incident sys_id or number (INC0012345)

### 2.5 CMDB Query (`servicenow_cmdb_query`)

Query CMDB Configuration Items.

**Purpose**: Retrieve CI information for incident context and impact analysis.

**API Endpoint**: `GET /api/now/cmdb/instance/{class}`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password
- `class` (required): CI class (e.g., `cmdb_ci_server`, `cmdb_ci_database`)
- `query` (optional): Encoded query string
- `fields` (optional): Comma-separated fields
- `limit` (optional): Max results (default: 50)

**Common CI Classes**:
- `cmdb_ci_server` - Servers
- `cmdb_ci_database` - Databases
- `cmdb_ci_app_server` - Application Servers
- `cmdb_ci_kubernetes_cluster` - Kubernetes Clusters
- `cmdb_ci_cloud_service_account` - Cloud Accounts

### 2.6 Create Change Request (`servicenow_change_create`)

Create a change request.

**Purpose**: Initiate change management workflows from AOF agents.

**API Endpoint**: `POST /api/now/table/change_request`

**Parameters**:
- `instance_url` (required): ServiceNow instance URL
- `username` (required): ServiceNow username
- `password` (required): ServiceNow password
- `short_description` (required): Brief change summary
- `description` (optional): Detailed description
- `type` (optional): Standard, Normal, Emergency
- `risk` (optional): High, Moderate, Low
- `impact` (optional): 1 (High), 2 (Medium), 3 (Low)
- `start_date` (optional): Planned start datetime
- `end_date` (optional): Planned end datetime
- `cmdb_ci` (optional): Affected CI sys_id

## 3. Configuration

### 3.1 Authentication

ServiceNow supports multiple authentication methods:

**Basic Auth (Development)**:
```
Authorization: Basic <base64(username:password)>
```

**OAuth 2.0 (Production Recommended)**:
```
Authorization: Bearer <access_token>
```

**API Key**:
```
x-sn-apikey: <api_key>
```

### 3.2 Configuration Schema

```yaml
# Environment variables (recommended)
SERVICENOW_INSTANCE_URL: "https://company.service-now.com"
SERVICENOW_USERNAME: "api_user"
SERVICENOW_PASSWORD: "secret"
# Or OAuth
SERVICENOW_CLIENT_ID: "client_id"
SERVICENOW_CLIENT_SECRET: "client_secret"

# Or in agent configuration
tools:
  - name: servicenow_incident_create
    config:
      instance_url: "${SERVICENOW_INSTANCE_URL}"
      username: "${SERVICENOW_USERNAME}"
      password: "${SERVICENOW_PASSWORD}"
```

## 4. Implementation Details

### 4.1 Tool Structure

```rust
// File: crates/aof-tools/src/tools/servicenow.rs

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

/// Collection of all ServiceNow tools
pub struct ServiceNowTools;

impl ServiceNowTools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(ServiceNowIncidentCreateTool::new()),
            Box::new(ServiceNowIncidentQueryTool::new()),
            Box::new(ServiceNowIncidentUpdateTool::new()),
            Box::new(ServiceNowIncidentGetTool::new()),
            Box::new(ServiceNowCmdbQueryTool::new()),
            Box::new(ServiceNowChangeCreateTool::new()),
        ]
    }
}
```

### 4.2 HTTP Client Setup

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

async fn create_servicenow_client(username: &str, password: &str) -> AofResult<Client> {
    let credentials = format!("{}:{}", username, password);
    let auth_header = format!("Basic {}", BASE64.encode(credentials.as_bytes()));

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&auth_header)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid auth: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}
```

### 4.3 Incident Create Implementation

```rust
pub struct ServiceNowIncidentCreateTool {
    config: ToolConfig,
}

impl ServiceNowIncidentCreateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL (e.g., https://company.service-now.com)"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "short_description": {
                    "type": "string",
                    "description": "Brief incident summary (max 160 chars)"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed incident description"
                },
                "urgency": {
                    "type": "string",
                    "description": "Urgency: 1 (High), 2 (Medium), 3 (Low)",
                    "enum": ["1", "2", "3"],
                    "default": "2"
                },
                "impact": {
                    "type": "string",
                    "description": "Impact: 1 (High), 2 (Medium), 3 (Low)",
                    "enum": ["1", "2", "3"],
                    "default": "2"
                },
                "category": {
                    "type": "string",
                    "description": "Incident category"
                },
                "assignment_group": {
                    "type": "string",
                    "description": "Assignment group name or sys_id"
                },
                "cmdb_ci": {
                    "type": "string",
                    "description": "Configuration Item sys_id"
                }
            }),
            vec!["instance_url", "username", "password", "short_description"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_create",
                "Create a new incident in ServiceNow for tracking and resolution.",
                parameters,
                60,
            ),
        }
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentCreateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let short_description: String = input.get_arg("short_description")?;
        let description: Option<String> = input.get_arg("description").ok();
        let urgency: String = input.get_arg("urgency").unwrap_or_else(|_| "2".to_string());
        let impact: String = input.get_arg("impact").unwrap_or_else(|_| "2".to_string());
        let category: Option<String> = input.get_arg("category").ok();
        let assignment_group: Option<String> = input.get_arg("assignment_group").ok();
        let cmdb_ci: Option<String> = input.get_arg("cmdb_ci").ok();

        debug!(short_description = %short_description, "Creating ServiceNow incident");

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!("{}/api/now/table/incident", instance_url.trim_end_matches('/'));

        let mut body = json!({
            "short_description": short_description,
            "urgency": urgency,
            "impact": impact
        });

        if let Some(desc) = description {
            body["description"] = json!(desc);
        }
        if let Some(cat) = category {
            body["category"] = json!(cat);
        }
        if let Some(group) = assignment_group {
            body["assignment_group"] = json!(group);
        }
        if let Some(ci) = cmdb_ci {
            body["cmdb_ci"] = json!(ci);
        }

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Request failed: {}", e)))?;

        handle_servicenow_response(response, "Create incident").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 4.4 Query Implementation with Encoded Query

```rust
pub struct ServiceNowIncidentQueryTool {
    config: ToolConfig,
}

impl ServiceNowIncidentQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "instance_url": {
                    "type": "string",
                    "description": "ServiceNow instance URL"
                },
                "username": {
                    "type": "string",
                    "description": "ServiceNow username"
                },
                "password": {
                    "type": "string",
                    "description": "ServiceNow password"
                },
                "query": {
                    "type": "string",
                    "description": "Encoded query (e.g., 'priority=1^state=1')"
                },
                "fields": {
                    "type": "string",
                    "description": "Comma-separated fields to return"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results",
                    "default": 50
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset",
                    "default": 0
                }
            }),
            vec!["instance_url", "username", "password"],
        );

        Self {
            config: tool_config_with_timeout(
                "servicenow_incident_query",
                "Query incidents from ServiceNow with filters and pagination.",
                parameters,
                60,
            ),
        }
    }
}

#[async_trait]
impl Tool for ServiceNowIncidentQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let instance_url: String = input.get_arg("instance_url")?;
        let username: String = input.get_arg("username")?;
        let password: String = input.get_arg("password")?;
        let query: Option<String> = input.get_arg("query").ok();
        let fields: Option<String> = input.get_arg("fields").ok();
        let limit: i32 = input.get_arg("limit").unwrap_or(50);
        let offset: i32 = input.get_arg("offset").unwrap_or(0);

        let client = create_servicenow_client(&username, &password).await?;
        let url = format!("{}/api/now/table/incident", instance_url.trim_end_matches('/'));

        let mut params: Vec<(&str, String)> = vec![
            ("sysparm_limit", limit.to_string()),
            ("sysparm_offset", offset.to_string()),
        ];

        if let Some(q) = query {
            params.push(("sysparm_query", q));
        }
        if let Some(f) = fields {
            params.push(("sysparm_fields", f));
        }

        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| aof_core::AofError::tool(format!("Query failed: {}", e)))?;

        handle_servicenow_response(response, "Query incidents").await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

### 4.5 Response Handling

```rust
async fn handle_servicenow_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("{} parse error: {}", operation, e)))?;

    // Check for ServiceNow error response
    if let Some(error) = body.get("error") {
        let message = error.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        let detail = error.get("detail")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        return Ok(ToolResult::error(format!(
            "{} failed: {} - {}",
            operation, message, detail
        )));
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!(
            "{} HTTP {}: {:?}",
            operation, status, body
        )));
    }

    // Extract result from response
    let result = body.get("result").cloned().unwrap_or(body);
    Ok(ToolResult::success(result))
}
```

## 5. Tool Parameters Schema

### 5.1 Common Parameters

```json
{
  "instance_url": {
    "type": "string",
    "description": "ServiceNow instance URL (https://company.service-now.com)"
  },
  "username": {
    "type": "string",
    "description": "ServiceNow username. Can use env var SERVICENOW_USERNAME"
  },
  "password": {
    "type": "string",
    "description": "ServiceNow password. Can use env var SERVICENOW_PASSWORD"
  }
}
```

### 5.2 Incident Create Schema

```json
{
  "type": "object",
  "properties": {
    "instance_url": { "type": "string" },
    "username": { "type": "string" },
    "password": { "type": "string" },
    "short_description": {
      "type": "string",
      "description": "Brief incident summary (max 160 chars)"
    },
    "description": {
      "type": "string",
      "description": "Detailed description"
    },
    "urgency": {
      "type": "string",
      "enum": ["1", "2", "3"],
      "default": "2"
    },
    "impact": {
      "type": "string",
      "enum": ["1", "2", "3"],
      "default": "2"
    },
    "category": { "type": "string" },
    "subcategory": { "type": "string" },
    "assignment_group": { "type": "string" },
    "assigned_to": { "type": "string" },
    "cmdb_ci": { "type": "string" }
  },
  "required": ["instance_url", "username", "password", "short_description"]
}
```

### 5.3 Incident Query Schema

```json
{
  "type": "object",
  "properties": {
    "instance_url": { "type": "string" },
    "username": { "type": "string" },
    "password": { "type": "string" },
    "query": {
      "type": "string",
      "description": "Encoded query (e.g., 'priority=1^state!=6')"
    },
    "fields": {
      "type": "string",
      "description": "Comma-separated fields (e.g., 'number,short_description,state')"
    },
    "limit": {
      "type": "integer",
      "default": 50
    },
    "offset": {
      "type": "integer",
      "default": 0
    }
  },
  "required": ["instance_url", "username", "password"]
}
```

## 6. Error Handling

### 6.1 Common Error Scenarios

**Authentication Errors (401)**:
```json
{
  "error": {
    "message": "User Not Authenticated",
    "detail": "Required to provide Auth information"
  },
  "status": "failure"
}
```

**Record Not Found (404)**:
```json
{
  "error": {
    "message": "No Record found",
    "detail": "Record doesn't exist or ACL restricts the record retrieval"
  },
  "status": "failure"
}
```

**Validation Errors (400)**:
```json
{
  "error": {
    "message": "Validation error",
    "detail": "short_description is a mandatory field"
  },
  "status": "failure"
}
```

### 6.2 Rate Limiting

ServiceNow implements rate limiting with configurable rules:
- Default: ~166 concurrent transactions per semaphore
- HTTP 429 when limits exceeded
- Headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### 6.3 Retry Strategy

```rust
async fn execute_with_retry<F, Fut>(
    operation: F,
    max_retries: u32,
) -> AofResult<ToolResult>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = AofResult<ToolResult>>,
{
    let mut attempt = 0;
    let mut backoff = std::time::Duration::from_millis(500);

    loop {
        match operation().await {
            Ok(result) => {
                if let Some(error) = result.error() {
                    if error.contains("429") || error.contains("rate limit") {
                        attempt += 1;
                        if attempt >= max_retries {
                            return Ok(result);
                        }
                        tokio::time::sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                }
                return Ok(result);
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 7. Example Usage in Agent YAML

### 7.1 Incident Management Agent

```yaml
name: servicenow-incident-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - servicenow_incident_create
  - servicenow_incident_query
  - servicenow_incident_update
  - servicenow_incident_get

config:
  servicenow:
    instance_url: "${SERVICENOW_INSTANCE_URL}"
    username: "${SERVICENOW_USERNAME}"
    password: "${SERVICENOW_PASSWORD}"

system_prompt: |
  You are a ServiceNow incident management agent.

  Capabilities:
  1. Create new incidents with proper categorization
  2. Query existing incidents with filters
  3. Update incident status and add work notes
  4. Retrieve incident details

  Incident States:
  - 1: New
  - 2: In Progress
  - 3: On Hold
  - 6: Resolved
  - 7: Closed

  Priority is calculated from Urgency x Impact.

user_prompt: |
  Create a high-priority incident for a database connection issue
  affecting the payment service.
```

### 7.2 CMDB Query Agent

```yaml
name: servicenow-cmdb-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - servicenow_cmdb_query
  - servicenow_incident_query

config:
  servicenow:
    instance_url: "${SERVICENOW_INSTANCE_URL}"
    username: "${SERVICENOW_USERNAME}"
    password: "${SERVICENOW_PASSWORD}"

system_prompt: |
  You are a ServiceNow CMDB analysis agent.

  Query the CMDB to:
  1. Find Configuration Items (CIs)
  2. Analyze CI relationships
  3. Identify impact of outages
  4. Correlate incidents with CIs

  Common CI classes:
  - cmdb_ci_server
  - cmdb_ci_database
  - cmdb_ci_app_server
  - cmdb_ci_kubernetes_cluster

user_prompt: |
  Find all production database servers and list any active incidents
  affecting them.
```

### 7.3 Change Management Agent

```yaml
name: servicenow-change-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - servicenow_change_create
  - servicenow_cmdb_query

config:
  servicenow:
    instance_url: "${SERVICENOW_INSTANCE_URL}"
    username: "${SERVICENOW_USERNAME}"
    password: "${SERVICENOW_PASSWORD}"

system_prompt: |
  You are a ServiceNow change management agent.

  Capabilities:
  1. Create change requests for infrastructure changes
  2. Query CMDB for affected CIs
  3. Assess change risk and impact

  Change Types:
  - Standard: Pre-approved, low risk
  - Normal: Requires CAB approval
  - Emergency: Expedited approval process

user_prompt: |
  Create a normal change request to upgrade the production Kubernetes
  cluster to version 1.28.
```

### 7.4 Integrated Incident Response Agent

```yaml
name: servicenow-integrated-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - servicenow_incident_create
  - servicenow_incident_query
  - servicenow_incident_update
  - servicenow_cmdb_query
  - servicenow_change_create

config:
  servicenow:
    instance_url: "${SERVICENOW_INSTANCE_URL}"
    username: "${SERVICENOW_USERNAME}"
    password: "${SERVICENOW_PASSWORD}"

system_prompt: |
  You are a comprehensive ServiceNow ITSM agent.

  Capabilities:
  1. Full incident lifecycle management
  2. CMDB queries for impact analysis
  3. Change request creation

  Workflow:
  1. Query CMDB for affected CIs
  2. Create incident with proper CI linkage
  3. Update incident with investigation notes
  4. Create change request if needed
  5. Resolve incident when fixed

user_prompt: |
  A critical production issue has been reported. Investigate the
  database servers, create an incident, and prepare a change request
  for the fix.
```

## 8. Implementation Checklist

- [ ] Create `ServiceNowTools` struct with all 6 operations
- [ ] Implement Basic Auth and OAuth 2.0 authentication
- [ ] Implement incident CRUD operations
- [ ] Implement CMDB query functionality
- [ ] Implement change request creation
- [ ] Add encoded query builder utilities
- [ ] Implement rate limit handling with retry
- [ ] Create comprehensive error handling
- [ ] Write unit tests for each tool
- [ ] Add integration tests with mock ServiceNow API
- [ ] Document in `docs/tools/servicenow.md`
- [ ] Add examples to `examples/agents/servicenow-*.yaml`
- [ ] Add feature flag `itsm` in `Cargo.toml`
- [ ] Export tools in `crates/aof-tools/src/lib.rs`

## 9. Rate Limits and Best Practices

### 9.1 Rate Limits

- **Default**: ~166 concurrent transactions
- **Per-user limits**: Configurable per instance
- **Batch operations**: Available for bulk updates

### 9.2 Best Practices

1. **Use encoded queries**: More efficient than client-side filtering
2. **Limit fields**: Use `sysparm_fields` to reduce payload size
3. **Paginate results**: Use `sysparm_limit` and `sysparm_offset`
4. **Use display values**: `sysparm_display_value=true` for readable output
5. **Cache static data**: CI data, categories, assignment groups

## 10. Security Considerations

### 10.1 Credential Management

- Store credentials in environment variables
- Use OAuth 2.0 for production
- Create dedicated API users with minimal permissions
- Rotate credentials regularly

### 10.2 Access Control

- Use ServiceNow ACLs to restrict API access
- Audit API usage
- Implement IP whitelisting if possible

---

**Document Version**: 1.0
**Last Updated**: 2025-12-25
**Author**: AOF Hive Mind Swarm
**Status**: Ready for Implementation
