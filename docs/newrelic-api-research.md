# New Relic API Research for AOF Integration

**Research Date**: December 25, 2025
**Purpose**: Comprehensive API analysis for implementing New Relic integration in AOF

---

## Executive Summary

New Relic provides multiple API options for integration, with **NerdGraph (GraphQL API)** being the recommended modern API. The REST API v2 is being phased out in favor of NerdGraph, and **REST API keys are being deprecated** (EOL: March 1, 2025) in favor of User API keys.

**Key Finding**: For AOF integration, we should:
1. Use **NerdGraph (GraphQL)** as the primary API
2. Implement **User API Key** authentication
3. Leverage existing **Rust crates** (community-maintained) or build custom HTTP/GraphQL client
4. Focus on alerts, incidents, and NRQL query capabilities

---

## 1. API Overview

### 1.1 Available APIs

| API | Status | Use Case | Endpoint |
|-----|--------|----------|----------|
| **NerdGraph (GraphQL)** | ✅ Recommended | Querying data, configuration, modern API | `https://api.newrelic.com/graphql` |
| **REST API v2** | ⚠️ Legacy (being replaced) | Legacy querying and configuration | `https://api.newrelic.com/v2/...` |
| **Events API** | ✅ Active | Send custom events | `https://insights-collector.newrelic.com/v1/accounts/{accountId}/events` |
| **Log API** | ✅ Active | Send log data | HTTP endpoint for logs |
| **Metric API** | ✅ Active | Send metric data | Metric ingest endpoint |

### 1.2 NerdGraph (GraphQL API) - Recommended

**Advantages**:
- Single unified API interface for all New Relic services
- Request exactly the data needed (no over/under-fetching)
- Query multiple data sources in a single request
- Modern GraphQL format with strong typing
- Interactive explorer at `https://api.newrelic.com/graphiql`

**Key Capabilities**:
- Query account information, infrastructure data, APM metrics
- Execute NRQL queries
- Manage alerts, policies, and conditions
- Configure features (tags, workloads, golden metrics)
- Access entity data and relationships

**GraphiQL Explorer**: Provides interactive query builder with schema documentation and code generation (curl, New Relic CLI).

### 1.3 REST API v2 - Legacy

⚠️ **Important**: New Relic is transitioning from REST API v2 to NerdGraph. While still functional, new integrations should use NerdGraph.

**Remaining Use Cases**:
- Some legacy alert endpoints
- Infrastructure alert conditions (`https://infra-api.newrelic.com/v2/alerts/...`)

---

## 2. Authentication

### 2.1 API Key Types

| Key Type | Purpose | Scope | Format |
|----------|---------|-------|--------|
| **User Key** (Personal API Key) | NerdGraph & REST API | User-specific, multi-account access | Tied to individual user |
| **License Key** (Ingest Key) | Data ingest (metrics, events, logs) | Account-level | 40-character hex string |
| **Browser Key** | Browser monitoring data | Account-level | N/A |
| **Mobile App Token** | Mobile monitoring data | Account-level | N/A |

### 2.2 User API Key (Recommended for AOF)

**Characteristics**:
- Required for **NerdGraph** and **REST API**
- Tied to a specific New Relic user
- Provides access to all accounts the user has permissions for
- **Cannot be transferred** between users
- Automatically deactivated if the user is deleted

**Security**:
- Treat as sensitive credentials (like passwords)
- Implement key rotation strategy
- Store securely in environment variables or secret managers

### 2.3 REST API Keys Deprecation

🚨 **CRITICAL**: REST API keys are being retired on **March 1, 2025**.

**Migration Required**:
- Replace all REST API keys with **User API keys** before March 1, 2025
- Failure to migrate will cause API call failures and service disruptions
- User API keys provide better security and auditing capabilities

**Migration Steps**:
1. Identify which New Relic account needs API access
2. Create new User API keys in New Relic UI (User menu → API keys)
3. Update systems/integrations to use new User API keys
4. Test thoroughly before March 1, 2025

### 2.4 Authentication Format

