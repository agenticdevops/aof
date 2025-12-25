# New Relic Tool Specification

## 1. Overview

The New Relic Tool provides integration with New Relic's observability platform APIs, enabling agents to query metrics, alerts, incidents, and entities programmatically. This tool follows the existing observability tool pattern in `aof-tools` (like Datadog) and provides comprehensive access to New Relic's monitoring capabilities via the NerdGraph GraphQL API.

### New Relic Platform Capabilities

New Relic is a comprehensive observability platform that provides:

- **APM**: Application Performance Monitoring with distributed tracing
- **Infrastructure**: Host and container monitoring
- **Logs**: Centralized log management
- **NRQL**: New Relic Query Language for flexible data analysis
- **Alerts**: Alert policies and condition management
- **Entities**: Unified entity model for all monitored resources
- **Synthetics**: Synthetic monitoring for availability
- **Browser/Mobile**: Real user monitoring

### API Architecture

New Relic uses **NerdGraph** (GraphQL API) as the primary modern API:
- **US Endpoint**: `https://api.newrelic.com/graphql`
- **EU Endpoint**: `https://api.eu.newrelic.com/graphql`
- Interactive explorer: `https://api.newrelic.com/graphiql`

**IMPORTANT**: REST API keys are being deprecated on March 1, 2025. This implementation uses User API Keys only.

## 2. Tool Operations

### 2.1 NRQL Query (`newrelic_nrql_query`)

Execute NRQL queries against New Relic data.

**Purpose**: Query any New Relic data (metrics, logs, events, traces) using NRQL.

**GraphQL Operation**:
```graphql
query NrqlQuery($accountId: Int!, $nrql: Nrql!) {
  actor {
    account(id: $accountId) {
      nrql(query: $nrql) {
        results
        totalResult
        metadata {
          timeWindow {
            begin
            end
          }
        }
      }
    }
  }
}
```

**Parameters**:
- `account_id` (required): New Relic account ID
- `api_key` (required): User API Key
- `query` (required): NRQL query string
- `region` (optional): `us` or `eu` (default: `us`)

**Example NRQL Queries**:
```sql
-- CPU usage across hosts
SELECT average(cpuPercent) FROM SystemSample FACET hostname SINCE 1 hour ago

-- Error rate by service
SELECT percentage(count(*), WHERE error IS true) FROM Transaction
WHERE appName = 'my-app' SINCE 30 minutes ago TIMESERIES

-- Log count by severity
SELECT count(*) FROM Log FACET level SINCE 1 day ago
```

**Response Format**:
```json
{
  "success": true,
  "data": {
    "results": [
      { "hostname": "web-01", "average.cpuPercent": 45.2 },
      { "hostname": "web-02", "average.cpuPercent": 32.1 }
    ],
    "metadata": {
      "timeWindow": {
        "begin": "2025-12-25T06:00:00Z",
        "end": "2025-12-25T07:00:00Z"
      }
    }
  }
}
```

### 2.2 List Alerts (`newrelic_alerts_list`)

List alert policies and their conditions.

**Purpose**: Retrieve alert configurations for analysis and management.

**GraphQL Operation**:
```graphql
query ListAlertPolicies($accountId: Int!, $cursor: String) {
  actor {
    account(id: $accountId) {
      alerts {
        policiesSearch(cursor: $cursor) {
          policies {
            id
            name
            incidentPreference
          }
          nextCursor
        }
      }
    }
  }
}
```

**Parameters**:
- `account_id` (required): New Relic account ID
- `api_key` (required): User API Key
- `region` (optional): `us` or `eu` (default: `us`)
- `limit` (optional): Max policies to return (default: 50)

### 2.3 List Incidents (`newrelic_incidents_list`)

List active and recent incidents.

**Purpose**: Retrieve incident information for incident response workflows.

