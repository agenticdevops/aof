# Splunk API Research for AOF Integration

**Research Date:** 2025-12-25
**Purpose:** Comprehensive API analysis for implementing Splunk integration in AOF
**Status:** ✅ Complete

---

## Executive Summary

Splunk provides a comprehensive REST API ecosystem for integration with external tools. For AOF integration, we have multiple options:

1. **Splunk REST API** - Full-featured API for searches, alerts, and management (recommended for primary integration)
2. **HTTP Event Collector (HEC)** - Simplified token-based ingestion for sending events
3. **Community Rust Crates** - Third-party libraries available (`splunk-rs`, `splunk-cim`)

**Recommendation:** Implement AOF integration using the REST API v2 with the community `splunk-rs` crate as a foundation.

---

## 1. Splunk REST API Overview

### Core Concepts
- **Protocol:** HTTPS on port 8089 (management port, NOT port 8000 web UI)
- **Architecture:** RESTful, follows REST architectural principles
- **API Version:** v2 (v1 deprecated and disabled by default)
- **Access:** Requires authentication via username/password or tokens
- **Data Formats:** JSON, XML, CSV (configurable via `output_mode` parameter)

### Base URL Pattern
```
https://<splunk-host>:8089/services/<endpoint>
https://<splunk-host>:8089/servicesNS/<username>/<app>/<endpoint>
```

### Available HTTP Methods
- **GET** - Retrieve resource state or list child resources
- **POST** - Create/add resources, enable/disable functionality
- **DELETE** - Remove resources

---

## 2. Authentication Methods

### Authentication Token Types

Splunk supports three types of authentication tokens for API access:

#### 2.1 Static Authentication Tokens (Recommended)
- **Lifetime:** Can last indefinitely
- **Management:** Create, modify, update, delete via Splunk Web or API
- **Use Case:** Long-lived integrations like AOF
- **Required Capabilities:**
  - `edit_tokens_own` - Create tokens for yourself
  - `edit_tokens_all` - Create tokens for any user

#### 2.2 Ephemeral Authentication Tokens
- **Lifetime:** Maximum 6 hours
- **Management:** Cannot be created in Splunk Web
- **Use Case:** Short-lived, temporary access

#### 2.3 Interactive Authentication Tokens
- **Lifetime:** Like ephemeral but with more restrictions
- **Use Case:** Limited to specific interactive scenarios

### Authentication Schemes Supported
- Native Splunk authentication
- SAML-based SSO
- LDAP (Splunk Enterprise only)

### Token Format
- **Type:** JWT (JSON Web Tokens)
- **Format:** 128-bit GUID (32-character globally unique identifier)

### API Authentication Headers

**Option 1: Bearer Token (Standard JWT)**
```bash
curl -H "Authorization: Bearer <token>" \
     https://localhost:8089/services/search/jobs
```

**Option 2: Splunk Header**
```bash
curl -H "Authorization: Splunk <token>" \
     https://localhost:8089/services/search/jobs
```

### Token Creation via API
```bash
# Create token via REST endpoint
curl -k -u username:password \
     https://localhost:8089/services/authorization/tokens \
     -d name="aof-integration-token" \
     -d audience="AOF Integration"
```

### Token Validation Process
When a token is presented, Splunk validates:
1. Token authentication is enabled
2. Token signature is valid
3. Token has not expired
4. Token has not been deleted
5. Token is enabled
6. User is authorized to use it

---

## 3. Key REST API Endpoints for AOF

### 3.1 Search Endpoints

#### Primary Search Job Endpoint (v2 - Recommended)
```
POST /services/search/v2/jobs
```
- Creates asynchronous search job
- Returns search ID (sid) for polling results
- Supports SPL (Search Processing Language) queries

**Example: Create Search Job**
```bash
curl -k -u admin:password \
     https://localhost:8089/services/search/v2/jobs \
     -d search="search index=main error | head 100" \
     -d earliest_time="-24h" \
     -d latest_time="now" \
     -d output_mode="json"
```