**NerdGraph (GraphQL)**:
```bash
curl https://api.newrelic.com/graphql \
  -H "Content-Type: application/json" \
  -H "API-Key: YOUR_USER_API_KEY" \
  -d '{"query": "{ actor { user { name email } } }"}'
```

**REST API v2**:
```bash
curl -X GET https://api.newrelic.com/v2/applications.json \
  -H "Api-Key: YOUR_USER_API_KEY"
```

**Rust Implementation**:
```rust
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

let mut headers = HeaderMap::new();
headers.insert("API-Key", HeaderValue::from_str(&api_key)?);
headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

let client = reqwest::Client::builder()
    .default_headers(headers)
    .build()?;
```

---

## 3. Key Endpoints for AOF Integration

### 3.1 Alerts & Incidents Management

**Primary API**: NerdGraph (GraphQL)

**Key Capabilities**:
- Query active incidents
- Manage alert policies and conditions
- Configure NRQL alert conditions
- Set up workflows and notifications

**GraphQL Query Example - List Incidents**:
```graphql
{
  actor {
    account(id: YOUR_ACCOUNT_ID) {
      alerts {
        incidents {
          incidents {
            incidentId
            title
            priority
            state
            createdAt
            closedAt
            policyName
            conditionName
            violationUuids
          }
        }
      }
    }
  }
}
```

**REST API Endpoints (Legacy, but still functional)**:

| Operation | Method | Endpoint |
|-----------|--------|----------|
| List alert policies | GET | `/v2/alerts_policies.json` |
| Create alert policy | POST | `/v2/alerts_policies.json` |
| Create alert condition | POST | `/v2/alerts_conditions/policies/{policy_id}.json` |
| Create NRQL condition | POST | `/v2/alerts_nrql_conditions/policies/{policy_id}.json` |
| Get infrastructure conditions | GET | `https://infra-api.newrelic.com/v2/alerts/conditions?policy_id={id}` |
| Delete infrastructure condition | DELETE | `https://infra-api.newrelic.com/v2/alerts/conditions/{condition_id}` |

**Incident Event API**:
```bash
POST https://insights-collector.newrelic.com/v1/accounts/{accountId}/events
Content-Type: application/json
Api-Key: YOUR_LICENSE_KEY

{
  "eventType": "IncidentEvent",
  "title": "Custom Incident",
  "description": "Description of the incident",
  "severity": "CRITICAL"
}
```

### 3.2 Metrics Querying (APM, Infrastructure, Synthetics)

**NerdGraph NRQL Query**:
```graphql
{
  actor {
    account(id: YOUR_ACCOUNT_ID) {
      nrql(query: "SELECT average(duration) FROM Transaction WHERE appName='MyApp' SINCE 1 hour ago") {
        results
      }
    }
  }
}
```

**Supported Metric Types**:
- **APM**: `apm_app_metric`, `apm_kt_metric`, `apm_external_service`
- **Browser**: `browser_metric`
- **Mobile**: `mobile_metric`, `mobile_external_service`
- **Infrastructure**: Infrastructure metrics via NerdGraph
- **Synthetics**: Monitor results and metrics

### 3.3 NRQL Query Execution

**NRQL (New Relic Query Language)** is SQL-like syntax for querying data.

**GraphQL Example**:
```graphql
{
  actor {
    account(id: YOUR_ACCOUNT_ID) {
      nrql(query: "SELECT count(*) FROM Transaction FACET appName SINCE 1 day ago") {
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

**Common NRQL Queries for AOF**:
```sql
-- List recent incidents
SELECT * FROM AlertIncident SINCE 1 day ago

-- Application performance
SELECT average(duration), percentile(duration, 95)
FROM Transaction
WHERE appName = 'MyApp'
FACET name
SINCE 1 hour ago

-- Infrastructure health
SELECT average(cpuPercent), average(memoryUsedPercent)
FROM SystemSample
FACET hostname
SINCE 30 minutes ago