**GraphQL Operation**:
```graphql
query ListIncidents($accountId: Int!) {
  actor {
    account(id: $accountId) {
      aiIssues {
        issues(filter: {states: [ACTIVATED, CREATED]}) {
          issues {
            issueId
            title
            priority
            state
            activatedAt
            closedAt
            origins
            entityGuids
          }
        }
      }
    }
  }
}
```

**Parameters**:
- `account_id` (required): New Relic account ID
- `api_key` (required): User API Key
- `region` (optional): `us` or `eu` (default: `us`)
- `states` (optional): Filter by state (`ACTIVATED`, `CREATED`, `CLOSED`)

### 2.4 Entity Search (`newrelic_entity_search`)

Search for monitored entities (applications, hosts, services).

**Purpose**: Find and retrieve entity information across the New Relic platform.

**GraphQL Operation**:
```graphql
query EntitySearch($query: String!) {
  actor {
    entitySearch(query: $query) {
      results {
        entities {
          guid
          name
          type
          domain
          entityType
          reporting
          tags {
            key
            values
          }
          alertSeverity
        }
      }
    }
  }
}
```

**Parameters**:
- `api_key` (required): User API Key
- `query` (required): Entity search query (e.g., `type = 'APPLICATION' AND name LIKE 'my-app'`)
- `region` (optional): `us` or `eu` (default: `us`)
- `limit` (optional): Max entities to return (default: 50)

**Example Queries**:
```
# All APM applications
type = 'APPLICATION'

# Hosts with high CPU
type = 'HOST' AND tags.environment = 'production'

# Services by name
name LIKE 'payment%'
```

### 2.5 Get Metrics (`newrelic_metrics_query`)

Query metric timeslice data for specific entities.

**Purpose**: Retrieve detailed metric data for performance analysis.

**GraphQL Operation**:
```graphql
query MetricsQuery($accountId: Int!, $entityGuid: EntityGuid!, $metricNames: [String!]!, $from: EpochMilliseconds!, $to: EpochMilliseconds!) {
  actor {
    entity(guid: $entityGuid) {
      ... on ApmApplicationEntity {
        metricTimesliceData(
          metrics: { names: $metricNames, values: ["average_response_time", "call_count"] }
          from: $from
          to: $to
        ) {
          results {
            name
            values {
              average
              total
              count
            }
          }
        }
      }
    }
  }
}
```

**Parameters**:
- `account_id` (required): New Relic account ID
- `api_key` (required): User API Key
- `entity_guid` (required): Entity GUID
- `metric_names` (required): Array of metric names
- `from` (required): Start time (epoch milliseconds or ISO 8601)
- `to` (required): End time (epoch milliseconds or ISO 8601)
- `region` (optional): `us` or `eu` (default: `us`)

### 2.6 Acknowledge Incident (`newrelic_incident_ack`)

Acknowledge an active incident.

**Purpose**: Mark an incident as acknowledged in incident response workflows.

**GraphQL Operation**:
```graphql
mutation AcknowledgeIssue($accountId: Int!, $issueId: ID!) {
  aiIssuesAcknowledgeIssue(accountId: $accountId, issueId: $issueId) {
    issue {
      issueId
      state
      acknowledgedAt
    }
  }
}
```

**Parameters**:
- `account_id` (required): New Relic account ID
- `api_key` (required): User API Key
- `issue_id` (required): Issue/Incident ID
- `region` (optional): `us` or `eu` (default: `us`)

## 3. Configuration

### 3.1 Authentication

New Relic uses User API Keys for authentication:

**Header Required**:
```
API-Key: <user_api_key>
```

### 3.2 Configuration Schema

```yaml
# Environment variables (recommended)
NEWRELIC_API_KEY: "NRAK-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
NEWRELIC_ACCOUNT_ID: "1234567"
NEWRELIC_REGION: "us"  # or "eu"

# Or in agent configuration
tools:
  - name: newrelic_nrql_query
    config:
      api_key: "${NEWRELIC_API_KEY}"
      account_id: "${NEWRELIC_ACCOUNT_ID}"
      region: "us"
```