#### Streaming Search Export (No SID)
```
GET /services/search/jobs/export
```
- Stream results as they become available
- No search ID created
- Best for massive event export

**Example: Export Search Results**
```bash
curl -k -u admin:password \
     https://localhost:8089/services/search/jobs/export \
     --data-urlencode search="search index=_internal | head 1000" \
     -d earliest_time="-1h" \
     -d latest_time="now" \
     -d output_mode="json"
```

#### Get Search Results
```
GET /services/search/jobs/{search_id}/results
GET /services/search/jobs/{search_id}/events
```
- `/results` - For transforming searches (stats, charts)
- `/events` - For non-transforming searches (raw events)

**Example: Poll Search Results**
```bash
# Check search status
curl -k -u admin:password \
     https://localhost:8089/services/search/jobs/<sid> \
     -d output_mode="json"

# Get results when done
curl -k -u admin:password \
     https://localhost:8089/services/search/jobs/<sid>/results \
     -d output_mode="json" \
     -d count=0
```

### 3.2 Saved Searches (Alert Definitions)

#### Saved Searches Endpoint
```
GET  /services/saved/searches
POST /services/saved/searches
GET  /services/saved/searches/{name}
```

**Example: List Saved Searches**
```bash
curl -k -u admin:password \
     https://localhost:8089/services/saved/searches \
     -d output_mode="json"
```

**Example: Create Saved Search**
```bash
curl -k -u admin:password \
     https://localhost:8089/servicesNS/admin/search/saved/searches \
     -d name="AOF_Error_Monitor" \
     --data-urlencode search="index=main error | stats count by host" \
     -d cron_schedule="*/15 * * * *" \
     -d is_scheduled="1"
```

**Example: Execute Saved Search**
```bash
curl -k -u admin:password \
     https://localhost:8089/servicesNS/admin/search/saved/searches/AOF_Error_Monitor/dispatch \
     -d trigger_actions="1"
```

### 3.3 Alerts & Triggered Alerts

#### Triggered Alerts Endpoint
```
GET /services/alerts/fired_alerts
GET /servicesNS/{user}/{app}/alerts/fired_alerts
```

**Example: Get Triggered Alerts**
```bash
curl -k -u admin:password \
     https://localhost:8089/servicesNS/admin/search/alerts/fired_alerts \
     -d output_mode="json"
```

**Via Splunk Search (Internal REST)**
```spl
| rest /services/alerts/fired_alerts splunk_server=local
| table eai:acl.owner eai:acl.app id title triggered_alert_count
```

**Important:** Alerts must have "Add to Triggered Alerts" action enabled to be tracked and visible via this endpoint.

### 3.4 Index Management

```
GET /services/data/indexes
GET /services/data/indexes/{name}
```

**Example: List Indexes**
```bash
curl -k -u admin:password \
     https://localhost:8089/services/data/indexes \
     -d output_mode="json"
```

---

## 4. HTTP Event Collector (HEC)

### Overview
- **Purpose:** Send events to Splunk without a forwarder
- **Port:** 8088 (default, configurable)
- **Authentication:** Token-based (128-bit GUID)
- **Use Case:** Application logging, custom event ingestion

### HEC Endpoints

#### JSON Events
```
POST /services/collector/event
```

#### Raw Events
```
POST /services/collector/raw
```

### HEC Token Configuration
- Each token is a 32-character GUID
- Tokens configured in Splunk Web under Settings > Data Inputs > HTTP Event Collector
- No need to hardcode Splunk credentials in applications

### Event Format (JSON)

**Single Event**
```json
{
  "time": 1734938400,
  "host": "aof-host",
  "source": "aof-runtime",
  "sourcetype": "aof:agent:execution",
  "index": "main",
  "event": {
    "agent_id": "researcher-001",
    "status": "completed",
    "duration_ms": 1234
  }
}
```

**Multiple Events**
```json
{"event": {"message": "Event 1"}, "time": 1734938400}
{"event": {"message": "Event 2"}, "time": 1734938401}
```