-- Error rates
SELECT count(*) FROM TransactionError
FACET error.class
SINCE 1 day ago
```

### 3.4 Entity API (Infrastructure, Applications, Services)

**NerdGraph Entity Query**:
```graphql
{
  actor {
    entitySearch(query: "domain = 'APM' AND type = 'APPLICATION'") {
      results {
        entities {
          guid
          name
          entityType
          domain
          ... on ApmApplicationEntity {
            settings {
              apdexTarget
            }
            alertSeverity
          }
        }
      }
    }
  }
}
```

---

## 4. Rate Limits & Best Practices

### 4.1 NRQL Query Rate Limits

| Limit Type | Standard | Data Plus |
|------------|----------|-----------|
| **Queries per minute** | 3,000 per account | 3,000 per account |
| **Concurrent queries** | ~5 recommended (complex) | ~5 recommended (complex) |
| **NerdGraph concurrent requests** | 25 per user | 25 per user |
| **Inspected data points** | N/A | 150 billion (10B/min sustained) |
| **Result limit (LIMIT clause)** | 5,000 results | 5,000 results |
| **Query duration** | Varies by account | Varies by account |

**Notes**:
- Rate limits apply to **API queries only**, not UI queries
- Complex queries (with `FACET`, `TIMESERIES`, or >1M events) consume more resources
- Use the **Limits UI** in New Relic to monitor usage

### 4.2 Best Practices

**Query Optimization**:
1. Use `LIMIT` clause to restrict results (max 5,000)
2. Avoid overly broad time ranges (use `SINCE`, `UNTIL`)
3. Use `FACET` judiciously (adds complexity)
4. Implement caching for frequently requested data
5. Batch queries when possible (GraphQL supports multiple queries)

**Authentication**:
1. Rotate User API keys regularly
2. Use environment variables or secret managers
3. Never hardcode API keys in source code
4. Monitor API key usage via audit logs

**Error Handling**:
1. Implement exponential backoff for rate limit errors (429)
2. Handle authentication errors (401, 403)
3. Parse GraphQL errors (check `errors` field in response)
4. Log failed requests for troubleshooting

**Regional Considerations**:
- **US datacenter**: `https://api.newrelic.com/graphql`
- **EU datacenter**: `https://api.eu.newrelic.com/graphql`

**Async Processing**:
- Event API is asynchronous (low latency, high volume)
- Use for custom incident events, logs, and metrics

---

## 5. Rust Integration Options

### 5.1 Available Rust Crates

| Crate | Type | Status | Use Case |
|-------|------|--------|----------|
| **newrelic** | C SDK wrapper | Community | APM instrumentation (requires daemon) |
| **newrelic-telemetry-sdk-rust** | Official telemetry SDK | Alpha (not on crates.io) | Sending spans/telemetry to New Relic |
| **newrelic-unofficial** | Pure Rust agent | Community | Thread-safe APM alternative to C SDK |
| **tracing-newrelic** | Tracing integration | Community | Rust `tracing` framework integration |
| **tokio-newrelic** | Async wrapper | Community | Async Rust with Tokio |
| **newrelic-sys** | C SDK bindings | Community | Low-level C SDK bindings |

### 5.2 Recommended Approach for AOF

**Option 1: Custom HTTP/GraphQL Client (Recommended)**

**Pros**:
- Full control over API interactions
- No dependency on C SDK or daemon
- Direct NerdGraph integration
- Lightweight and flexible

**Cons**:
- Requires custom implementation
- More initial development effort