### 3.3 Region Configuration

```rust
fn get_endpoint(region: &str) -> String {
    match region.to_lowercase().as_str() {
        "eu" | "eu1" => "https://api.eu.newrelic.com/graphql",
        _ => "https://api.newrelic.com/graphql",
    }
}
```

## 4. Implementation Details

### 4.1 Tool Structure

```rust
// File: crates/aof-tools/src/tools/newrelic.rs

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

/// Collection of all New Relic tools
pub struct NewRelicTools;

impl NewRelicTools {
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(NewRelicNrqlQueryTool::new()),
            Box::new(NewRelicAlertsListTool::new()),
            Box::new(NewRelicIncidentsListTool::new()),
            Box::new(NewRelicEntitySearchTool::new()),
            Box::new(NewRelicMetricsQueryTool::new()),
            Box::new(NewRelicIncidentAckTool::new()),
        ]
    }
}
```

### 4.2 HTTP Client Setup (GraphQL)

```rust
async fn create_newrelic_client(api_key: &str) -> AofResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "API-Key",
        reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid API key: {}", e)))?,
    );

    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| aof_core::AofError::tool(format!("Failed to create HTTP client: {}", e)))
}

async fn execute_graphql(
    client: &Client,
    endpoint: &str,
    query: &str,
    variables: serde_json::Value,
) -> AofResult<ToolResult> {
    let body = json!({
        "query": query,
        "variables": variables
    });

    let response = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("GraphQL request failed: {}", e)))?;

    let status = response.status().as_u16();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| aof_core::AofError::tool(format!("Failed to parse response: {}", e)))?;

    // Check for GraphQL errors
    if let Some(errors) = body.get("errors") {
        return Ok(ToolResult::error(format!("GraphQL errors: {:?}", errors)));
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!("HTTP {}: {:?}", status, body)));
    }

    Ok(ToolResult::success(body.get("data").cloned().unwrap_or(body)))
}
```

### 4.3 NRQL Query Tool Implementation

```rust
pub struct NewRelicNrqlQueryTool {
    config: ToolConfig,
}

impl NewRelicNrqlQueryTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "api_key": {
                    "type": "string",
                    "description": "New Relic User API Key (NRAK-...)"
                },
                "account_id": {
                    "type": "string",
                    "description": "New Relic Account ID"
                },
                "query": {
                    "type": "string",
                    "description": "NRQL query (e.g., 'SELECT count(*) FROM Transaction')"
                },
                "region": {
                    "type": "string",
                    "description": "New Relic region: 'us' or 'eu'",
                    "default": "us",
                    "enum": ["us", "eu"]
                }
            }),
            vec!["api_key", "account_id", "query"],
        );

        Self {
            config: tool_config_with_timeout(
                "newrelic_nrql_query",
                "Execute NRQL queries against New Relic data. Query metrics, logs, traces, and events.",
                parameters,
                60,
            ),
        }
    }
}

#[async_trait]
impl Tool for NewRelicNrqlQueryTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let api_key: String = input.get_arg("api_key")?;
        let account_id: String = input.get_arg("account_id")?;
        let query: String = input.get_arg("query")?;
        let region: String = input.get_arg("region").unwrap_or_else(|_| "us".to_string());

        debug!(account_id = %account_id, query = %query, "Executing NRQL query");

        let client = create_newrelic_client(&api_key).await?;
        let endpoint = get_endpoint(&region);

        let graphql_query = r#"
            query NrqlQuery($accountId: Int!, $nrql: Nrql!) {
                actor {
                    account(id: $accountId) {
                        nrql(query: $nrql) {
                            results
                            totalResult
                            metadata {
                                timeWindow {
                                    begin
                                    end
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "accountId": account_id.parse::<i64>().unwrap_or(0),
            "nrql": query
        });

        execute_graphql(&client, &endpoint, graphql_query, variables).await
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
```

## 5. Tool Parameters Schema

### 5.1 Common Parameters