### HEC Authentication
```bash
curl -k https://localhost:8088/services/collector/event \
     -H "Authorization: Splunk <HEC-TOKEN>" \
     -d '{"event": {"message": "Hello from AOF"}}'
```

### Indexer Acknowledgment (Optional)
- Default: HTTP 200 sent immediately upon receipt
- With Ack: Confirms event entered processing pipeline
- Requires channel identifiers (GUIDs)

---

## 5. Rate Limits & Best Practices

### Rate Limits

#### Splunk Intelligence Management API
- **Per User:** 60 API calls per minute
- **Per IP:** 1000 API calls per 5 minutes
- **Daily Limit (Community+):** 300 API calls per day
- **Error Code:** HTTP 429 "Too Many Requests"
- **Response Field:** `waitTime` (seconds to wait before retry)

#### Core Splunk Platform
- **No Built-in Rate Limiting:** Core Splunk REST API does not have enforced rate limits
- **Recommendation:** Use external rate limiting (nginx, API gateway)
- **Role-Based Quotas:** Assign quotas to REST API user roles

#### Result Row Limits
- **Default Max Results:** 50,000 rows (`maxresultrows` in `limits.conf`)
- **Pagination Required:** Use offset/count for large result sets

### Best Practices

#### 1. Pagination for Large Results
```bash
# First batch
curl "...?offset=0&count=50000"

# Second batch
curl "...?offset=50000&count=50000"
```

#### 2. Time Range Restrictions
**CRITICAL:** Always use time modifiers to avoid inefficient `alltime` searches
```bash
-d earliest_time="-24h" \
-d latest_time="now"
```

#### 3. Use Appropriate Endpoints
- **Massive Export:** Use `/services/search/jobs/export`
- **Non-Transforming:** Use `/services/search/jobs/{sid}/events`
- **Transforming:** Use `/services/search/jobs/{sid}/results`

#### 4. Async Search Pattern
1. Create search job (POST to `/search/v2/jobs`)
2. Poll for completion (GET `/search/jobs/{sid}`)
3. Retrieve results when `isDone=true`

#### 5. Splunk Cloud Platform Restrictions
- Free trial accounts: **No REST API access**
- Authentication tokens required (create after getting access)
- **Search tier only access** (no management operations)

---

## 6. Rust SDK & Integration Options

### Official Rust Support
**Status:** ❌ No official Splunk SDK for Rust

Splunk does not provide an official Rust SDK. Official SDKs exist for:
- Python (`splunklib`)
- JavaScript/Node.js (`splunk-sdk`)
- Java
- C#

### Community Rust Crates

#### 6.1 `splunk-rs` (Recommended)
**Author:** yaleman
**GitHub:** https://github.com/yaleman/splunk-rs
**Docs:** https://yaleman.github.io/splunk-rs/splunk/

**Features:**
- ✅ Async/non-blocking (Tokio-based)
- ✅ HTTP Event Collector (HEC) support
- ✅ Search capabilities
- ✅ Batch event submission (`send_events`, `enqueue`, `flush`)
- ✅ Server configuration management

**Example Usage:**
```rust
use splunk::{HecClient, ServerConfig};

let config = ServerConfig::new("https://splunk.example.com:8088", "HEC-TOKEN");
let client = HecClient::new(config);

// Single event
client.send_event(event).await?;

// Batch events
client.enqueue(event1);
client.enqueue(event2);
client.flush().await?;
```

**Installation:**
```toml
[dependencies]
splunk = "0.x"  # Check crates.io for latest version
```

#### 6.2 `splunk-cim`
**Purpose:** Splunk Common Information Model (CIM) type definitions
**Docs:** https://docs.rs/splunk-cim

**Features:**
- Type-safe Splunk CIM field definitions
- Serialization/deserialization support

**Use Case:** When working with Splunk's standardized data models

#### 6.3 OpenTelemetry for Rust (Splunk Fork)
**GitHub:** https://github.com/splunk/opentelemetry-rust-latest

**Features:**
- Instrumentation for traces/spans
- Export to OpenTelemetry Collector
- Forward to Splunk Observability Cloud