**Implementation**:
```rust
use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
    path: Option<Vec<String>>,
}

pub struct NewRelicClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

impl NewRelicClient {
    pub fn new(api_key: String, region: &str) -> Result<Self> {
        let endpoint = match region {
            "eu" => "https://api.eu.newrelic.com/graphql",
            _ => "https://api.newrelic.com/graphql",
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("API-Key", api_key.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            api_key,
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<T> {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response: GraphQLResponse<T> = self.client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(errors) = response.errors {
            return Err(anyhow::anyhow!(
                "GraphQL errors: {:?}",
                errors.iter().map(|e| &e.message).collect::<Vec<_>>()
            ));
        }

        response.data.ok_or_else(|| anyhow::anyhow!("No data in response"))
    }

    pub async fn nrql_query(&self, account_id: u64, nrql: &str) -> Result<serde_json::Value> {
        let query = format!(
            r#"
            {{
              actor {{
                account(id: {}) {{
                  nrql(query: "{}") {{
                    results
                  }}
                }}
              }}
            }}
            "#,
            account_id, nrql
        );

        #[derive(Deserialize)]
        struct NrqlResponse {
            actor: Actor,
        }

        #[derive(Deserialize)]
        struct Actor {
            account: Account,
        }

        #[derive(Deserialize)]
        struct Account {
            nrql: NrqlResult,
        }

        #[derive(Deserialize)]
        struct NrqlResult {
            results: serde_json::Value,
        }

        let response: NrqlResponse = self.query(&query, None).await?;
        Ok(response.actor.account.nrql.results)
    }

    pub async fn list_incidents(&self, account_id: u64) -> Result<Vec<Incident>> {
        let query = format!(
            r#"
            {{
              actor {{
                account(id: {}) {{
                  alerts {{
                    incidents {{
                      incidents {{
                        incidentId
                        title
                        priority
                        state
                        createdAt
                        closedAt
                        policyName
                        conditionName
                      }}
                    }}
                  }}
                }}
              }}
            }}
            "#,
            account_id
        );

        #[derive(Deserialize)]
        struct IncidentsResponse {
            actor: Actor,
        }

        #[derive(Deserialize)]
        struct Actor {
            account: Account,
        }

        #[derive(Deserialize)]
        struct Account {
            alerts: Alerts,
        }

        #[derive(Deserialize)]
        struct Alerts {
            incidents: IncidentsWrapper,
        }

        #[derive(Deserialize)]
        struct IncidentsWrapper {
            incidents: Vec<Incident>,
        }

        let response: IncidentsResponse = self.query(&query, None).await?;
        Ok(response.actor.account.alerts.incidents.incidents)
    }
}

#[derive(Debug, Deserialize)]
pub struct Incident {
    #[serde(rename = "incidentId")]
    pub incident_id: String,
    pub title: String,
    pub priority: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<String>,
    #[serde(rename = "policyName")]
    pub policy_name: String,
    #[serde(rename = "conditionName")]
    pub condition_name: String,
}

// Cargo.toml dependencies
// [dependencies]
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// anyhow = "1.0"
// tokio = { version = "1", features = ["full"] }
```

**Option 2: Use `newrelic-unofficial` Crate**

**Pros**:
- Pure Rust implementation (no C SDK)
- Thread-safe
- Works standalone (no daemon required)
- Supports transactions, segments, error reporting

**Cons**:
- Community-maintained (not official)
- May not support all New Relic features
- Primarily focused on APM instrumentation

**Use Case**: If AOF needs APM-style instrumentation for tracking agent execution.

**Option 3: Use `newrelic` Crate (C SDK Wrapper)**

**Pros**:
- More mature and feature-rich
- Supports distributed tracing
- Closer to official New Relic SDK