```json
{
  "api_key": {
    "type": "string",
    "description": "New Relic User API Key (NRAK-...). Can use env var NEWRELIC_API_KEY"
  },
  "account_id": {
    "type": "string",
    "description": "New Relic Account ID. Can use env var NEWRELIC_ACCOUNT_ID"
  },
  "region": {
    "type": "string",
    "description": "New Relic region: 'us' or 'eu'",
    "default": "us",
    "enum": ["us", "eu"]
  }
}
```

### 5.2 NRQL Query Schema

```json
{
  "type": "object",
  "properties": {
    "api_key": { "type": "string" },
    "account_id": { "type": "string" },
    "query": {
      "type": "string",
      "description": "NRQL query string"
    },
    "region": { "type": "string", "default": "us" }
  },
  "required": ["api_key", "account_id", "query"]
}
```

### 5.3 Entity Search Schema

```json
{
  "type": "object",
  "properties": {
    "api_key": { "type": "string" },
    "query": {
      "type": "string",
      "description": "Entity search query (e.g., \"type = 'APPLICATION'\")"
    },
    "region": { "type": "string", "default": "us" },
    "limit": {
      "type": "integer",
      "description": "Max entities to return",
      "default": 50
    }
  },
  "required": ["api_key", "query"]
}
```

## 6. Error Handling

### 6.1 Common Error Scenarios

**Authentication Errors (401)**:
```json
{
  "errors": [
    {
      "message": "Invalid API key",
      "extensions": { "code": "AUTHENTICATION_ERROR" }
    }
  ]
}
```

**Rate Limiting (429)**:
- 3,000 queries per account per minute
- 25 concurrent NerdGraph requests per user

**Invalid NRQL (400)**:
```json
{
  "errors": [
    {
      "message": "NRQL Syntax Error",
      "extensions": { "code": "NRQL_PARSE_ERROR" }
    }
  ]
}
```

### 6.2 Error Handling Strategy

```rust
async fn handle_newrelic_response(
    response: reqwest::Response,
    operation: &str,
) -> AofResult<ToolResult> {
    let status = response.status().as_u16();
    let body: serde_json::Value = response.json().await
        .map_err(|e| aof_core::AofError::tool(format!("{} parse error: {}", operation, e)))?;

    // Check for GraphQL errors
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
            .map(String::from)
            .collect();

        return Ok(ToolResult::error(format!(
            "{} failed: {}",
            operation,
            error_messages.join("; ")
        )));
    }

    if status >= 400 {
        return Ok(ToolResult::error(format!(
            "{} returned status {}: {:?}",
            operation, status, body
        )));
    }

    Ok(ToolResult::success(body.get("data").cloned().unwrap_or(body)))
}
```

## 7. Example Usage in Agent YAML

### 7.1 Basic NRQL Query Agent

```yaml
name: newrelic-query-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - newrelic_nrql_query

config:
  newrelic:
    api_key: "${NEWRELIC_API_KEY}"
    account_id: "${NEWRELIC_ACCOUNT_ID}"
    region: "us"

system_prompt: |
  You are a New Relic observability agent.

  Query New Relic data using NRQL to analyze:
  - Application performance (Transaction, Span)
  - Infrastructure metrics (SystemSample, ProcessSample)
  - Logs (Log)
  - Custom events

  Use the newrelic_nrql_query tool to retrieve data.

user_prompt: |
  Check the error rate for all production applications in the last hour.
```

### 7.2 Incident Response Agent

```yaml
name: newrelic-incident-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - newrelic_incidents_list
  - newrelic_incident_ack
  - newrelic_nrql_query
  - newrelic_entity_search

config:
  newrelic:
    api_key: "${NEWRELIC_API_KEY}"
    account_id: "${NEWRELIC_ACCOUNT_ID}"

system_prompt: |
  You are a New Relic incident response agent.

  Capabilities:
  1. List active incidents
  2. Acknowledge incidents
  3. Query related metrics and logs
  4. Search for affected entities

  When an incident is reported, gather context and provide recommendations.

user_prompt: |
  List all active critical incidents and provide diagnostic context for each.
```