**Use Case:** Observability and APM, not traditional log search

### Recommended Integration Architecture for AOF

```
┌─────────────────────────────────────────────────┐
│              AOF Runtime                        │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │   Splunk MCP Tool Implementation        │  │
│  │                                          │  │
│  │  ┌────────────────────────────────────┐ │  │
│  │  │  splunk-rs (HTTP Client)          │ │  │
│  │  │  - Async REST API calls            │ │  │
│  │  │  - HEC event submission            │ │  │
│  │  └────────────────────────────────────┘ │  │
│  │                                          │  │
│  │  ┌────────────────────────────────────┐ │  │
│  │  │  Custom REST API Layer            │ │  │
│  │  │  - Search job management           │ │  │
│  │  │  - Alert queries                   │ │  │
│  │  │  - Saved search execution          │ │  │
│  │  └────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
                    ▼
        ┌───────────────────────┐
        │   Splunk Platform     │
        │  (Port 8089 - REST)   │
        │  (Port 8088 - HEC)    │
        └───────────────────────┘
```

---

## 7. Sample API Calls for AOF Integration

### 7.1 Authentication & Setup

```rust
use reqwest::{Client, header};

async fn create_splunk_client(base_url: &str, token: &str) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
    );

    Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true) // Only for dev/testing
        .build()
        .unwrap()
}
```

### 7.2 Run SPL Search

```rust
async fn run_splunk_search(
    client: &Client,
    base_url: &str,
    query: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}/services/search/v2/jobs", base_url);

    let params = [
        ("search", query),
        ("earliest_time", "-24h"),
        ("latest_time", "now"),
        ("output_mode", "json"),
    ];

    let response = client
        .post(&url)
        .form(&params)
        .send()
        .await?;

    let body = response.text().await?;

    // Parse JSON to extract sid
    let json: serde_json::Value = serde_json::from_str(&body)?;
    let sid = json["sid"].as_str().unwrap();

    Ok(sid.to_string())
}
```

### 7.3 Poll Search Results

```rust
async fn get_search_results(
    client: &Client,
    base_url: &str,
    sid: &str
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    // Check if search is done
    let status_url = format!("{}/services/search/jobs/{}", base_url, sid);

    loop {
        let status_resp = client
            .get(&status_url)
            .query(&[("output_mode", "json")])
            .send()
            .await?;

        let status: serde_json::Value = status_resp.json().await?;

        if status["entry"][0]["content"]["isDone"].as_bool().unwrap_or(false) {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Retrieve results
    let results_url = format!("{}/services/search/jobs/{}/results", base_url, sid);
    let results_resp = client
        .get(&results_url)
        .query(&[("output_mode", "json"), ("count", "0")])
        .send()
        .await?;

    let results: serde_json::Value = results_resp.json().await?;
    let events = results["results"].as_array().unwrap().clone();

    Ok(events)
}
```

### 7.4 Get Triggered Alerts

```rust
async fn get_triggered_alerts(
    client: &Client,
    base_url: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let url = format!("{}/servicesNS/-/-/alerts/fired_alerts", base_url);

    let response = client
        .get(&url)
        .query(&[("output_mode", "json")])
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;
    let alerts = body["entry"].as_array().unwrap().clone();

    Ok(alerts)
}
```

### 7.5 Send Events via HEC

```rust
use splunk::{HecClient, ServerConfig, Event};

async fn send_aof_event(
    hec_url: &str,
    hec_token: &str,
    event_data: serde_json::Value
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(hec_url, hec_token);
    let client = HecClient::new(config);

    let event = Event::new(event_data)
        .host("aof-host")
        .source("aof-runtime")
        .sourcetype("aof:agent:execution")
        .index("main");

    client.send_event(event).await?;

    Ok(())
}
```

---

## 8. AOF Integration Recommendations

### 8.1 Recommended MCP Tools

Based on the research, AOF should implement the following Splunk MCP tools:

#### Search Tools
1. **`splunk_search`** - Execute SPL queries
   - Input: SPL query, time range
   - Output: Search results (JSON)
   - Use case: Ad-hoc log analysis by agents

2. **`splunk_export`** - Stream large result sets
   - Input: SPL query, time range
   - Output: Streaming events
   - Use case: Bulk data export

#### Alert Tools
3. **`splunk_get_alerts`** - Fetch triggered alerts
   - Input: Time range, filter criteria
   - Output: List of fired alerts
   - Use case: Monitor system health

4. **`splunk_create_alert`** - Create saved search/alert
   - Input: Name, SPL, schedule, actions
   - Output: Alert configuration
   - Use case: Automated monitoring setup

#### Event Ingestion Tools
5. **`splunk_send_event`** - Send events via HEC
   - Input: Event data, metadata
   - Output: Acknowledgment
   - Use case: AOF agent execution logging

#### Index Management
6. **`splunk_list_indexes`** - List available indexes
   - Output: Index names and metadata
   - Use case: Discover data sources

### 8.2 Implementation Approach

```rust
// File: crates/aof-mcp/src/servers/splunk.rs

pub struct SplunkMcpServer {
    rest_client: Client,
    hec_client: HecClient,
    base_url: String,
    token: SecretString,
}

impl SplunkMcpServer {
    pub async fn execute_search(&self, query: &str) -> Result<Vec<Event>> {
        // 1. Create search job
        // 2. Poll until complete
        // 3. Return results
    }

    pub async fn get_triggered_alerts(&self) -> Result<Vec<Alert>> {
        // Query /alerts/fired_alerts
    }

    pub async fn send_event(&self, event: Event) -> Result<()> {
        // Use HEC client
    }
}
```

### 8.3 Configuration Schema

```yaml
# Example: aof-fleet.yaml
spec:
  mcp:
    servers:
      - name: splunk-prod
        command: aof-mcp
        args: ["splunk"]
        env:
          SPLUNK_HOST: https://splunk.example.com:8089
          SPLUNK_HEC_URL: https://splunk.example.com:8088
          SPLUNK_TOKEN: ${SPLUNK_API_TOKEN}
          SPLUNK_HEC_TOKEN: ${SPLUNK_HEC_TOKEN}
        tools:
          - splunk_search
          - splunk_get_alerts
          - splunk_send_event
```

### 8.4 Dependencies

```toml
# crates/aof-mcp/Cargo.toml

[dependencies]
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
splunk = "0.x"  # Community crate for HEC
anyhow = "1.0"
secrecy = "0.8"
```

---

## 9. Security Considerations

### 9.1 Token Management
- ✅ Use static authentication tokens (not ephemeral)
- ✅ Store tokens in secure secret management (e.g., environment variables, vaults)
- ✅ Never hardcode tokens in code
- ✅ Use `secrecy::SecretString` in Rust to prevent accidental logging

### 9.2 TLS/SSL
- ✅ Always use HTTPS in production
- ✅ Validate certificates (avoid `danger_accept_invalid_certs` in production)
- ✅ Support custom CA certificates for self-signed Splunk instances

### 9.3 Role-Based Access
- ✅ Create dedicated Splunk user for AOF integration
- ✅ Grant minimum required capabilities:
  - `search` - Run searches
  - `edit_tokens_own` - Manage own tokens
  - Index-specific read permissions

### 9.4 Input Validation
- ✅ Sanitize SPL queries to prevent injection attacks
- ✅ Validate time ranges
- ✅ Limit result set sizes

---

## 10. Next Steps for Implementation

### Phase 1: Core Search (Week 1)
- [ ] Implement `SplunkMcpServer` struct
- [ ] Add REST API client with authentication
- [ ] Implement `splunk_search` tool
- [ ] Add search job polling logic
- [ ] Unit tests with mock Splunk responses

### Phase 2: Alerts (Week 2)
- [ ] Implement `splunk_get_alerts` tool
- [ ] Add saved search listing
- [ ] Implement alert creation/management
- [ ] Integration tests with Splunk sandbox