**Cons**:
- Requires New Relic daemon to be running
- C SDK dependency (more complex build)
- Not musl-compatible (won't link against musl)

**Use Case**: If AOF requires full APM instrumentation and daemon management is acceptable.

### 5.3 Recommendation for AOF

**Recommended**: **Option 1 - Custom HTTP/GraphQL Client**

**Rationale**:
1. **Lightweight**: No daemon or C SDK dependencies
2. **Flexible**: Full control over API interactions
3. **Modern**: Uses NerdGraph (GraphQL) API
4. **Aligned with AOF architecture**: RESTful/GraphQL client pattern matches existing MCP/LLM integrations
5. **Easy to test**: Mock HTTP requests for unit tests
6. **Cross-platform**: Works on all platforms (including musl)

**Dependencies**:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
```

---

## 6. Sample API Calls for AOF Integration

### 6.1 List Active Incidents

**GraphQL Query**:
```graphql
{
  actor {
    account(id: YOUR_ACCOUNT_ID) {
      alerts {
        incidents(filter: {states: [CREATED, ACKNOWLEDGED]}) {
          incidents {
            incidentId
            title
            priority
            state
            createdAt
            policyName
            conditionName
            violationUuids
          }
        }
      }
    }
  }
}
```

**Rust Implementation**:
```rust
let incidents = client.list_incidents(account_id).await?;
for incident in incidents {
    println!("Incident: {} - {} ({})", incident.incident_id, incident.title, incident.state);
}
```

### 6.2 Execute NRQL Query

**NRQL**:
```sql
SELECT count(*) FROM Transaction WHERE appName='MyApp' FACET name SINCE 1 hour ago
```

**Rust Implementation**:
```rust
let results = client.nrql_query(
    account_id,
    "SELECT count(*) FROM Transaction WHERE appName='MyApp' FACET name SINCE 1 hour ago"
).await?;

println!("Results: {}", serde_json::to_string_pretty(&results)?);
```

### 6.3 Get Application Performance Metrics

**GraphQL Query**:
```graphql
{
  actor {
    entitySearch(query: "domain = 'APM' AND type = 'APPLICATION'") {
      results {
        entities {
          name
          ... on ApmApplicationEntity {
            apmSummary {
              errorRate
              responseTimeAverage
              throughput
              apdexScore
            }
          }
        }
      }
    }
  }
}
```

### 6.4 Create Alert Policy

**GraphQL Mutation**:
```graphql
mutation {
  alertsPolicyCreate(accountId: YOUR_ACCOUNT_ID, policy: {
    name: "My AOF Alert Policy"
    incidentPreference: PER_CONDITION
  }) {
    id
    name
  }
}
```

### 6.5 Create NRQL Alert Condition

**GraphQL Mutation**:
```graphql
mutation {
  alertsNrqlConditionStaticCreate(
    accountId: YOUR_ACCOUNT_ID
    policyId: POLICY_ID
    condition: {
      name: "High Error Rate"
      enabled: true
      nrql: {
        query: "SELECT count(*) FROM TransactionError WHERE appName='MyApp'"
      }
      signal: {
        aggregationWindow: 60
        evaluationOffset: 3
      }
      terms: [{
        threshold: 10
        thresholdOccurrences: AT_LEAST_ONCE
        thresholdDuration: 300
        operator: ABOVE
        priority: CRITICAL
      }]
      valueFunction: SINGLE_VALUE
    }
  ) {
    id
    name
  }
}
```

---

## 7. Implementation Recommendations for AOF

### 7.1 Architecture

```
AOF Integration Layer
├── NewRelicClient (HTTP/GraphQL client)
│   ├── Authentication (User API Key)
│   ├── Query Interface (NerdGraph GraphQL)
│   ├── NRQL Execution
│   └── Error Handling
├── Incident Fetcher (Poll/Webhook)
│   ├── List active incidents
│   ├── Get incident details
│   └── Update incident status
├── Metrics Collector
│   ├── APM metrics
│   ├── Infrastructure metrics
│   └── Custom NRQL queries
└── Alert Manager
    ├── Create/update policies
    ├── Create/update conditions
    └── Manage workflows
```

### 7.2 Configuration Schema

```yaml
triggers:
  - type: newrelic
    name: newrelic-alerts
    config:
      api_key: "${NEW_RELIC_API_KEY}"  # User API Key
      account_id: 1234567
      region: us  # or 'eu'
      poll_interval: 60  # seconds
      filters:
        priority: ["CRITICAL", "HIGH"]
        policy_names: ["Production Alerts"]
    actions:
      - type: invoke_agent
        agent: incident-responder
        params:
          incident: "{{ trigger.incident }}"
```

### 7.3 Key Features to Implement

1. **Incident Polling**: Periodically query NerdGraph for new incidents
2. **Webhook Support**: Future enhancement for real-time incident notifications
3. **NRQL Querying**: Execute custom NRQL queries for metrics
4. **Alert Management**: Create/update alert policies and conditions
5. **Multi-Account Support**: Handle multiple New Relic accounts
6. **Regional Support**: Support both US and EU datacenters
7. **Rate Limiting**: Implement client-side rate limiting and backoff
8. **Caching**: Cache frequently accessed data (entities, policies)

### 7.4 Testing Strategy

1. **Unit Tests**: Mock HTTP requests using `wiremock` or `mockito`
2. **Integration Tests**: Test against New Relic sandbox account
3. **Rate Limit Tests**: Verify handling of 429 responses
4. **Authentication Tests**: Test invalid API keys, expired keys
5. **Query Tests**: Validate NRQL syntax and GraphQL queries

---

## 8. References & Sources

### Official Documentation
- [Introduction to New Relic APIs](https://docs.newrelic.com/docs/apis/intro-apis/introduction-new-relic-apis/)
- [NerdGraph (GraphQL API) Documentation](https://docs.newrelic.com/docs/apis/nerdgraph/get-started/introduction-new-relic-nerdgraph/)
- [NerdGraph API Explorer Tutorial](https://docs.newrelic.com/docs/apis/nerdgraph/get-started/nerdgraph-explorer/)
- [New Relic API Keys](https://docs.newrelic.com/docs/apis/intro-apis/new-relic-api-keys/)
- [REST API v2 (Legacy)](https://docs.newrelic.com/docs/apis/rest-api-v2/get-started/introduction-new-relic-rest-api-v2/)
- [REST API Calls for Alerts](https://docs.newrelic.com/docs/alerts-applied-intelligence/new-relic-alerts/advanced-alerts/rest-api-alerts/rest-api-calls-alerts/)
- [NRQL Query Rate Limits](https://docs.newrelic.com/docs/nrql/using-nrql/rate-limits-nrql-queries/)
- [REST API Keys End-of-Life Notice](https://docs.newrelic.com/whats-new/2025/01/whats-new-03-01-rest-api-keys-eol/)

### Rust Crates
- [newrelic (crates.io)](https://crates.io/crates/newrelic)
- [newrelic - GitHub](https://github.com/sd2k/newrelic)
- [newrelic-telemetry-sdk-rust - GitHub](https://github.com/newrelic/newrelic-telemetry-sdk-rust)
- [tracing-newrelic (crates.io)](https://crates.io/crates/tracing-newrelic)
- [tokio-newrelic (lib.rs)](https://lib.rs/crates/tokio-newrelic)
- [newrelic-unofficial (crates.io)](https://crates.io/crates/newrelic-unofficial)

### Tutorials & Guides
- [Getting Started with NerdGraph (New Relic Blog)](https://newrelic.com/blog/how-to-relic/graphql-api)
- [New Relic Postman Collection](https://www.postman.com/new-relic/new-relic-graphql-api-collection/documentation/btuxnnc/new-relic-nerdgraph-graphql-api-collection)
- [New Relic API Guide (DevOpsSchool)](https://www.devopsschool.com/blog/new-relic-api-guide-step-by-step-tutorial-with-example/)

### Additional Resources
- [Incident Event REST API](https://docs.newrelic.com/docs/data-apis/ingest-apis/event-api/incident-event-rest-api/)
- [NerdGraph NRQL Tutorial](https://docs.newrelic.com/docs/apis/nerdgraph/examples/nerdgraph-nrql-tutorial/)
- [NerdGraph Entities API Tutorial](https://docs.newrelic.com/docs/apis/nerdgraph/examples/nerdgraph-entities-api-tutorial/)
- [Data Limits Documentation](https://docs.newrelic.com/docs/data-apis/manage-data/view-system-limits/)

---

## 9. Next Steps

1. **Prototype Implementation**:
   - Create `NewRelicClient` using `reqwest` and GraphQL
   - Implement authentication with User API Key
   - Test basic queries (incidents, NRQL)

2. **Configuration Integration**:
   - Define trigger schema for New Relic integration
   - Add to `aof-triggers` crate
   - Support environment variable substitution for API keys

3. **Testing**:
   - Set up New Relic test account
   - Create integration tests
   - Implement mock HTTP server for unit tests

4. **Documentation**:
   - Add New Relic trigger documentation to `docs/`
   - Create example YAML configurations
   - Document API key setup process

5. **Advanced Features** (Future):
   - Webhook support for real-time incident notifications
   - Multi-account management
   - Alert policy automation
   - Metrics collection and reporting

---

**Research Completed**: December 25, 2025
**Prepared by**: AOF Research Agent
**Status**: ✅ Ready for Implementation