### 7.3 Comprehensive Observability Agent

```yaml
name: newrelic-observability-agent
version: 1.0.0
model: google:gemini-2.5-flash

tools:
  - newrelic_nrql_query
  - newrelic_alerts_list
  - newrelic_incidents_list
  - newrelic_entity_search
  - newrelic_metrics_query
  - newrelic_incident_ack

config:
  newrelic:
    api_key: "${NEWRELIC_API_KEY}"
    account_id: "${NEWRELIC_ACCOUNT_ID}"

system_prompt: |
  You are a comprehensive New Relic observability agent.

  Capabilities:
  1. Execute NRQL queries for any data analysis
  2. List and manage alert policies
  3. List and acknowledge incidents
  4. Search and analyze entities
  5. Query detailed metrics

  Use all available tools to provide complete observability workflows.

user_prompt: |
  Investigate the production API service:
  1. Check current error rate
  2. List any active incidents
  3. Find related hosts and their status
  4. Query recent error logs
```

## 8. Implementation Checklist

- [ ] Create `NewRelicTools` struct with all 6 operations
- [ ] Implement GraphQL client with User API Key authentication
- [ ] Add region configuration (US/EU endpoints)
- [ ] Implement NRQL query execution
- [ ] Implement alert policy listing
- [ ] Implement incident listing and acknowledgment
- [ ] Implement entity search
- [ ] Implement metrics query
- [ ] Add retry logic for rate limiting
- [ ] Create comprehensive error handling
- [ ] Write unit tests for each tool
- [ ] Add integration tests with mock NerdGraph API
- [ ] Document in `docs/tools/newrelic.md`
- [ ] Add examples to `examples/agents/newrelic-*.yaml`
- [ ] Update `ObservabilityTools::all()` to include New Relic tools
- [ ] Export tools in `crates/aof-tools/src/lib.rs`
- [ ] Add to feature flags in `Cargo.toml` under `observability`

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_endpoint_us() {
        assert_eq!(get_endpoint("us"), "https://api.newrelic.com/graphql");
    }

    #[test]
    fn test_get_endpoint_eu() {
        assert_eq!(get_endpoint("eu"), "https://api.eu.newrelic.com/graphql");
    }

    #[test]
    fn test_tool_config() {
        let tool = NewRelicNrqlQueryTool::new();
        assert_eq!(tool.config().name, "newrelic_nrql_query");
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_nrql_query_integration() {
    // Use wiremock to mock NerdGraph API
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "actor": {
                    "account": {
                        "nrql": {
                            "results": [{ "count": 100 }]
                        }
                    }
                }
            }
        })))
        .mount(&mock_server)
        .await;

    // Test tool execution
}
```

## 10. Rate Limits and Best Practices

### 10.1 Rate Limits

- **NRQL Queries**: 3,000 per account per minute
- **Concurrent Requests**: 25 per user
- **Query Result Limit**: 5,000 rows per query

### 10.2 Best Practices

1. **Use time bounds in NRQL**: Always include `SINCE` and `UNTIL` clauses
2. **Limit result sets**: Use `LIMIT` to prevent excessive data transfer
3. **Cache entity searches**: Entity metadata changes infrequently
4. **Batch operations**: Use concurrent queries where possible
5. **Handle pagination**: Use cursors for large result sets

## 11. Security Considerations

### 11.1 API Key Management

- Store API keys in environment variables
- Never log API keys
- Use User API Keys (not deprecated REST API keys)
- Rotate keys periodically

### 11.2 Query Validation

- Validate NRQL syntax before execution
- Limit query complexity
- Sanitize user inputs in queries

---

**Document Version**: 1.0
**Last Updated**: 2025-12-25
**Author**: AOF Hive Mind Swarm
**Status**: Ready for Implementation