### Phase 3: Event Ingestion (Week 3)
- [ ] Integrate `splunk-rs` HEC client
- [ ] Implement `splunk_send_event` tool
- [ ] Add batch event submission
- [ ] Performance testing

### Phase 4: Documentation & Examples (Week 4)
- [ ] Update `/docs/mcp/providers/splunk.md`
- [ ] Create example fleet configurations
- [ ] Add tutorial for Splunk integration
- [ ] API reference documentation

---

## Sources

### REST API Documentation
- [Using the REST API reference | Splunk Docs](https://help.splunk.com/en/splunk-enterprise/rest-api-reference/10.0/introduction/using-the-rest-api-reference)
- [Basic concepts about the Splunk platform REST API | Splunk Docs](https://docs.splunk.com/Documentation/Splunk/9.4.2/RESTUM/RESTusing)
- [Creating searches using the REST API | Splunk Docs](https://help.splunk.com/en/splunk-enterprise/leverage-rest-apis/rest-api-tutorials/10.0/rest-api-tutorials/creating-searches-using-the-rest-api)
- [REST API Reference | Splunk Docs](https://help.splunk.com/en/splunk-cloud-platform/rest-api-reference)

### Search API
- [Search endpoint descriptions | Splunk Docs](https://docs.splunk.com/Documentation/SplunkCloud/latest/RESTREF/RESTsearch)
- [Splunk Searching with REST API - Hurricane Labs](https://hurricanelabs.com/splunk-tutorials/splunk-searching-with-rest-api/)
- [Running Searches Using Splunk's REST API | TekStream Solutions](https://www.tekstream.com/blog/splunk-rest-api-part1/)

### Rust SDK
- [GitHub - yaleman/splunk-rs](https://github.com/yaleman/splunk-rs)
- [splunk - Rust](https://docs.rs/splunk)
- [splunk_cim - Rust](https://docs.rs/splunk-cim)
- [GitHub - splunk/opentelemetry-rust-latest](https://github.com/splunk/opentelemetry-rust-latest)

### HEC Documentation
- [Set up and use HTTP Event Collector | Splunk Docs](https://docs.splunk.com/Documentation/Splunk/9.4.2/Data/UsetheHTTPEventCollector)
- [HTTP Event Collector examples | Splunk Docs](https://docs.splunk.com/Documentation/Splunk/9.4.2/Data/HECExamples)
- [HEC configuration | Splunk Developer Program](https://dev.splunk.com/enterprise/docs/devtools/httpeventcollector/)
- [Format events for HTTP Event Collector | Splunk Docs](https://docs.splunk.com/Documentation/Splunk/9.4.2/Data/FormateventsforHTTPEventCollector)

### Authentication
- [Use authentication tokens | Splunk Docs](https://docs.splunk.com/Documentation/SplunkCloud/latest/Security/UseAuthTokens)
- [Create authentication tokens | Splunk Docs](https://docs.splunk.com/Documentation/SplunkCloud/latest/Security/CreateAuthTokens)
- [Make REST API calls with Authentication Tokens](https://splunk.my.site.com/customer/s/article/Make-REST-API-calls-with-Authentication-Tokens)

### Alerts API
- [How to work with alerts using the Splunk Enterprise SDK](https://dev.splunk.com/goto/workwithalerts/)
- [List of Alerts via REST](https://gosplunk.com/list-of-alerts-via-rest/)

### Saved Searches
- [Work with saved searches | Splunk Developer Program](https://dev.splunk.com/view/SP-CAAAEKZ)
- [Creating searches using the REST API | Splunk Docs](https://docs.splunk.com/Documentation/SplunkCloud/latest/RESTTUT/RESTsearches)

### Rate Limits & Best Practices
- [Policy for API usage in Splunk Intelligence Management](https://docs.splunk.com/Documentation/SIM/current/Intro/UsagePolicy)
- [Access requirements and limitations for the Splunk Cloud Platform REST API](https://docs.splunk.com/Documentation/SplunkCloud/latest/RESTTUT/RESTandCloud)

---

**End of Research Report**
